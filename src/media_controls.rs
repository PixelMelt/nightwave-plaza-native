use crate::api::StatusSong;
use crate::message::Msg;
use futures::channel::mpsc::UnboundedSender;
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use std::time::Duration;

pub struct MediaSession {
    controls: Option<MediaControls>,
    position: Option<Duration>,
    playing: bool,
}

impl MediaSession {
    pub fn new(events: UnboundedSender<Msg>) -> Self {
        let controls = attach(events)
            .map_err(|e| eprintln!("Media controls unavailable: {e:?}"))
            .ok();
        Self {
            controls,
            position: None,
            playing: false,
        }
    }

    pub fn set_song(&mut self, song: &StatusSong) {
        self.position = Some(Duration::from_secs_f64(song.position));
        if let Some(controls) = &mut self.controls {
            let metadata = MediaMetadata {
                title: non_empty(&song.title),
                album: non_empty(&song.album),
                artist: non_empty(&song.artist),
                cover_url: song.artwork_src.as_deref(),
                duration: (song.length > 0.0).then(|| Duration::from_secs_f64(song.length)),
            };
            if let Err(e) = controls.set_metadata(metadata) {
                eprintln!("Failed to publish media metadata: {e:?}");
            }
        }
        self.publish_playback();
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
        self.publish_playback();
    }

    fn publish_playback(&mut self) {
        let Some(controls) = &mut self.controls else {
            return;
        };
        let progress = self.position.map(MediaPosition);
        let playback = if self.playing {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };
        if let Err(e) = controls.set_playback(playback) {
            eprintln!("Failed to publish playback state: {e:?}");
        }
    }
}

fn non_empty(s: &str) -> Option<&str> {
    Some(s).filter(|s| !s.is_empty())
}

#[cfg(not(target_os = "windows"))]
fn attach(events: UnboundedSender<Msg>) -> Result<MediaControls, souvlaki::Error> {
    let config = souvlaki::PlatformConfig {
        dbus_name: "nightwave_plaza",
        display_name: "Nightwave Plaza",
        hwnd: None,
    };
    let mut controls = MediaControls::new(config)?;
    controls.attach(move |event| {
        let _ = events.unbounded_send(Msg::Media(event));
    })?;
    Ok(controls)
}

#[cfg(target_os = "windows")]
fn attach(_events: UnboundedSender<Msg>) -> Result<MediaControls, &'static str> {
    Err("not supported on Windows")
}
