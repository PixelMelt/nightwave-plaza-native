use crate::api::StatusSong;
use crate::state::Msg;
use futures::channel::mpsc::UnboundedSender;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal::DeviceId;
use rodio::mixer::MixerSource;
use rodio::{
    buffer::SamplesBuffer, ChannelCount, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player,
    Sample, SampleRate, Source,
};
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use std::io::{self, Read, Seek, SeekFrom};
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const STREAM_URL: &str = "https://radio.plaza.one/mp3";

const PREBUFFER: Duration = Duration::from_secs(5);
const CHUNK: Duration = Duration::from_millis(250);

const DEFAULT_VOLUME: f32 = 0.5;
const RECONNECT_DELAY: Duration = Duration::from_secs(2);
/// How often the streaming thread checks whether the OS default output device changed.
const DEVICE_POLL: Duration = Duration::from_secs(2);

/// Format of the app-owned mixer that sits between the player and whatever
/// output device is currently open.
const MIX_CHANNELS: ChannelCount = NonZero::new(2).unwrap();
const MIX_RATE: SampleRate = NonZero::new(44100).unwrap();

pub struct AudioPlayer {
    shared: Arc<Shared>,
    controls: Mutex<Option<MediaControls>>,
    progress: Mutex<Option<Duration>>,
}

/// State shared between the UI, the streaming thread and the device error callback.
struct Shared {
    /// Player connected to the app-owned mixer; survives output device changes.
    player: Player,
    /// Output side of the app-owned mixer, pulled by the currently open device sink.
    relay: Arc<Mutex<MixerSource>>,
    playing: AtomicBool,
    streaming: AtomicBool,
    /// Set by the device error callback (e.g. device unplugged); forces a reopen.
    device_lost: Arc<AtomicBool>,
    wake: Condvar,
    wake_lock: Mutex<()>,
    events: UnboundedSender<Msg>,
}

impl AudioPlayer {
    /// Never fails: the output device is opened lazily by the streaming thread,
    /// so a missing or broken device only delays playback.
    pub fn new(events: UnboundedSender<Msg>) -> Self {
        let (mixer, relay) = rodio::mixer::mixer(MIX_CHANNELS, MIX_RATE);
        let player = Player::connect_new(&mixer);
        player.set_volume(DEFAULT_VOLUME);

        let shared = Arc::new(Shared {
            player,
            relay: Arc::new(Mutex::new(relay)),
            playing: AtomicBool::new(true),
            streaming: AtomicBool::new(false),
            device_lost: Arc::new(AtomicBool::new(false)),
            wake: Condvar::new(),
            wake_lock: Mutex::new(()),
            events: events.clone(),
        });

        #[cfg(not(target_os = "windows"))]
        let controls = build_controls(events).ok();
        #[cfg(target_os = "windows")]
        let controls: Option<MediaControls> = None;

        let this = Self {
            shared: shared.clone(),
            controls: Mutex::new(controls),
            progress: Mutex::new(None),
        };
        std::thread::spawn(move || stream_forever(shared));
        this.emit_playback();
        this
    }

    pub fn is_playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    pub fn is_streaming(&self) -> bool {
        self.shared.streaming.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, vol: f32) {
        self.shared.player.set_volume(vol.clamp(0.0, 1.0));
    }

    pub fn play(&self) {
        // Hold the wake lock while flipping the flag so the streaming thread
        // cannot miss the notification between its check and its wait.
        let guard = self.shared.wake_lock.lock().unwrap();
        if self.shared.playing.swap(true, Ordering::Relaxed) {
            return;
        }
        self.shared.player.play();
        self.shared.wake.notify_all();
        drop(guard);
        self.emit_playback();
    }

    pub fn stop(&self) {
        if !self.shared.playing.swap(false, Ordering::Relaxed) {
            return;
        }
        self.shared.player.clear();
        self.emit_playback();
    }

    pub fn update_metadata(&self, song: &StatusSong) {
        let length = (song.length > 0.0).then(|| Duration::from_secs_f64(song.length));
        *self.progress.lock().unwrap() = Some(Duration::from_secs_f64(song.position));

        if let Some(controls) = self.controls.lock().unwrap().as_mut() {
            let _ = controls.set_metadata(MediaMetadata {
                title: opt_str(&song.title),
                album: opt_str(&song.album),
                artist: opt_str(&song.artist),
                cover_url: song.artwork_src.as_deref(),
                duration: length,
            });
        }

        self.emit_playback();
    }

    fn emit_playback(&self) {
        let progress = (*self.progress.lock().unwrap()).map(MediaPosition);
        let playback = if self.is_playing() {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };
        if let Some(controls) = self.controls.lock().unwrap().as_mut() {
            let _ = controls.set_playback(playback);
        }
    }
}

impl Shared {
    fn set_streaming(&self, streaming: bool) {
        if self.streaming.swap(streaming, Ordering::Relaxed) != streaming {
            let _ = self.events.unbounded_send(Msg::StreamChanged);
        }
    }
}

fn opt_str(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

#[cfg(not(target_os = "windows"))]
fn build_controls(tx: UnboundedSender<Msg>) -> Result<MediaControls, souvlaki::Error> {
    let config = souvlaki::PlatformConfig {
        dbus_name: "nightwave_plaza",
        display_name: "Nightwave Plaza",
        hwnd: None,
    };
    let mut controls = MediaControls::new(config)?;
    controls.attach(move |event| {
        let _ = tx.unbounded_send(Msg::Media(event));
    })?;
    Ok(controls)
}

/// Feeds the app-owned mixer's output into a device sink's mixer. A fresh one
/// is attached to every device sink we open, so the player's queue carries
/// over untouched when the output device changes.
struct Relay(Arc<Mutex<MixerSource>>);

impl Iterator for Relay {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        Some(self.0.lock().unwrap().next().unwrap_or(0.0))
    }
}

impl Source for Relay {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        MIX_CHANNELS
    }

    fn sample_rate(&self) -> SampleRate {
        MIX_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// The currently open output device, owned by the streaming thread.
struct Output {
    _sink: MixerDeviceSink,
    device: Option<DeviceId>,
    checked: Instant,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn default_device_id() -> Option<DeviceId> {
    rodio::cpal::default_host()
        .default_output_device()?
        .id()
        .ok()
}

fn open_output(shared: &Shared) -> Result<Output, BoxError> {
    let host = rodio::cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no audio output device")?;
    let id = device.id().ok();

    let open = |device: rodio::cpal::Device| -> Result<MixerDeviceSink, BoxError> {
        let lost = shared.device_lost.clone();
        let mut sink = DeviceSinkBuilder::from_device(device)?
            .with_error_callback(move |e| {
                eprintln!("Audio device error: {e}");
                lost.store(true, Ordering::Relaxed);
            })
            .open_stream()?;
        sink.log_on_drop(false);
        Ok(sink)
    };

    // Like rodio's open_default_sink: fall back to any other working device.
    let sink = open(device).or_else(|err| {
        host.output_devices()
            .ok()
            .and_then(|devices| devices.filter_map(|d| open(d).ok()).next())
            .ok_or(err)
    })?;
    sink.mixer().add(Relay(shared.relay.clone()));

    Ok(Output {
        _sink: sink,
        device: id,
        checked: Instant::now(),
    })
}

/// Makes sure a usable output is open, reopening it after a device error or
/// when the OS default output device has changed.
fn ensure_output(output: &mut Option<Output>, shared: &Shared) -> Result<(), BoxError> {
    if let Some(out) = output.as_mut() {
        let lost = shared.device_lost.swap(false, Ordering::Relaxed);
        if !lost && out.checked.elapsed() < DEVICE_POLL {
            return Ok(());
        }
        if !lost && default_device_id() == out.device {
            out.checked = Instant::now();
            return Ok(());
        }
        // Drop the old sink first so only one relay pulls from the mixer.
        *output = None;
        let reopened = open_output(shared)?;
        eprintln!("Audio output reopened on {:?}", reopened.device);
        *output = Some(reopened);
        return Ok(());
    }
    *output = Some(open_output(shared)?);
    Ok(())
}

fn stream_forever(shared: Arc<Shared>) {
    let mut output: Option<Output> = None;
    loop {
        {
            let mut guard = shared.wake_lock.lock().unwrap();
            if !shared.playing.load(Ordering::Relaxed) {
                // Release the device while stopped so it can power down.
                output = None;
            }
            while !shared.playing.load(Ordering::Relaxed) {
                guard = shared.wake.wait(guard).unwrap();
            }
        }
        if let Err(e) = stream_once(&shared, &mut output) {
            eprintln!("Audio stream error: {e}");
        }
        shared.set_streaming(false);
        if shared.playing.load(Ordering::Relaxed) {
            std::thread::sleep(RECONNECT_DELAY);
        }
    }
}

fn stream_once(shared: &Shared, output: &mut Option<Output>) -> Result<(), BoxError> {
    ensure_output(output, shared)?;

    let nocache = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let resp = crate::net::agent()
        .get(&format!("{STREAM_URL}?nocache={nocache}"))
        .call()?;
    let decoder = Decoder::new_mp3(StreamReader::new(resp.into_reader()))?;

    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    let per_sec = sample_rate.get() as usize * channels.get() as usize;
    let prebuffer_len = per_sec * PREBUFFER.as_millis() as usize / 1000;
    let chunk_len = (per_sec * CHUNK.as_millis() as usize / 1000).max(1);

    let mut buf: Vec<f32> = Vec::with_capacity(prebuffer_len);
    let mut threshold = prebuffer_len;

    for sample in decoder {
        buf.push(sample);
        if buf.len() < threshold {
            continue;
        }
        if !shared.playing.load(Ordering::Relaxed) {
            return Ok(());
        }
        ensure_output(output, shared)?;
        let chunk = std::mem::replace(&mut buf, Vec::with_capacity(chunk_len));
        shared
            .player
            .append(SamplesBuffer::new(channels, sample_rate, chunk));
        if threshold != chunk_len {
            threshold = chunk_len;
            shared.set_streaming(true);
        }
    }

    Ok(())
}

struct StreamReader {
    inner: Box<dyn Read + Send + Sync>,
    pos: u64,
}

impl StreamReader {
    fn new(inner: Box<dyn Read + Send + Sync>) -> Self {
        Self { inner, pos: 0 }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

/// The decoder probes with seeks; a live stream can only skip forward.
impl Seek for StreamReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(off) => (self.pos as i64).saturating_add(off) as u64,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "cannot seek from the end of a live stream",
                ))
            }
        };
        if target < self.pos {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "cannot seek backward in a live stream",
            ));
        }
        let mut remaining = target - self.pos;
        let mut scratch = [0u8; 8192];
        while remaining > 0 {
            let want = remaining.min(scratch.len() as u64) as usize;
            let n = self.read(&mut scratch[..want])?;
            if n == 0 {
                break;
            }
            remaining -= n as u64;
        }
        Ok(self.pos)
    }
}
