use crate::api::StatusSong;
use crate::state::Msg;
use chrono::Utc;
use futures::channel::mpsc::UnboundedSender;
use rodio::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::cpal::{
    self, BufferSize, DeviceId, FromSample, SampleFormat, SizedSample, StreamConfig,
};
use rodio::source::UniformSourceIterator;
use rodio::{ChannelCount, Decoder, SampleRate, Source};
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const STREAM_URL: &str = "https://radio.plaza.one/mp3";

const PREBUFFER: Duration = Duration::from_secs(5);
const DEFAULT_KBPS: usize = 128;
const PIPE_CAP: usize = 512 * 1024;
const CHUNK: Duration = Duration::from_millis(250);
const QUEUE_AHEAD: usize = 3;

const RECONNECT_DELAY: Duration = Duration::from_secs(2);
const DEVICE_POLL: Duration = Duration::from_secs(5);
const DEVICE_BUFFER: Duration = Duration::from_millis(200);

const MIX_CHANNELS: u16 = 2;
const MIX_RATE: u32 = 44100;

pub struct AudioPlayer {
    shared: Arc<Shared>,
    controls: Option<MediaControls>,
    progress: Option<Duration>,
}

struct Shared {
    feed: Mutex<VecDeque<f32>>,
    volume: AtomicU32,
    playing: AtomicBool,
    streaming: AtomicBool,
    device_lost: AtomicBool,
    wake: Condvar,
    wake_lock: Mutex<()>,
    events: UnboundedSender<Msg>,
}

impl AudioPlayer {
    pub fn new(events: UnboundedSender<Msg>) -> Self {
        let shared = Arc::new(Shared {
            feed: Mutex::new(VecDeque::new()),
            volume: AtomicU32::new(1.0f32.to_bits()),
            playing: AtomicBool::new(true),
            streaming: AtomicBool::new(false),
            device_lost: AtomicBool::new(false),
            wake: Condvar::new(),
            wake_lock: Mutex::new(()),
            events: events.clone(),
        });

        #[cfg(not(target_os = "windows"))]
        let controls = build_controls(events).ok();
        #[cfg(target_os = "windows")]
        let controls: Option<MediaControls> = None;

        let mut this = Self {
            shared: shared.clone(),
            controls,
            progress: None,
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
        self.shared
            .volume
            .store(vol.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn play(&mut self) {
        let guard = self.shared.wake_lock.lock().unwrap();
        if self.shared.playing.swap(true, Ordering::Relaxed) {
            return;
        }
        self.shared.wake.notify_all();
        drop(guard);
        self.emit_playback();
    }

    pub fn stop(&mut self) {
        if !self.shared.playing.swap(false, Ordering::Relaxed) {
            return;
        }
        self.shared.clear();
        self.emit_playback();
    }

    pub fn update_metadata(&mut self, song: &StatusSong) {
        let length = (song.length > 0.0).then(|| Duration::from_secs_f64(song.length));
        self.progress = Some(Duration::from_secs_f64(song.position));

        if let Some(controls) = self.controls.as_mut() {
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

    fn emit_playback(&mut self) {
        let progress = self.progress.map(MediaPosition);
        let playback = if self.is_playing() {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };
        if let Some(controls) = self.controls.as_mut() {
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

    fn queued(&self) -> usize {
        self.feed.lock().unwrap().len()
    }

    fn clear(&self) {
        self.feed.lock().unwrap().clear();
    }

    fn render<T: SizedSample + FromSample<f32>>(&self, out: &mut [T]) {
        let volume = f32::from_bits(self.volume.load(Ordering::Relaxed));
        let mut feed = self.feed.lock().unwrap();
        let n = out.len().min(feed.len());
        for (o, s) in out.iter_mut().zip(feed.drain(..n)) {
            *o = T::from_sample(s * volume);
        }
        drop(feed);
        out[n..].fill(T::EQUILIBRIUM);
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

type Format = (ChannelCount, SampleRate);

struct Output {
    _stream: cpal::Stream,
    format: Format,
    device: Option<DeviceId>,
    checked: Instant,
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn default_device_id() -> Option<DeviceId> {
    cpal::default_host().default_output_device()?.id().ok()
}

fn build_stream<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: &StreamConfig,
    shared: &Arc<Shared>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let feed = shared.clone();
    let lost = shared.clone();
    device.build_output_stream::<T, _, _>(
        config,
        move |out, _| feed.render(out),
        move |e| {
            eprintln!("Audio device error: {e}");
            lost.device_lost.store(true, Ordering::Relaxed);
        },
        None,
    )
}

fn open_stream(
    device: &cpal::Device,
    shared: &Arc<Shared>,
    rate: Option<u32>,
) -> Result<(cpal::Stream, Format), BoxError> {
    let default = device.default_output_config()?;
    let mut config = default.config();
    if let Some(rate) = rate {
        config.sample_rate = rate;
        config.channels = MIX_CHANNELS;
        config.buffer_size = BufferSize::Fixed(rate * DEVICE_BUFFER.as_millis() as u32 / 1000);
    }
    let format = (
        ChannelCount::new(config.channels).ok_or("device has no channels")?,
        SampleRate::new(config.sample_rate).ok_or("device has no sample rate")?,
    );

    macro_rules! build {
        ($($variant:ident => $ty:ty),+) => {
            match default.sample_format() {
                $(SampleFormat::$variant => build_stream::<$ty>(device, &config, shared)?,)+
                other => return Err(format!("unsupported sample format {other}").into()),
            }
        };
    }
    let stream = build!(
        F32 => f32, F64 => f64,
        I8 => i8, I16 => i16, I24 => cpal::I24, I32 => i32, I64 => i64,
        U8 => u8, U16 => u16, U24 => cpal::U24, U32 => u32, U64 => u64
    );
    stream.play()?;
    Ok((stream, format))
}

fn open_output(shared: &Arc<Shared>) -> Result<Output, BoxError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no audio output device")?;
    let id = device.id().ok();

    let open = |device: cpal::Device| -> Result<(cpal::Stream, Format), BoxError> {
        let default_rate = device.default_output_config().ok().map(|c| c.sample_rate());
        open_stream(&device, shared, Some(MIX_RATE))
            .or_else(|_| open_stream(&device, shared, default_rate))
            .or_else(|_| open_stream(&device, shared, None))
    };

    let (stream, format) = open(device).or_else(|err| {
        host.output_devices()
            .ok()
            .and_then(|devices| devices.filter_map(|d| open(d).ok()).next())
            .ok_or(err)
    })?;

    Ok(Output {
        _stream: stream,
        format,
        device: id,
        checked: Instant::now(),
    })
}

fn ensure_output(output: &mut Option<Output>, shared: &Arc<Shared>) -> Result<Format, BoxError> {
    if let Some(out) = output.as_mut() {
        let lost = shared.device_lost.swap(false, Ordering::Relaxed);
        if !lost && out.checked.elapsed() < DEVICE_POLL {
            return Ok(out.format);
        }
        if !lost && default_device_id() == out.device {
            out.checked = Instant::now();
            return Ok(out.format);
        }
        *output = None;
        let reopened = open_output(shared)?;
        eprintln!("Audio output reopened on {:?}", reopened.device);
        return Ok(output.insert(reopened).format);
    }
    Ok(output.insert(open_output(shared)?).format)
}

fn stream_forever(shared: Arc<Shared>) {
    let mut output: Option<Output> = None;
    loop {
        {
            let mut guard = shared.wake_lock.lock().unwrap();
            if !shared.playing.load(Ordering::Relaxed) {
                output = None;
                shared.clear();
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

fn stream_once(shared: &Arc<Shared>, output: &mut Option<Output>) -> Result<(), BoxError> {
    ensure_output(output, shared)?;

    let resp = crate::net::agent()
        .get(&format!(
            "{STREAM_URL}?nocache={}",
            Utc::now().timestamp_millis()
        ))
        .call()?;
    let kbps = resp
        .header("icy-br")
        .and_then(|br| br.trim().parse().ok())
        .unwrap_or(DEFAULT_KBPS);
    let prebuffer_bytes = kbps * 125 * PREBUFFER.as_millis() as usize / 1000;

    let pipe = Arc::new(Pipe::default());
    let feeder = pipe.clone();
    let mut body = resp.into_reader();
    std::thread::spawn(move || feeder.fill(&mut body));
    let reader = PipeReader { pipe, pos: 0 };
    if !reader.pipe.wait_for(prebuffer_bytes) || !shared.playing.load(Ordering::Relaxed) {
        return Ok(());
    }

    let decoder = Decoder::new_mp3(reader)?;
    let format = ensure_output(output, shared)?;
    if (decoder.channels(), decoder.sample_rate()) == format {
        pump(shared, output, format, decoder)
    } else {
        pump(
            shared,
            output,
            format,
            UniformSourceIterator::new(decoder, format.0, format.1),
        )
    }
}

fn pump(
    shared: &Arc<Shared>,
    output: &mut Option<Output>,
    format: Format,
    samples: impl Iterator<Item = f32>,
) -> Result<(), BoxError> {
    let channels = format.0.get() as usize;
    let frames = (format.1.get() as usize * CHUNK.as_millis() as usize / 1000).max(1);
    let chunk_len = frames * channels;
    let mut buf: Vec<f32> = Vec::with_capacity(chunk_len);

    for sample in samples {
        buf.push(sample);
        if buf.len() < chunk_len {
            continue;
        }
        while shared.queued() >= chunk_len * QUEUE_AHEAD && shared.playing.load(Ordering::Relaxed) {
            std::thread::sleep(CHUNK);
        }
        if !shared.playing.load(Ordering::Relaxed) {
            return Ok(());
        }
        if ensure_output(output, shared)? != format {
            shared.clear();
            return Err("audio output format changed".into());
        }
        shared.feed.lock().unwrap().extend(buf.drain(..));
        shared.set_streaming(true);
    }

    Ok(())
}

#[derive(Default)]
struct Pipe {
    state: Mutex<PipeState>,
    changed: Condvar,
}

#[derive(Default)]
struct PipeState {
    buf: VecDeque<u8>,
    done: bool,
    error: Option<io::Error>,
    closed: bool,
}

impl Pipe {
    fn fill(&self, body: &mut dyn Read) {
        let mut scratch = [0u8; 16 * 1024];
        let error = loop {
            let n = match body.read(&mut scratch) {
                Ok(0) => break None,
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("Audio stream read error: {e}");
                    break Some(e);
                }
            };
            let mut state = self.state.lock().unwrap();
            while state.buf.len() >= PIPE_CAP && !state.closed {
                state = self.changed.wait(state).unwrap();
            }
            if state.closed {
                return;
            }
            state.buf.extend(&scratch[..n]);
            self.changed.notify_all();
        };
        let mut state = self.state.lock().unwrap();
        state.done = true;
        state.error = error;
        self.changed.notify_all();
    }

    fn wait_for(&self, bytes: usize) -> bool {
        let mut state = self.state.lock().unwrap();
        while state.buf.len() < bytes && !state.done {
            state = self.changed.wait(state).unwrap();
        }
        state.buf.len() >= bytes
    }
}

struct PipeReader {
    pipe: Arc<Pipe>,
    pos: u64,
}

impl Read for PipeReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut state = self.pipe.state.lock().unwrap();
        while state.buf.is_empty() && !state.done {
            state = self.pipe.changed.wait(state).unwrap();
        }
        if state.buf.is_empty() {
            if let Some(e) = state.error.take() {
                return Err(e);
            }
        }
        let was_full = state.buf.len() >= PIPE_CAP;
        let n = state.buf.read(out)?;
        if was_full {
            self.pipe.changed.notify_all();
        }
        self.pos += n as u64;
        Ok(n)
    }
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        self.pipe.state.lock().unwrap().closed = true;
        self.pipe.changed.notify_all();
    }
}

impl Seek for PipeReader {
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
