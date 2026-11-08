use super::account::authed;
use super::{prefetch, show_error, show_info};
use crate::api::{self, Status};
use crate::lastfm::{self, Scrobble};
use crate::message::{LastfmMsg, Msg};
use crate::platform;
use crate::state::{ArtworkSize, Notice, Plaza, SongReaction};
use iced::{Task, window};
use souvlaki::MediaControlEvent;
use std::time::{Duration, Instant};

const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

pub fn fetch_status() -> Task<Msg> {
    Task::perform(api::fetch_status(), Msg::Status)
}

pub fn refresh(state: &mut Plaza) -> Task<Msg> {
    platform::trim_heap();
    if state.status_poll.loading {
        return Task::none();
    }
    state.status_poll.loading = true;
    fetch_status()
}

pub fn status(state: &mut Plaza, result: api::Result<Status>) -> Task<Msg> {
    state.status_poll.loading = false;
    let status = match result {
        Ok(status) => status,
        Err(e) => {
            let first_failure = !state.status_poll.failed;
            state.status_poll.failed = true;
            return if first_failure {
                show_error(state, e.to_string())
            } else {
                Task::none()
            };
        }
    };
    state.status_poll.failed = false;
    let now = Instant::now();
    let song_changed = state
        .scrobble
        .as_ref()
        .is_none_or(|s| s.song.id != status.song.id);
    let mut tasks = Vec::new();

    if song_changed {
        if let Some(previous) = state.scrobble.take() {
            tasks.push(scrobble(state, &previous, now));
        }
        state.scrobble = Some(Scrobble::new(status.song.clone()));
    }
    let listening = state.player.is_listening();
    if let Some(scrobble) = &mut state.scrobble {
        scrobble.set_playing(listening, now);
    }

    let artwork_url = status.song.artwork_src.clone().unwrap_or_default();
    state.status = status;
    state.status_at = now;
    state.media.set_song(&state.status.song);
    if song_changed {
        tasks.push(announce_now_playing(state));
    }
    sync_discord(state);

    if artwork_url != state.artwork_url {
        state.artwork_url.clone_from(&artwork_url);
        if artwork_url.is_empty() {
            state.artwork = None;
        } else if let Some(handle) = state.cache.covers.get(&artwork_url) {
            state.artwork = Some(handle);
        } else {
            tasks.push(prefetch::artwork(state, ArtworkSize::Cover, artwork_url));
        }
    }
    Task::batch(tasks)
}

pub fn tick(state: &mut Plaza) -> Task<Msg> {
    let now = Instant::now();
    if state.notice.as_ref().is_some_and(|n| now >= n.until) {
        state.notice = None;
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

pub fn media(state: &mut Plaza, event: &MediaControlEvent) -> Task<Msg> {
    match event {
        MediaControlEvent::Toggle => toggle(state),
        MediaControlEvent::Play => set_playing(state, true),
        MediaControlEvent::Pause | MediaControlEvent::Stop => set_playing(state, false),
        _ => Task::none(),
    }
}

pub fn set_playing(state: &mut Plaza, playing: bool) -> Task<Msg> {
    let started = playing && !state.player.is_playing();
    if playing {
        state.player.play();
    } else {
        state.player.stop();
    }
    state.media.set_playing(playing);
    sync_integrations(state);
    if started {
        announce_now_playing(state)
    } else {
        Task::none()
    }
}

pub fn sync_scrobble(state: &mut Plaza) {
    let listening = state.player.is_listening();
    if let Some(scrobble) = &mut state.scrobble {
        scrobble.set_playing(listening, Instant::now());
    }
}

pub fn sync_integrations(state: &mut Plaza) {
    sync_scrobble(state);
    sync_discord(state);
}

pub fn shutdown(state: &mut Plaza) -> Task<Msg> {
    std::thread::spawn(|| {
        std::thread::sleep(SHUTDOWN_DEADLINE);
        std::process::exit(0);
    });
    state.player.stop();
    let submit = match state.scrobble.take() {
        Some(pending) => scrobble(state, &pending, Instant::now()),
        None => Task::none(),
    };
    let close_windows = Task::batch(state.windows.keys().copied().map(window::close));
    Task::batch([close_windows, submit]).chain(Task::done(Msg::Exit))
}

pub fn set_volume(state: &mut Plaza, percent: f32) {
    state.volume = percent;
    state.player.set_volume(percent / 100.0);
    state.notice = Some(Notice::new(format!("Volume: {}%", percent as u32)));
}

pub fn react(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        return show_info(
            state,
            "Please sign in to your Nightwave Plaza account to access this feature.",
        );
    };
    let song_id = state.status.song.id.clone();
    if song_id.is_empty() || state.reaction_pending {
        return Task::none();
    }
    let reaction = SongReaction {
        reaction: state.reaction.for_song(&song_id).next(),
        song_id,
    };
    state.reaction_pending = true;
    let requested = reaction.reaction;
    authed(
        token,
        move |token| api::react(token, requested),
        move |result| Msg::Reacted(reaction, result),
    )
}

pub fn reacted(state: &mut Plaza, reaction: SongReaction, result: api::Result<u32>) -> Task<Msg> {
    state.reaction_pending = false;
    match result {
        Ok(count) => {
            if state.status.song.id == reaction.song_id {
                state.status.song.reactions = count;
            }
            state.reaction = reaction;
            state.stats_at = None;
            Task::none()
        }
        Err(e) => show_error(state, e.to_string()),
    }
}

pub fn sync_discord(state: &Plaza) {
    let song = &state.status.song;
    if state.config.discord.enabled && state.player.is_playing() && !song.title.is_empty() {
        state.discord.set(song, state.song_position());
    } else {
        state.discord.clear();
    }
}

pub fn announce_now_playing(state: &Plaza) -> Task<Msg> {
    let song = &state.status.song;
    let Some(session_key) = state.config.lastfm.active_session_key() else {
        return Task::none();
    };
    if !state.player.is_playing() || !song.has_metadata() {
        return Task::none();
    }
    let (session_key, song) = (session_key.to_owned(), song.clone());
    let used_key = session_key.clone();
    Task::perform(lastfm::update_now_playing(session_key, song), move |r| {
        Msg::Lastfm(LastfmMsg::Submitted(used_key, r))
    })
}

fn scrobble(state: &Plaza, scrobble: &Scrobble, now: Instant) -> Task<Msg> {
    let Some(session_key) = state.config.lastfm.active_session_key() else {
        return Task::none();
    };
    let Some(started_at) = scrobble.due(now) else {
        return Task::none();
    };
    let (session_key, song) = (session_key.to_owned(), scrobble.song.clone());
    let used_key = session_key.clone();
    Task::perform(lastfm::scrobble(session_key, song, started_at), move |r| {
        Msg::Lastfm(LastfmMsg::Submitted(used_key, r))
    })
}
