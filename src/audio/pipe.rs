use super::{lock, wait, wait_timeout};
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub const CAPACITY: usize = 512 * 1024;
const CANCEL_POLL: Duration = Duration::from_millis(100);

#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    buf: VecDeque<u8>,
    finished: bool,
    error: Option<io::Error>,
    reader_gone: bool,
}

pub struct PipeWriter {
    shared: Arc<Shared>,
}

pub struct PipeReader {
    shared: Arc<Shared>,
    position: u64,
}

pub fn pipe() -> (PipeWriter, PipeReader) {
    let shared = Arc::new(Shared::default());
    let writer = PipeWriter {
        shared: Arc::clone(&shared),
    };
    let reader = PipeReader {
        shared,
        position: 0,
    };
    (writer, reader)
}

impl PipeWriter {
    pub fn fill_from(self, mut source: impl Read) {
        let mut scratch = [0u8; 16 * 1024];
        let error = loop {
            let n = match source.read(&mut scratch) {
                Ok(0) => break None,
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("Audio stream read error: {e}");
                    break Some(e);
                }
            };
            let mut state = lock(&self.shared.state);
            while state.buf.len() >= CAPACITY && !state.reader_gone {
                state = wait(&self.shared.changed, state);
            }
            if state.reader_gone {
                return;
            }
            state.buf.extend(&scratch[..n]);
            self.shared.changed.notify_all();
        };
        let mut state = lock(&self.shared.state);
        state.finished = true;
        state.error = error;
        self.shared.changed.notify_all();
    }
}

impl PipeReader {
    pub fn wait_for(&self, bytes: usize, cancelled: impl Fn() -> bool) -> bool {
        let mut state = lock(&self.shared.state);
        while state.buf.len() < bytes && !state.finished {
            if cancelled() {
                return false;
            }
            state = wait_timeout(&self.shared.changed, state, CANCEL_POLL);
        }
        state.buf.len() >= bytes
    }
}

impl Read for PipeReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut state = lock(&self.shared.state);
        while state.buf.is_empty() && !state.finished {
            state = wait(&self.shared.changed, state);
        }
        if state.buf.is_empty()
            && let Some(e) = state.error.take()
        {
            return Err(e);
        }
        let was_full = state.buf.len() >= CAPACITY;
        let n = state.buf.read(out)?;
        if was_full {
            self.shared.changed.notify_all();
        }
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for PipeReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let unsupported = |msg| io::Error::new(io::ErrorKind::Unsupported, msg);
        let target = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(offset) => self
                .position
                .checked_add_signed(offset)
                .ok_or_else(|| unsupported("cannot seek before the start of a stream"))?,
            SeekFrom::End(_) => {
                return Err(unsupported("cannot seek from the end of a live stream"));
            }
        };
        if target < self.position {
            return Err(unsupported("cannot seek backward in a live stream"));
        }
        let skip = target - self.position;
        io::copy(&mut self.by_ref().take(skip), &mut io::sink())?;
        Ok(self.position)
    }
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        lock(&self.shared.state).reader_gone = true;
        self.shared.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Instant;

    #[test]
    fn wait_for_notices_a_cancel_that_arrives_while_waiting() {
        let (_writer, reader) = pipe();
        let started = Instant::now();
        assert!(!reader.wait_for(1, || started.elapsed() >= CANCEL_POLL));
    }

    #[test]
    fn wait_for_succeeds_at_capacity() {
        let (writer, reader) = pipe();
        let source = Cursor::new(vec![0u8; CAPACITY * 2]);
        let producer = std::thread::spawn(move || writer.fill_from(source));
        assert!(reader.wait_for(CAPACITY, || false));
        drop(reader);
        producer
            .join()
            .expect("producer exits once the reader is gone");
    }
}
