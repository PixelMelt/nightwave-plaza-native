mod output;
mod pipe;

use crate::message::Msg;
use crate::net;
use chrono::Utc;
use futures::channel::mpsc::UnboundedSender;
use output::{Format, Output};
use rodio::cpal::{FromSample, SizedSample};
use rodio::source::UniformSourceIterator;
use rodio::{Decoder, Source};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

const STREAM_URL: &str = "https://radio.plaza.one/mp3";

const PREBUFFER: Duration = Duration::from_secs(5);
const DEFAULT_KBPS: usize = 128;
const CHUNK: Duration = Duration::from_millis(250);
const CHUNKS_AHEAD: usize = 3;
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct AudioPlayer {
    shared: Arc<Shared>,
}

struct Shared {
    samples: Mutex<VecDeque<f32>>,
    volume: AtomicU32,
    playing: AtomicBool,
    streaming: AtomicBool,
    device_lost: AtomicBool,
    resume: Condvar,
    resume_lock: Mutex<()>,
    events: UnboundedSender<Msg>,
}

impl AudioPlayer {
    pub fn spawn(events: UnboundedSender<Msg>) -> Self {
        let shared = Arc::new(Shared {
            samples: Mutex::new(VecDeque::new()),
            volume: AtomicU32::new(1.0f32.to_bits()),
            playing: AtomicBool::new(true),
            streaming: AtomicBool::new(false),
            device_lost: AtomicBool::new(false),
            resume: Condvar::new(),
            resume_lock: Mutex::new(()),
            events,
        });
        let thread_shared = Arc::clone(&shared);
        std::thread::spawn(move || run(&thread_shared));
        Self { shared }
    }

    pub fn is_playing(&self) -> bool {
        self.shared.is_playing()
    }

    pub fn is_streaming(&self) -> bool {
        self.shared.streaming.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) {
        self.shared
            .volume
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn play(&self) {
        let _guard = lock(&self.shared.resume_lock);
        if !self.shared.playing.swap(true, Ordering::Relaxed) {
            self.shared.resume.notify_all();
        }
    }

    pub fn stop(&self) {
        if self.shared.playing.swap(false, Ordering::Relaxed) {
            self.shared.clear();
        }
    }
}

impl Shared {
    fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    fn set_streaming(&self, streaming: bool) {
        if self.streaming.swap(streaming, Ordering::Relaxed) != streaming {
            let _ = self.events.unbounded_send(Msg::StreamChanged);
        }
    }

    fn queued(&self) -> usize {
        lock(&self.samples).len()
    }

    fn clear(&self) {
        lock(&self.samples).clear();
    }

    fn render<T: SizedSample + FromSample<f32>>(&self, out: &mut [T]) {
        let volume = f32::from_bits(self.volume.load(Ordering::Relaxed));
        let mut samples = lock(&self.samples);
        let n = out.len().min(samples.len());
        for (slot, sample) in out.iter_mut().zip(samples.drain(..n)) {
            *slot = T::from_sample(sample * volume);
        }
        drop(samples);
        out[n..].fill(T::EQUILIBRIUM);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn wait<'a, T>(condvar: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    condvar.wait(guard).unwrap_or_else(PoisonError::into_inner)
}

fn run(shared: &Arc<Shared>) {
    let mut output = Output::default();
    loop {
        {
            let mut guard = lock(&shared.resume_lock);
            if !shared.is_playing() {
                output.close();
                shared.clear();
            }
            while !shared.is_playing() {
                guard = wait(&shared.resume, guard);
            }
        }
        if let Err(e) = stream(shared, &mut output) {
            eprintln!("Audio stream error: {e}");
        }
        shared.set_streaming(false);
        if shared.is_playing() {
            std::thread::sleep(RECONNECT_DELAY);
        }
    }
}

fn stream(shared: &Arc<Shared>, output: &mut Output) -> Result<(), BoxError> {
    output.ensure_open(shared)?;

    let url = format!("{STREAM_URL}?nocache={}", Utc::now().timestamp_millis());
    let response = net::agent().get(&url).call()?;
    let kbps = response
        .header("icy-br")
        .and_then(|br| br.trim().parse().ok())
        .unwrap_or(DEFAULT_KBPS);
    let bytes_per_sec = kbps * 1000 / 8;
    let prebuffer_bytes = bytes_per_sec * PREBUFFER.as_millis() as usize / 1000;

    let (writer, reader) = pipe::pipe();
    let body = response.into_reader();
    std::thread::spawn(move || writer.fill_from(body));
    if !reader.wait_for(prebuffer_bytes) || !shared.is_playing() {
        return Ok(());
    }

    let decoder = Decoder::new_mp3(reader)?;
    let format = output.ensure_open(shared)?;
    if decoder.channels() == format.channels && decoder.sample_rate() == format.sample_rate {
        pump(shared, output, format, decoder)
    } else {
        let resampled = UniformSourceIterator::new(decoder, format.channels, format.sample_rate);
        pump(shared, output, format, resampled)
    }
}

fn pump(
    shared: &Arc<Shared>,
    output: &mut Output,
    format: Format,
    samples: impl Iterator<Item = f32>,
) -> Result<(), BoxError> {
    let frames = (format.sample_rate.get() as usize * CHUNK.as_millis() as usize / 1000).max(1);
    let chunk_len = frames * usize::from(format.channels.get());
    let mut chunk = Vec::with_capacity(chunk_len);

    for sample in samples {
        chunk.push(sample);
        if chunk.len() < chunk_len {
            continue;
        }
        while shared.queued() >= chunk_len * CHUNKS_AHEAD && shared.is_playing() {
            std::thread::sleep(CHUNK);
        }
        if !shared.is_playing() {
            return Ok(());
        }
        if output.ensure_open(shared)? != format {
            shared.clear();
            return Err("audio output format changed".into());
        }
        lock(&shared.samples).extend(chunk.drain(..));
        shared.set_streaming(true);
    }

    Ok(())
}
