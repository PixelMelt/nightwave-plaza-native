use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::{BufReader, Cursor, Read};
use std::sync::{Arc, Mutex};

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Arc<Sink>,
    playing: Arc<Mutex<bool>>,
    streaming: Arc<Mutex<bool>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        sink.set_volume(0.5);
        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink: Arc::new(sink),
            playing: Arc::new(Mutex::new(false)),
            streaming: Arc::new(Mutex::new(false)),
        })
    }

    pub fn is_playing(&self) -> bool {
        *self.playing.lock().unwrap()
    }

    pub fn is_streaming(&self) -> bool {
        *self.streaming.lock().unwrap()
    }

    #[allow(dead_code)]
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol.clamp(0.0, 1.0));
    }

    pub fn play(&self) {
        if self.is_playing() {
            return;
        }
        *self.playing.lock().unwrap() = true;
        *self.streaming.lock().unwrap() = false;
        let sink = self.sink.clone();
        let playing = self.playing.clone();
        let streaming = self.streaming.clone();

        std::thread::spawn(move || {
            let result = stream_audio(sink, playing.clone(), streaming);
            if let Err(e) = result {
                eprintln!("Audio stream error: {}", e);
            }
            *playing.lock().unwrap() = false;
        });
    }

    pub fn stop(&self) {
        *self.playing.lock().unwrap() = false;
        *self.streaming.lock().unwrap() = false;
        self.sink.stop();
    }
}

fn stream_audio(
    sink: Arc<Sink>,
    playing: Arc<Mutex<bool>>,
    streaming: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://radio.plaza.one/mp3?nocache={}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let mut resp = reqwest::blocking::get(&url)?;

    let mut buffer = Vec::with_capacity(256 * 1024);
    let mut chunk = [0u8; 16384];

    loop {
        if !*playing.lock().unwrap() {
            return Ok(());
        }
        let n = resp.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&chunk[..n]);
        if buffer.len() >= 65536 {
            break;
        }
    }

    let cursor = Cursor::new(buffer.clone());
    match Decoder::new(BufReader::new(cursor)) {
        Ok(source) => {
            sink.append(source);

            *streaming.lock().unwrap() = true;
        }
        Err(e) => {
            eprintln!("Initial decode failed: {}, retrying with more data...", e);
        }
    }

    loop {
        if !*playing.lock().unwrap() {
            break;
        }

        buffer.clear();
        let mut accumulated = 0usize;

        while accumulated < 131072 {
            if !*playing.lock().unwrap() {
                return Ok(());
            }
            let n = resp.read(&mut chunk)?;
            if n == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..n]);
            accumulated += n;
        }

        let cursor = Cursor::new(buffer.clone());
        match Decoder::new(BufReader::new(cursor)) {
            Ok(source) => {
                sink.append(source);
                if !*streaming.lock().unwrap() {
                    *streaming.lock().unwrap() = true;
                }
            }
            Err(e) => {
                eprintln!("Chunk decode error: {}", e);
            }
        }
    }

    Ok(())
}
