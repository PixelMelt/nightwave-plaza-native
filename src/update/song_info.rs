use super::{COVER_PX, artwork_or_log, open_window};
use crate::api;
use crate::message::{Msg, SongInfoMsg};
use crate::state::{Plaza, SongInfoState};
use crate::window::WindowKind;
use iced::Task;

pub fn update(state: &mut Plaza, msg: SongInfoMsg) -> Task<Msg> {
    let info = &mut state.song_info;
    match msg {
        SongInfoMsg::Open(song_id) => {
            *info = SongInfoState::default();
            Task::batch([
                open_window(state, WindowKind::SongInfo),
                Task::perform(api::fetch_song(song_id), |r| {
                    Msg::SongInfo(SongInfoMsg::Loaded(r))
                }),
            ])
        }
        SongInfoMsg::Loaded(Ok(response)) => {
            let artwork = response
                .data
                .artwork_src
                .clone()
                .filter(|url| !url.is_empty());
            info.data = Some(response);
            match artwork {
                Some(url) => Task::perform(api::fetch_artwork(url, COVER_PX), |r| {
                    Msg::SongInfo(SongInfoMsg::Artwork(r))
                }),
                None => Task::none(),
            }
        }
        SongInfoMsg::Loaded(Err(e)) => {
            info.error = Some(e.to_string());
            Task::none()
        }
        SongInfoMsg::Artwork(result) => {
            info.artwork = artwork_or_log(result);
            Task::none()
        }
        SongInfoMsg::ToggleFavorite => toggle_favorite(state),
        SongInfoMsg::FavoriteAdded(result) => {
            info.favorite_pending = false;
            match result {
                Ok(id) => info.favorite_id = Some(id),
                Err(e) => state.alert = Some(e.to_string()),
            }
            Task::none()
        }
        SongInfoMsg::FavoriteRemoved(result) => {
            info.favorite_pending = false;
            match result {
                Ok(()) => info.favorite_id = None,
                Err(e) => state.alert = Some(e.to_string()),
            }
            Task::none()
        }
    }
}

fn toggle_favorite(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        state.alert = Some("Please sign in to favorite songs.".into());
        return Task::none();
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
    match info.favorite_id {
        Some(favorite_id) => Task::perform(api::delete_favorite(token, favorite_id), |r| {
            Msg::SongInfo(SongInfoMsg::FavoriteRemoved(r))
        }),
        None => Task::perform(api::add_favorite(token, song_id), |r| {
            Msg::SongInfo(SongInfoMsg::FavoriteAdded(r))
        }),
    }
}
