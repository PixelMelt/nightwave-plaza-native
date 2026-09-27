use super::{BoxError, Shared};
use rodio::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::cpal::{
    self, BufferSize, DeviceId, FromSample, SampleFormat, SizedSample, StreamConfig,
};
use rodio::{ChannelCount, SampleRate};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

const DEFAULT_DEVICE_POLL: Duration = Duration::from_secs(5);
const DEVICE_BUFFER: Duration = Duration::from_millis(200);

const PREFERRED_CHANNELS: u16 = 2;
const PREFERRED_RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format {
    pub channels: ChannelCount,
    pub sample_rate: SampleRate,
}

#[derive(Default)]
pub struct Output {
    open: Option<OpenOutput>,
}

struct OpenOutput {
    _stream: cpal::Stream,
    format: Format,
    device: Option<DeviceId>,
    checked_at: Instant,
}

impl Output {
    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn ensure_open(&mut self, shared: &Arc<Shared>) -> Result<Format, BoxError> {
        if let Some(open) = &mut self.open {
            let lost = shared.device_lost.swap(false, Ordering::Relaxed);
            if !lost && open.checked_at.elapsed() < DEFAULT_DEVICE_POLL {
                return Ok(open.format);
            }
            if !lost && default_device_id() == open.device {
                open.checked_at = Instant::now();
                return Ok(open.format);
            }
            self.open = None;
            let reopened = open_default(shared)?;
            eprintln!("Audio output reopened on {:?}", reopened.device);
            return Ok(self.open.insert(reopened).format);
        }
        Ok(self.open.insert(open_default(shared)?).format)
    }
}

fn default_device_id() -> Option<DeviceId> {
    cpal::default_host().default_output_device()?.id().ok()
}

fn open_default(shared: &Arc<Shared>) -> Result<OpenOutput, BoxError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no audio output device")?;
    let id = device.id().ok();

    let (stream, format) = open_device(&device, shared).or_else(|err| {
        host.output_devices()
            .ok()
            .and_then(|mut devices| devices.find_map(|d| open_device(&d, shared).ok()))
            .ok_or(err)
    })?;

    Ok(OpenOutput {
        _stream: stream,
        format,
        device: id,
        checked_at: Instant::now(),
    })
}

fn open_device(
    device: &cpal::Device,
    shared: &Arc<Shared>,
) -> Result<(cpal::Stream, Format), BoxError> {
    let default_rate = device.default_output_config().ok().map(|c| c.sample_rate());
    open_stream(device, shared, Some(PREFERRED_RATE))
        .or_else(|_| open_stream(device, shared, default_rate))
        .or_else(|_| open_stream(device, shared, None))
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
        config.channels = PREFERRED_CHANNELS;
        config.buffer_size = BufferSize::Fixed(rate * DEVICE_BUFFER.as_millis() as u32 / 1000);
    }
    let format = Format {
        channels: ChannelCount::new(config.channels).ok_or("device has no channels")?,
        sample_rate: SampleRate::new(config.sample_rate).ok_or("device has no sample rate")?,
    };

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

fn build_stream<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: &StreamConfig,
    shared: &Arc<Shared>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let render = Arc::clone(shared);
    let on_error = Arc::clone(shared);
    device.build_output_stream::<T, _, _>(
        config,
        move |out, _| render.render(out),
        move |e| {
            eprintln!("Audio device error: {e}");
            on_error.device_lost.store(true, Ordering::Relaxed);
        },
        None,
    )
}
