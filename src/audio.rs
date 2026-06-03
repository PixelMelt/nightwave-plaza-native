use crate::api::StatusSong;
use futures::channel::mpsc::UnboundedSender;
use rodio::{buffer::SamplesBuffer, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

const STREAM_URL: &str = "https://radio.plaza.one/mp3";

const PREBUFFER: Duration = Duration::from_secs(5);
const CHUNK: Duration = Duration::from_millis(250);

const DEFAULT_VOLUME: f32 = 0.5;
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

pub struct AudioPlayer {
    _stream: MixerDeviceSink,
    player: Arc<Player>,
    muted: Arc<AtomicBool>,
    streaming: Arc<AtomicBool>,
    target_volume: Mutex<f32>,
    controls: Mutex<Option<MediaControls>>,
    progress: Mutex<Option<Duration>>,
    length: Mutex<Option<Duration>>,
}

impl AudioPlayer {
    pub fn new(
        media_tx: Option<UnboundedSender<MediaControlEvent>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(stream.mixer());
        player.set_volume(DEFAULT_VOLUME);

        #[cfg(not(target_os = "windows"))]
        let controls = media_tx.and_then(|tx| build_controls(tx).ok());
        #[cfg(target_os = "windows")]
        let controls: Option<MediaControls> = {
            let _ = media_tx;
            None
        };

        let this = Self {
            _stream: stream,
            player: Arc::new(player),
            muted: Arc::new(AtomicBool::new(false)),
            streaming: Arc::new(AtomicBool::new(false)),
            target_volume: Mutex::new(DEFAULT_VOLUME),
            controls: Mutex::new(controls),
            progress: Mutex::new(None),
            length: Mutex::new(None),
        };

        let player = this.player.clone();
        let streaming = this.streaming.clone();
        std::thread::spawn(move || stream_forever(player, streaming));

        this.emit_playback();

        Ok(this)
    }

    pub fn is_playing(&self) -> bool {
        !self.muted.load(Ordering::Relaxed)
    }

    pub fn is_streaming(&self) -> bool {
        self.streaming.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, vol: f32) {
        let v = vol.clamp(0.0, 1.0);
        if let Ok(mut g) = self.target_volume.lock() {
            *g = v;
        }
        if !self.muted.load(Ordering::Relaxed) {
            self.player.set_volume(v);
        }
    }

    pub fn play(&self) {
        if !self.muted.swap(false, Ordering::Relaxed) {
            return;
        }
        let v = self
            .target_volume
            .lock()
            .map(|g| *g)
            .unwrap_or(DEFAULT_VOLUME);
        self.player.set_volume(v);
        self.emit_playback();
    }

    pub fn stop(&self) {
        if self.muted.swap(true, Ordering::Relaxed) {
            return;
        }
        self.player.set_volume(0.0);
        self.emit_playback();
    }

    pub fn update_metadata(&self, song: &StatusSong) {
        let length = (song.length > 0.0).then(|| Duration::from_secs_f64(song.length));
        let position =
            (song.position >= 0.0).then(|| Duration::from_secs_f64(song.position.max(0.0)));

        if let Ok(mut g) = self.length.lock() {
            *g = length;
        }
        if let Ok(mut g) = self.progress.lock() {
            *g = position;
        }

        if let Ok(mut guard) = self.controls.lock() {
            if let Some(controls) = guard.as_mut() {
                let _ = controls.set_metadata(MediaMetadata {
                    title: opt_str(&song.title),
                    album: opt_str(&song.album),
                    artist: opt_str(&song.artist),
                    cover_url: song.artwork_src.as_deref(),
                    duration: length,
                });
            }
        }

        self.emit_playback();
    }

    fn emit_playback(&self) {
        let progress = self
            .progress
            .lock()
            .ok()
            .and_then(|g| *g)
            .map(MediaPosition);
        let playback = if self.muted.load(Ordering::Relaxed) {
            MediaPlayback::Paused { progress }
        } else {
            MediaPlayback::Playing { progress }
        };
        if let Ok(mut guard) = self.controls.lock() {
            if let Some(controls) = guard.as_mut() {
                let _ = controls.set_playback(playback);
            }
        }
    }
}

fn opt_str(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(not(target_os = "windows"))]
fn build_controls(
    tx: UnboundedSender<MediaControlEvent>,
) -> Result<MediaControls, souvlaki::Error> {
    let config = souvlaki::PlatformConfig {
        dbus_name: "nightwave_plaza",
        display_name: "Nightwave Plaza",
        hwnd: None,
    };
    let mut controls = MediaControls::new(config)?;
    controls.attach(move |event| {
        let _ = tx.unbounded_send(event);
    })?;
    Ok(controls)
}

fn stream_forever(player: Arc<Player>, streaming: Arc<AtomicBool>) {
    loop {
        if let Err(e) = stream_once(&player, &streaming) {
            eprintln!("Audio stream error: {e}");
        }
        streaming.store(false, Ordering::Relaxed);
        std::thread::sleep(RECONNECT_DELAY);
    }
}

fn stream_once(
    player: &Player,
    streaming: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "{}?nocache={}",
        STREAM_URL,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let resp = crate::net::agent().get(&url).call()?;

    let decoder = Decoder::new_mp3(StreamReader::new(resp.into_reader()))?;

    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    let per_sec = sample_rate.get() as usize * channels.get() as usize;
    let prebuffer_len = per_sec * PREBUFFER.as_millis() as usize / 1000;
    let chunk_len = (per_sec * CHUNK.as_millis() as usize / 1000).max(1);

    let mut buf: Vec<f32> = Vec::with_capacity(prebuffer_len.max(chunk_len));
    let mut prebuffered = false;

    for sample in decoder {
        buf.push(sample);
        let threshold = if prebuffered {
            chunk_len
        } else {
            prebuffer_len
        };
        if buf.len() >= threshold {
            let chunk = std::mem::replace(&mut buf, Vec::with_capacity(chunk_len));
            player.append(SamplesBuffer::new(channels, sample_rate, chunk));
            if !prebuffered {
                prebuffered = true;
                streaming.store(true, Ordering::Relaxed);
            }
        }
    }

    if !buf.is_empty() {
        player.append(SamplesBuffer::new(channels, sample_rate, buf));
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
        if target == self.pos {
            return Ok(self.pos);
        }
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
