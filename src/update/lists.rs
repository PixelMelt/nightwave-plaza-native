use super::{prefetch, show_error};
use crate::api;
use crate::message::{ExportMsg, FavoritesMsg, HistoryMsg, Msg, NewsMsg, RatingsMsg};
use crate::state::{Ahead, ArtworkSize, ExportState, Plaza, RatingsPage};
use crate::window::WindowKind;
use iced::Task;
use std::collections::HashMap;

pub fn load_history(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.history.pager.start_loading(page);
    match state.history.ahead.take(&page) {
        Ahead::Ready(response) => {
            show_history(state, response);
            Task::none()
        }
        Ahead::Pending => Task::none(),
        Ahead::Missing => fetch_history(page),
    }
}

pub fn prefetch_history(state: &mut Plaza, page: u32) -> Task<Msg> {
    let history = &mut state.history;
    if history.pager.holds(page) || !history.ahead.request(page) {
        return Task::none();
    }
    fetch_history(page)
}

fn fetch_history(page: u32) -> Task<Msg> {
    Task::perform(api::fetch_history(page), move |r| {
        Msg::History(HistoryMsg::Loaded(page, r))
    })
}

fn show_history(state: &mut Plaza, response: api::HistoryResponse) {
    let history = &mut state.history;
    history.list = response.data;
    history.pager.finish_loading(&response.meta);
    history.date_range = response.date_range;
}

pub fn load_ratings(state: &mut Plaza, page: u32) -> Task<Msg> {
    let key = RatingsPage {
        range: state.ratings.range,
        page,
    };
    state.ratings.pager.start_loading(page);
    match state.ratings.ahead.take(&key) {
        Ahead::Ready(response) => {
            show_ratings(state, response);
            Task::none()
        }
        Ahead::Pending => Task::none(),
        Ahead::Missing => fetch_ratings(key),
    }
}

pub fn prefetch_ratings(state: &mut Plaza, key: RatingsPage) -> Task<Msg> {
    let ratings = &mut state.ratings;
    let shown = key.range == ratings.range && ratings.pager.holds(key.page);
    if shown || !ratings.ahead.request(key) {
        return Task::none();
    }
    fetch_ratings(key)
}

fn fetch_ratings(key: RatingsPage) -> Task<Msg> {
    Task::perform(api::fetch_ratings(key.range, key.page), move |r| {
        Msg::Ratings(RatingsMsg::Loaded(key, r))
    })
}

fn show_ratings(state: &mut Plaza, response: api::Page<api::RatingEntry>) {
    state.ratings.list = response.data;
    state.ratings.pager.finish_loading(&response.meta);
}

pub fn load_news(state: &mut Plaza, page: u32) -> Task<Msg> {
    state.news.pager.start_loading(page);
    match state.news.ahead.take(&page) {
        Ahead::Ready(response) => {
            show_news(state, response);
            Task::none()
        }
        Ahead::Pending => Task::none(),
        Ahead::Missing => fetch_news(page),
    }
}

pub fn prefetch_news(state: &mut Plaza, page: u32) -> Task<Msg> {
    let news = &mut state.news;
    if news.pager.holds(page) || !news.ahead.request(page) {
        return Task::none();
    }
    fetch_news(page)
}

fn fetch_news(page: u32) -> Task<Msg> {
    Task::perform(api::fetch_news(page), move |r| {
        Msg::News(NewsMsg::Loaded(page, r))
    })
}

fn show_news(state: &mut Plaza, response: api::Page<api::NewsArticle>) {
    state.news.list = response.data.into_iter().map(Into::into).collect();
    state.news.pager.finish_loading(&response.meta);
}

pub fn load_favorites(state: &mut Plaza, page: u32) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    state.favorites.removed.clear();
    state.favorites.pager.start_loading(page);
    match state.favorites.ahead.take(&page) {
        Ahead::Ready(response) => show_favorites(state, response),
        Ahead::Pending => Task::none(),
        Ahead::Missing => fetch_favorites(token, page),
    }
}

pub fn prefetch_favorites(state: &mut Plaza, page: u32) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    let shown =
        state.window_of(WindowKind::UserFavorites).is_some() && state.favorites.pager.holds(page);
    if shown || !state.favorites.ahead.request(page) {
        return Task::none();
    }
    fetch_favorites(token, page)
}

fn fetch_favorites(token: String, page: u32) -> Task<Msg> {
    Task::perform(api::fetch_favorites(token, page), move |r| {
        Msg::Favorites(FavoritesMsg::Loaded(page, r))
    })
}

fn show_favorites(state: &mut Plaza, response: api::Page<api::FavoriteEntry>) -> Task<Msg> {
    state.favorites.pager.finish_loading(&response.meta);
    let mut artwork = HashMap::new();
    let mut fetches = Vec::new();
    for url in response.data.iter().filter_map(|f| f.song.thumb_url()) {
        match state.cache.thumbs.get(url) {
            Some(handle) => {
                artwork.insert(url.to_owned(), handle);
            }
            None => fetches.push(prefetch::artwork(state, ArtworkSize::Thumb, url.to_owned())),
        }
    }
    state.favorites.artwork = artwork;
    state.favorites.list = response.data;
    Task::batch(fetches)
}

fn prefetch_thumbs(state: &mut Plaza, urls: Vec<String>) -> Task<Msg> {
    Task::batch(
        urls.into_iter()
            .map(|url| prefetch::artwork(state, ArtworkSize::Thumb, url))
            .collect::<Vec<_>>(),
    )
}

pub fn history(state: &mut Plaza, msg: HistoryMsg) -> Task<Msg> {
    match msg {
        HistoryMsg::Loaded(page, result) if !state.history.pager.awaits(page) => {
            state.history.ahead.fill(&page, result);
            Task::none()
        }
        HistoryMsg::Loaded(_, Ok(response)) => {
            show_history(state, response);
            Task::none()
        }
        HistoryMsg::Loaded(_, Err(e)) => {
            state.history.pager.loading = false;
            show_error(state, e.to_string())
        }
        HistoryMsg::Page(msg) => match state.history.pager.apply(msg) {
            Some(page) => load_history(state, page),
            None => Task::none(),
        },
    }
}

pub fn ratings(state: &mut Plaza, msg: RatingsMsg) -> Task<Msg> {
    match msg {
        RatingsMsg::Loaded(key, result)
            if key.range != state.ratings.range || !state.ratings.pager.awaits(key.page) =>
        {
            state.ratings.ahead.fill(&key, result);
            Task::none()
        }
        RatingsMsg::Loaded(_, Ok(response)) => {
            show_ratings(state, response);
            Task::none()
        }
        RatingsMsg::Loaded(_, Err(e)) => {
            state.ratings.pager.loading = false;
            show_error(state, e.to_string())
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
        NewsMsg::Loaded(page, result) if !state.news.pager.awaits(page) => {
            state.news.ahead.fill(&page, result);
            Task::none()
        }
        NewsMsg::Loaded(_, Ok(response)) => {
            show_news(state, response);
            Task::none()
        }
        NewsMsg::Loaded(_, Err(e)) => {
            state.news.pager.loading = false;
            show_error(state, e.to_string())
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
        FavoritesMsg::Loaded(page, result) if !favorites.pager.awaits(page) => {
            let thumbs = favorites
                .ahead
                .fill(&page, result)
                .map(|response| {
                    response
                        .data
                        .iter()
                        .filter_map(|f| f.song.thumb_url().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            prefetch_thumbs(state, thumbs)
        }
        FavoritesMsg::Loaded(_, Ok(response)) => show_favorites(state, response),
        FavoritesMsg::Loaded(_, Err(e)) => {
            favorites.pager.loading = false;
            show_error(state, e.to_string())
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
            favorites.ahead.clear();
            state.stats_at = None;
            if state.song_info.favorite_id == Some(id) {
                state.song_info.favorite_id = None;
            }
            Task::none()
        }
        FavoritesMsg::Removed(_, Err(e)) => show_error(state, e.to_string()),
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
