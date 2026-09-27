use super::THUMB_PX;
use crate::api;
use crate::message::{ExportMsg, FavoritesMsg, HistoryMsg, Msg, NewsMsg, RatingsMsg};
use crate::state::{ExportState, Plaza};
use iced::Task;
use std::collections::HashSet;

pub fn load_history(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.history.pager.start_loading(page);
    Task::perform(api::fetch_history(page), |r| {
        Msg::History(HistoryMsg::Loaded(r))
    })
}

pub fn load_ratings(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.ratings.pager.start_loading(page);
    Task::perform(api::fetch_ratings(state.ratings.range, page), |r| {
        Msg::Ratings(RatingsMsg::Loaded(r))
    })
}

pub fn load_news(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.news.pager.start_loading(page);
    Task::perform(api::fetch_news(page), |r| Msg::News(NewsMsg::Loaded(r)))
}

pub fn load_favorites(state: &mut Plaza, page: u32) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    state.favorites.removed.clear();
    state.favorites.pager.start_loading(page);
    Task::perform(api::fetch_favorites(token, page), |r| {
        Msg::Favorites(FavoritesMsg::Loaded(r))
    })
}

pub fn history(state: &mut Plaza, msg: HistoryMsg) -> Task<Msg> {
    match msg {
        HistoryMsg::Loaded(Ok(response)) => {
            state.history.list = response.data;
            state.history.pager.finish_loading(&response.meta);
            state.history.date_range = response.date_range;
            Task::none()
        }
        HistoryMsg::Loaded(Err(e)) => {
            state.history.pager.loading = false;
            state.alert = Some(e.to_string());
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
        RatingsMsg::Loaded(Ok(response)) => {
            state.ratings.list = response.data;
            state.ratings.pager.finish_loading(&response.meta);
            Task::none()
        }
        RatingsMsg::Loaded(Err(e)) => {
            state.ratings.pager.loading = false;
            state.alert = Some(e.to_string());
            Task::none()
        }
        RatingsMsg::Page(msg) => match state.ratings.pager.apply(msg) {
            Some(page) => load_ratings(state, page),
            None => Task::none(),
        },
        RatingsMsg::Range(range) => {
            state.ratings.range = range;
            state.ratings.list.clear();
            load_ratings(state, 1)
        }
    }
}

pub fn news(state: &mut Plaza, msg: NewsMsg) -> Task<Msg> {
    match msg {
        NewsMsg::Loaded(Ok(response)) => {
            state.news.list = response.data.into_iter().map(Into::into).collect();
            state.news.pager.finish_loading(&response.meta);
            Task::none()
        }
        NewsMsg::Loaded(Err(e)) => {
            state.news.pager.loading = false;
            state.alert = Some(e.to_string());
            Task::none()
        }
        NewsMsg::Page(msg) => match state.news.pager.apply(msg) {
            Some(page) => load_news(state, page),
            None => Task::none(),
        },
    }
}

pub fn favorites(state: &mut Plaza, msg: FavoritesMsg) -> Task<Msg> {
    let favorites = &mut state.favorites;
    match msg {
        FavoritesMsg::Loaded(Ok(response)) => {
            favorites.pager.finish_loading(&response.meta);

            let wanted: HashSet<&str> = response
                .data
                .iter()
                .filter_map(|f| f.song.thumb_url())
                .collect();
            favorites
                .artwork
                .retain(|url, _| wanted.contains(url.as_str()));
            let fetches: Vec<_> = wanted
                .into_iter()
                .filter(|url| !favorites.artwork.contains_key(*url))
                .map(|url| {
                    let url = url.to_owned();
                    Task::perform(api::fetch_artwork(url.clone(), THUMB_PX), move |r| {
                        Msg::Favorites(FavoritesMsg::Artwork(url, r))
                    })
                })
                .collect();

            favorites.list = response.data;
            Task::batch(fetches)
        }
        FavoritesMsg::Loaded(Err(e)) => {
            favorites.pager.loading = false;
            state.alert = Some(e.to_string());
            Task::none()
        }
        FavoritesMsg::Artwork(url, Ok(handle)) => {
            favorites.artwork.insert(url, handle);
            Task::none()
        }
        FavoritesMsg::Artwork(url, Err(e)) => {
            eprintln!("Thumbnail download failed for {url}: {e}");
            Task::none()
        }
        FavoritesMsg::Page(msg) => match favorites.pager.apply(msg) {
            Some(page) => load_favorites(state, page),
            None => Task::none(),
        },
        FavoritesMsg::Remove(id) => {
            let Some(token) = state.token() else {
                return Task::none();
            };
            Task::perform(api::delete_favorite(token, id), move |r| {
                Msg::Favorites(FavoritesMsg::Removed(id, r))
            })
        }
        FavoritesMsg::Removed(id, Ok(())) => {
            if !favorites.removed.contains(&id) {
                favorites.removed.push(id);
            }
            Task::none()
        }
        FavoritesMsg::Removed(_, Err(e)) => {
            state.alert = Some(e.to_string());
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
            state.export = ExportState {
                loading: true,
                ..ExportState::default()
            };
            Task::perform(api::export_favorites(token), |r| {
                Msg::Export(ExportMsg::Done(r))
            })
        }
        ExportMsg::Done(result) => {
            state.export.loading = false;
            match result {
                Ok(link) => state.export.link = Some(link),
                Err(e) => state.export.error = Some(e.to_string()),
            }
            Task::none()
        }
    }
}
