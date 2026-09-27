use super::task;
use crate::api::{self, RatingsRange};
use crate::state::{
    ExportMsg, FavoritesMsg, HistoryMsg, Msg, NewsMsg, Plaza, RatingsMsg, SongInfoMsg, WinType,
};
use iced::Task;
use std::collections::HashSet;

const THUMB_PX: u32 = 128;
const COVER_PX: u32 = 256;

pub fn load_history(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.history.pager.goto(page);
    task(api::fetch_history(page), |r| {
        Msg::History(HistoryMsg::Loaded(r))
    })
}

pub fn load_ratings(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.ratings.pager.goto(page);
    task(api::fetch_ratings(state.ratings.range, page), |r| {
        Msg::Ratings(RatingsMsg::Loaded(r))
    })
}

pub fn load_news(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.news.pager.goto(page);
    task(api::fetch_news(page), |r| Msg::News(NewsMsg::Loaded(r)))
}

pub fn load_favorites(state: &mut Plaza, page: u32) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    state.favorites.deleted.clear();
    state.favorites.pager.goto(page);
    task(api::fetch_favorites(token, page), |r| {
        Msg::Favorites(FavoritesMsg::Loaded(r))
    })
}

pub fn history(state: &mut Plaza, msg: HistoryMsg) -> Task<Msg> {
    match msg {
        HistoryMsg::Loaded(Ok(resp)) => {
            state.history.list = resp.data;
            state.history.pager.loaded(&resp.meta);
            state.history.date_range = resp.date_range;
            Task::none()
        }
        HistoryMsg::Loaded(Err(e)) => {
            state.history.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        HistoryMsg::Page(msg) => match state.history.pager.apply(msg) {
            Some(page) => load_history(state, page),
            None => Task::none(),
        },
    }
}

pub fn ratings(state: &mut Plaza, msg: RatingsMsg) -> Task<Msg> {
    match msg {
        RatingsMsg::Loaded(Ok(resp)) => {
            state.ratings.list = resp.data;
            state.ratings.pager.loaded(&resp.meta);
            Task::none()
        }
        RatingsMsg::Loaded(Err(e)) => {
            state.ratings.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        RatingsMsg::Page(msg) => match state.ratings.pager.apply(msg) {
            Some(page) => load_ratings(state, page),
            None => Task::none(),
        },
        RatingsMsg::Range(range) => set_range(state, range),
    }
}

fn set_range(state: &mut Plaza, range: RatingsRange) -> Task<Msg> {
    state.ratings.range = range;
    state.ratings.list.clear();
    load_ratings(state, 1)
}

pub fn news(state: &mut Plaza, msg: NewsMsg) -> Task<Msg> {
    match msg {
        NewsMsg::Loaded(Ok(resp)) => {
            state.news.list = resp.data.into_iter().map(Into::into).collect();
            state.news.pager.loaded(&resp.meta);
            Task::none()
        }
        NewsMsg::Loaded(Err(e)) => {
            state.news.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        NewsMsg::Page(msg) => match state.news.pager.apply(msg) {
            Some(page) => load_news(state, page),
            None => Task::none(),
        },
    }
}

pub fn favorites(state: &mut Plaza, msg: FavoritesMsg) -> Task<Msg> {
    let favs = &mut state.favorites;
    match msg {
        FavoritesMsg::Loaded(Ok(resp)) => {
            favs.pager.loaded(&resp.meta);

            let wanted: HashSet<&str> = resp
                .data
                .iter()
                .filter_map(|f| f.song.thumb_url())
                .collect();
            favs.artwork.retain(|url, _| wanted.contains(url.as_str()));

            let fetches: Vec<Task<Msg>> = wanted
                .into_iter()
                .filter(|url| !favs.artwork.contains_key(*url))
                .map(|url| {
                    let url = url.to_string();
                    task(api::fetch_artwork(url.clone(), THUMB_PX), move |r| {
                        Msg::Favorites(FavoritesMsg::Artwork(url.clone(), r))
                    })
                })
                .collect();

            favs.list = resp.data;
            Task::batch(fetches)
        }
        FavoritesMsg::Loaded(Err(e)) => {
            favs.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        FavoritesMsg::Artwork(url, Ok(handle)) => {
            favs.artwork.insert(url, handle);
            Task::none()
        }
        FavoritesMsg::Artwork(url, Err(e)) => {
            eprintln!("Thumbnail download failed for {url}: {e}");
            Task::none()
        }
        FavoritesMsg::Page(msg) => match favs.pager.apply(msg) {
            Some(page) => load_favorites(state, page),
            None => Task::none(),
        },
        FavoritesMsg::Delete(id) => {
            let Some(token) = state.token() else {
                return Task::none();
            };
            task(api::delete_favorite(token, id), move |r| {
                Msg::Favorites(FavoritesMsg::Deleted(id, r))
            })
        }
        FavoritesMsg::Deleted(id, Ok(())) => {
            if !favs.deleted.contains(&id) {
                favs.deleted.push(id);
            }
            Task::none()
        }
        FavoritesMsg::Deleted(_, Err(e)) => {
            state.error_msg = Some(e);
            Task::none()
        }
    }
}

pub fn export(state: &mut Plaza, msg: ExportMsg) -> Task<Msg> {
    match msg {
        ExportMsg::Start => {
            let Some(token) = state.token() else {
                return Task::none();
            };
            state.export = crate::state::ExportState {
                loading: true,
                ..Default::default()
            };
            task(api::export_favorites(token), |r| {
                Msg::Export(ExportMsg::Done(r))
            })
        }
        ExportMsg::Done(result) => {
            state.export.loading = false;
            match result {
                Ok(link) => state.export.link = Some(link),
                Err(e) => state.export.error = Some(e),
            }
            Task::none()
        }
    }
}

pub fn song_info(state: &mut Plaza, msg: SongInfoMsg) -> Task<Msg> {
    let info = &mut state.song_info;
    match msg {
        SongInfoMsg::Open(song_id) => {
            *info = Default::default();
            Task::batch([
                super::open(state, WinType::SongInfo),
                task(api::fetch_song(song_id), |r| {
                    Msg::SongInfo(SongInfoMsg::Loaded(r))
                }),
            ])
        }
        SongInfoMsg::Loaded(Ok(resp)) => {
            let artwork = resp.data.artwork_src.clone().filter(|url| !url.is_empty());
            info.data = Some(resp);
            match artwork {
                Some(url) => task(api::fetch_artwork(url, COVER_PX), |r| {
                    Msg::SongInfo(SongInfoMsg::Artwork(r))
                }),
                None => Task::none(),
            }
        }
        SongInfoMsg::Loaded(Err(e)) => {
            info.error = Some(e);
            Task::none()
        }
        SongInfoMsg::Artwork(result) => {
            info.artwork = super::playback::artwork(result);
            Task::none()
        }
        SongInfoMsg::ToggleFavorite => {
            let Some(token) = state.token() else {
                state.error_msg = Some("Please sign in to favorite songs.".into());
                return Task::none();
            };
            let info = &mut state.song_info;
            if info.fav_sending {
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
            info.fav_sending = true;
            match info.favorite_id {
                Some(fav_id) => task(api::delete_favorite(token, fav_id), |r| {
                    Msg::SongInfo(SongInfoMsg::FavoriteRemoved(r))
                }),
                None => task(api::add_favorite(token, song_id), |r| {
                    Msg::SongInfo(SongInfoMsg::FavoriteAdded(r))
                }),
            }
        }
        SongInfoMsg::FavoriteAdded(result) => {
            info.fav_sending = false;
            match result {
                Ok(id) => info.favorite_id = Some(id),
                Err(e) => state.error_msg = Some(e),
            }
            Task::none()
        }
        SongInfoMsg::FavoriteRemoved(result) => {
            info.fav_sending = false;
            match result {
                Ok(()) => info.favorite_id = None,
                Err(e) => state.error_msg = Some(e),
            }
            Task::none()
        }
    }
}
