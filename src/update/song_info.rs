use super::account::authed;
use super::{open_window, prefetch, show_error, show_info};
use crate::api;
use crate::message::{Msg, PrefetchMsg, SongInfoMsg};
use crate::state::{ArtworkSize, Plaza, SongInfoState};
use crate::window::WindowKind;
use iced::Task;

pub fn update(state: &mut Plaza, msg: SongInfoMsg) -> Task<Msg> {
    let info = &mut state.song_info;
    match msg {
        SongInfoMsg::Open(song_id) => open(state, song_id),
        SongInfoMsg::ToggleFavorite => toggle_favorite(state),
        SongInfoMsg::FavoriteAdded(song_id, result) => {
            let current = song_id == info.song_id;
            if current {
                info.favorite_pending = false;
            }
            match result {
                Ok(id) => {
                    if current {
                        info.favorite_id = Some(id);
                    }
                    forget_favorite_state(state, &song_id);
                    Task::none()
                }
                Err(e) => show_error(state, e.to_string()),
            }
        }
        SongInfoMsg::FavoriteRemoved(song_id, result) => {
            let current = song_id == info.song_id;
            if current {
                info.favorite_pending = false;
            }
            match result {
                Ok(()) => {
                    let favorites = &mut state.favorites;
                    if current
                        && let Some(id) = info.favorite_id.take()
                        && favorites.list.iter().any(|entry| entry.id == id)
                        && !favorites.removed.contains(&id)
                    {
                        favorites.removed.push(id);
                    }
                    forget_favorite_state(state, &song_id);
                    Task::none()
                }
                Err(e) => show_error(state, e.to_string()),
            }
        }
    }
}

fn forget_favorite_state(state: &mut Plaza, song_id: &str) {
    state.favorites.ahead.clear();
    state.stats_at = None;
    state.cache.songs.remove(song_id);
}

fn open(state: &mut Plaza, song_id: String) -> Task<Msg> {
    let favorites = &state.favorites;
    let favorite_id = favorites
        .list
        .iter()
        .find(|entry| entry.song.id == song_id && !favorites.removed.contains(&entry.id))
        .map(|entry| entry.id);
    state.song_info = SongInfoState {
        song_id: song_id.clone(),
        favorite_id,
        ..SongInfoState::default()
    };
    let window = open_window(state, WindowKind::SongInfo);
    let song = match state.cache.songs.get(&song_id) {
        Some(song) => show(state, song),
        None => request(state, song_id),
    };
    Task::batch([window, song])
}

pub fn reload(state: &mut Plaza) -> Task<Msg> {
    let song_id = state.song_info.song_id.clone();
    if song_id.is_empty() || state.window_of(WindowKind::SongInfo).is_none() {
        return Task::none();
    }
    state.cache.songs.remove(&song_id);
    request(state, song_id)
}

pub fn request(state: &mut Plaza, song_id: String) -> Task<Msg> {
    if song_id.is_empty() || !state.cache.songs.claim(&song_id) {
        return Task::none();
    }
    Task::perform(api::fetch_song(song_id.clone(), state.token()), move |r| {
        Msg::Prefetch(PrefetchMsg::Song(song_id, r))
    })
}

pub fn loaded(
    state: &mut Plaza,
    song_id: String,
    result: api::Result<api::SongResponse>,
) -> Task<Msg> {
    let info = &mut state.song_info;
    let current = info.song_id == song_id;
    match result {
        Ok(song) => {
            state.cache.songs.insert(song_id, song.clone());
            if current {
                show(state, song)
            } else {
                Task::none()
            }
        }
        Err(e) => {
            state.cache.songs.fail(&song_id);
            if current && info.data.is_none() {
                info.error = Some(e.to_string());
            } else {
                eprintln!("Song prefetch failed for {song_id}: {e}");
            }
            Task::none()
        }
    }
}

fn show(state: &mut Plaza, song: api::SongResponse) -> Task<Msg> {
    let cover = song.data.artwork_url().map(str::to_owned);
    if let Some(id) = song.current_user.as_ref().and_then(|user| user.favorite_id) {
        state.song_info.favorite_id = Some(id);
    }
    state.song_info.data = Some(song);
    let Some(url) = cover else {
        return Task::none();
    };
    match state.cache.covers.get(&url) {
        Some(handle) => {
            state.song_info.artwork = Some(handle);
            Task::none()
        }
        None => prefetch::artwork(state, ArtworkSize::Cover, url),
    }
}

fn toggle_favorite(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        return show_info(state, "Please sign in to favorite songs.");
    };
    let info = &mut state.song_info;
    if info.favorite_pending {
        return Task::none();
    }
    let Some(song_id) = info
        .data
        .as_ref()
        .map(|d| d.data.id.clone())
        .filter(|id| !id.is_empty())
    else {
        return Task::none();
    };
    info.favorite_pending = true;
    let selected = info.song_id.clone();
    match info.favorite_id {
        Some(favorite_id) => authed(
            token,
            move |token| api::delete_favorite(token, favorite_id),
            move |r| Msg::SongInfo(SongInfoMsg::FavoriteRemoved(selected, r)),
        ),
        None => authed(
            token,
            move |token| api::add_favorite(token, song_id),
            move |r| Msg::SongInfo(SongInfoMsg::FavoriteAdded(selected, r)),
        ),
    }
}
