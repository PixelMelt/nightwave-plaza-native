use super::{account, lists, song_info};
use crate::api;
use crate::message::{Msg, Prefetch, PrefetchMsg, SongPreview};
use crate::state::{ArtworkSize, HoverIntent, Plaza, RatingsPage};
use crate::window::WindowKind;
use iced::Task;
use iced::widget::image;
use std::time::{Duration, Instant};

pub const HOVER_POLL: Duration = Duration::from_millis(30);
const HOVER_DELAY: Duration = Duration::from_millis(60);

pub fn update(state: &mut Plaza, msg: PrefetchMsg) -> Task<Msg> {
    match msg {
        PrefetchMsg::Hover(target) => {
            state.hover = Some(HoverIntent {
                target,
                since: Instant::now(),
            });
            Task::none()
        }
        PrefetchMsg::Settle => match state.hover.take() {
            Some(intent) if intent.since.elapsed() >= HOVER_DELAY => prefetch(state, intent.target),
            intent => {
                state.hover = intent;
                Task::none()
            }
        },
        PrefetchMsg::Song(song_id, result) => song_info::loaded(state, song_id, result),
        PrefetchMsg::Artwork(size, url, result) => {
            artwork_loaded(state, size, url, result);
            Task::none()
        }
    }
}

pub fn artwork(state: &mut Plaza, size: ArtworkSize, url: String) -> Task<Msg> {
    if !state.cache.artwork(size).claim(&url) {
        return Task::none();
    }
    Task::perform(api::fetch_artwork(url.clone(), size.px()), move |r| {
        Msg::Prefetch(PrefetchMsg::Artwork(size, url, r))
    })
}

fn prefetch(state: &mut Plaza, target: Prefetch) -> Task<Msg> {
    match target {
        Prefetch::Song(preview) => song(state, preview),
        Prefetch::Window(kind) => window(state, kind),
        Prefetch::History(page) => lists::prefetch_history(state, page),
        Prefetch::Ratings(key) => lists::prefetch_ratings(state, key),
        Prefetch::News(page) => lists::prefetch_news(state, page),
        Prefetch::Favorites(page) => lists::prefetch_favorites(state, page),
    }
}

fn song(state: &mut Plaza, preview: SongPreview) -> Task<Msg> {
    let cover = match preview.artwork.filter(|url| !url.is_empty()) {
        Some(url) => artwork(state, ArtworkSize::Cover, url),
        None => Task::none(),
    };
    Task::batch([song_info::request(state, preview.id), cover])
}

fn window(state: &mut Plaza, kind: WindowKind) -> Task<Msg> {
    match kind {
        WindowKind::History if state.history.list.is_empty() => lists::prefetch_history(state, 1),
        WindowKind::Ratings if state.ratings.list.is_empty() => {
            let key = RatingsPage {
                range: state.ratings.range,
                page: 1,
            };
            lists::prefetch_ratings(state, key)
        }
        WindowKind::News if state.news.list.is_empty() => lists::prefetch_news(state, 1),
        WindowKind::UserFavorites => lists::prefetch_favorites(state, 1),
        WindowKind::UserProfile => account::refresh_stats(state),
        _ => Task::none(),
    }
}

fn artwork_loaded(
    state: &mut Plaza,
    size: ArtworkSize,
    url: String,
    result: api::Result<image::Handle>,
) {
    let handle = match result {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Artwork download failed for {url}: {e}");
            state.cache.artwork(size).fail(&url);
            if size == ArtworkSize::Cover && url == state.artwork_url {
                state.artwork = None;
            }
            return;
        }
    };
    match size {
        ArtworkSize::Cover => {
            if url == state.artwork_url {
                state.artwork = Some(handle.clone());
            }
            let info = &mut state.song_info;
            if info.data.as_ref().and_then(|song| song.data.artwork_url()) == Some(url.as_str()) {
                info.artwork = Some(handle.clone());
            }
        }
        ArtworkSize::Thumb => {
            let favorites = &mut state.favorites;
            if favorites
                .list
                .iter()
                .any(|entry| entry.song.thumb_url() == Some(url.as_str()))
            {
                favorites.artwork.insert(url.clone(), handle.clone());
            }
        }
    }
    state.cache.artwork(size).insert(url, handle);
}
