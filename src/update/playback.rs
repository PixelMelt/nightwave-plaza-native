use super::COVER_PX;
use crate::api::{self, Status};
use crate::lastfm::{self, Scrobble};
use crate::message::Msg;
use crate::state::{Notice, Plaza, SongReaction};
use iced::Task;
use souvlaki::MediaControlEvent;
use std::time::Instant;

pub fn fetch_status() -> Task<Msg> {
    Task::perform(api::fetch_status(), Msg::Status)
}

pub fn status(state: &mut Plaza, result: api::Result<Status>) -> Task<Msg> {
    let status = match result {
        Ok(status) => status,
        Err(e) => {
            state.alert = Some(e.to_string());
            return Task::none();
        }
    };
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
    let playing = state.player.is_playing();
    if let Some(scrobble) = &mut state.scrobble {
        scrobble.set_playing(playing, now);
    }

    let new_artwork = status
        .song
        .artwork_src
        .as_deref()
        .filter(|url| !url.is_empty() && *url != state.artwork_url)
        .map(str::to_owned);
    state.status = status;
    state.status_at = now;
    state.media.set_song(&state.status.song);
    if song_changed {
        tasks.push(announce_now_playing(state));
    }
    sync_discord(state);

    if let Some(url) = new_artwork {
        state.artwork_url.clone_from(&url);
        tasks.push(Task::perform(
            api::fetch_artwork(url, COVER_PX),
            Msg::Artwork,
        ));
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

pub fn sync_integrations(state: &mut Plaza) {
    let playing = state.player.is_playing();
    if let Some(scrobble) = &mut state.scrobble {
        scrobble.set_playing(playing, Instant::now());
    }
    sync_discord(state);
}

pub fn set_volume(state: &mut Plaza, percent: f32) {
    state.volume = percent;
    state.player.set_volume(percent / 100.0);
    state.notice = Some(Notice::new(format!("Volume: {}%", percent as u32)));
}

pub fn react(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        state.alert =
            Some("Please sign in to your Nightwave Plaza account to access this feature.".into());
        return Task::none();
    };
    let song_id = state.status.song.id.clone();
    if song_id.is_empty() {
        return Task::none();
    }
    let reaction = SongReaction {
        reaction: state.reaction.for_song(&song_id).next(),
        song_id,
    };
    Task::perform(api::react(token, reaction.reaction), move |result| {
        Msg::Reacted(reaction, result)
    })
}

pub fn reacted(state: &mut Plaza, reaction: SongReaction, result: api::Result<u32>) {
    match result {
        Ok(count) => {
            if state.status.song.id == reaction.song_id {
                state.status.song.reactions = count;
            }
            state.reaction = reaction;
        }
        Err(e) => state.alert = Some(e.to_string()),
    }
}

pub fn sync_discord(state: &Plaza) {
    let song = &state.status.song;
    if state.config.discord.enabled && state.player.is_playing() && !song.title.is_empty() {
        state.discord.set(song);
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
    Task::future(async move {
        if let Err(e) = lastfm::update_now_playing(session_key, song).await {
            eprintln!("Last.fm now playing failed: {e}");
        }
    })
    .discard()
}

fn scrobble(state: &Plaza, scrobble: &Scrobble, now: Instant) -> Task<Msg> {
    let Some(session_key) = state.config.lastfm.active_session_key() else {
        return Task::none();
    };
    let Some(started_at) = scrobble.due(now) else {
        return Task::none();
    };
    let (session_key, song) = (session_key.to_owned(), scrobble.song.clone());
    Task::future(async move {
        if let Err(e) = lastfm::scrobble(session_key, song, started_at).await {
            eprintln!("Last.fm scrobble failed: {e}");
        }
    })
    .discard()
}
