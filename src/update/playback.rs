use super::task;
use crate::api::{self, Status};
use crate::lastfm::{self, Scrobble};
use crate::state::{Msg, Plaza, Reaction, NOTICE_DURATION};
use iced::widget::image;
use iced::Task;
use souvlaki::MediaControlEvent;
use std::time::Instant;

const COVER_PX: u32 = 256;

pub fn fetch_status() -> Task<Msg> {
    task(api::fetch_status(), Msg::Status)
}

pub fn status(state: &mut Plaza, result: Result<Status, String>) -> Task<Msg> {
    let status = match result {
        Ok(status) => status,
        Err(e) => {
            state.error_msg = Some(e);
            return Task::none();
        }
    };
    let now = Instant::now();
    let playing = state.player.is_playing();
    let song_changed = state
        .scrobble
        .as_ref()
        .is_none_or(|s| s.song.id != status.song.id);
    let mut tasks = Vec::new();

    if song_changed {
        if let Some(old) = state.scrobble.take() {
            tasks.push(scrobble_task(state, &old, now));
        }
        state.scrobble = Some(Scrobble::new(status.song.clone()));
    }
    if let Some(scrobble) = state.scrobble.as_mut() {
        scrobble.set_playing(playing, now);
    }

    let artwork = status
        .song
        .artwork_src
        .as_deref()
        .filter(|url| !url.is_empty() && *url != state.artwork_url)
        .map(str::to_owned);
    state.status = status;
    state.status_at = now;
    state.player.update_metadata(&state.status.song);
    if song_changed && playing {
        tasks.push(now_playing_task(state));
    }
    discord_update(state);

    if let Some(url) = artwork {
        state.artwork_url = url.clone();
        tasks.push(task(api::fetch_artwork(url, COVER_PX), Msg::Artwork));
    }
    Task::batch(tasks)
}

pub fn artwork(result: Result<image::Handle, String>) -> Option<image::Handle> {
    result
        .map_err(|e| eprintln!("Artwork download failed: {e}"))
        .ok()
}

pub fn tick(state: &mut Plaza) -> Task<Msg> {
    let now = Instant::now();
    if state
        .time_notice
        .as_ref()
        .is_some_and(|(_, until)| now >= *until)
    {
        state.time_notice = None;
    }
    if state.timer.until.is_some_and(|until| now >= until) {
        state.timer.until = None;
        return set_playing(state, false);
    }
    Task::none()
}

pub fn toggle(state: &mut Plaza) -> Task<Msg> {
    let playing = !state.player.is_playing();
    set_playing(state, playing)
}

pub fn media(state: &mut Plaza, event: MediaControlEvent) -> Task<Msg> {
    match event {
        MediaControlEvent::Toggle => toggle(state),
        MediaControlEvent::Play => set_playing(state, true),
        MediaControlEvent::Pause | MediaControlEvent::Stop => set_playing(state, false),
        _ => Task::none(),
    }
}

pub fn set_playing(state: &mut Plaza, playing: bool) -> Task<Msg> {
    let was_playing = state.player.is_playing();
    if playing {
        state.player.play();
    } else {
        state.player.stop();
    }
    sync_playback(state, playing && !was_playing)
}

pub fn sync_playback(state: &mut Plaza, started: bool) -> Task<Msg> {
    let playing = state.player.is_playing();
    if let Some(scrobble) = state.scrobble.as_mut() {
        scrobble.set_playing(playing, Instant::now());
    }
    discord_update(state);
    if started {
        now_playing_task(state)
    } else {
        Task::none()
    }
}

pub fn volume(state: &mut Plaza, volume: f32) -> Task<Msg> {
    state.volume = volume;
    state.player.set_volume(volume / 100.0);
    state.time_notice = Some((
        format!("Volume: {}%", volume as u32),
        Instant::now() + NOTICE_DURATION,
    ));
    Task::none()
}

pub fn react(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        state.error_msg =
            Some("Please sign in to your Nightwave Plaza account to access this feature.".into());
        return Task::none();
    };
    let song_id = state.status.song.id.clone();
    if song_id.is_empty() {
        return Task::none();
    }
    let reaction = Reaction {
        rate: (state.reaction.rate_for(&song_id) + 1) % 3,
        song_id,
    };
    let rate = reaction.rate;
    task(api::react(token, rate), move |r| {
        Msg::Reacted(reaction.clone(), r)
    })
}

pub fn reacted(state: &mut Plaza, reaction: Reaction, result: Result<u32, String>) -> Task<Msg> {
    match result {
        Ok(count) => {
            if state.status.song.id == reaction.song_id {
                state.status.song.reactions = count;
            }
            state.reaction = reaction;
        }
        Err(e) => state.error_msg = Some(e),
    }
    Task::none()
}

pub fn discord_update(state: &Plaza) {
    let song = &state.status.song;
    if state.config.discord.enabled && state.player.is_playing() && !song.title.is_empty() {
        state.discord.set(song);
    } else {
        state.discord.clear();
    }
}

fn now_playing_task(state: &Plaza) -> Task<Msg> {
    let song = &state.status.song;
    let Some(sk) = state.config.lastfm.active_session_key() else {
        return Task::none();
    };
    if !lastfm::has_metadata(song) {
        return Task::none();
    }
    let (sk, song) = (sk.to_owned(), song.clone());
    Task::future(async move {
        if let Err(e) = lastfm::update_now_playing(sk, song).await {
            eprintln!("Last.fm now playing failed: {e}");
        }
    })
    .discard()
}

fn scrobble_task(state: &Plaza, scrobble: &Scrobble, now: Instant) -> Task<Msg> {
    let Some(sk) = state.config.lastfm.active_session_key() else {
        return Task::none();
    };
    let Some(start) = scrobble.due(now) else {
        return Task::none();
    };
    let (sk, song) = (sk.to_owned(), scrobble.song.clone());
    Task::future(async move {
        if let Err(e) = lastfm::scrobble(sk, song, start).await {
            eprintln!("Last.fm scrobble failed: {e}");
        }
    })
    .discard()
}
