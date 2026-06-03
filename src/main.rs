#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod api;
mod audio;
mod config;
mod discord;
mod lastfm;
mod net;
mod paths;
mod state;
mod theme;
mod views;

use audio::AudioPlayer;
use futures::channel::mpsc as futures_mpsc;
use iced::{Element, Fill, Font, Size, Subscription, Task, Theme};
use souvlaki::MediaControlEvent;
use state::{Msg, Plaza, WinType};
use std::sync::Mutex;

static MEDIA_RX: Mutex<Option<futures_mpsc::UnboundedReceiver<MediaControlEvent>>> =
    Mutex::new(None);

const APP_ICON: &[u8] = include_bytes!("assets/icons/favicon-32x32.png");
const TAHOMA: &[u8] = include_bytes!("assets/fonts/subset-Tahoma.ttf");
const TAHOMA_BOLD: &[u8] = include_bytes!("assets/fonts/subset-Tahoma-Bold.ttf");
const ICONS_FONT: &[u8] = include_bytes!("assets/fonts/icons.ttf");

fn app_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(APP_ICON, Some(::image::ImageFormat::Png)).ok()
}

fn dev_mode() -> bool {
    static FLAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FLAG.get_or_init(|| std::env::var("NIGHTWAVE_DEV").is_ok())
}

#[cfg(target_os = "linux")]
fn platform_specific() -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific {
        application_id: "nightwave-plaza".into(),
        ..Default::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn platform_specific() -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific::default()
}

fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();

    if let Err(e) = result {
        eprintln!("Failed to open URL {url}: {e}");
    }
}
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn lastfm_now_playing_task(state: &Plaza) -> Task<Msg> {
    let Some(sk) = state
        .config
        .lastfm
        .is_active()
        .then(|| state.config.lastfm.session_key.clone())
        .flatten()
    else {
        return Task::none();
    };
    let song = &state.status.song;
    if song.artist.is_empty() || song.title.is_empty() {
        return Task::none();
    }
    let (artist, title, album) = (song.artist.clone(), song.title.clone(), song.album.clone());
    Task::perform(
        async move { lastfm::update_now_playing(&sk, &artist, &title, &album).await },
        |_| Msg::Noop,
    )
}

fn lastfm_begin_current(state: &mut Plaza) -> Task<Msg> {
    let is_playing = state.player.as_ref().map_or(false, |p| p.is_playing());
    if !is_playing {
        return Task::none();
    }
    if let Some(s) = state.scrobble.as_mut() {
        s.resume(Instant::now());
    }
    lastfm_now_playing_task(state)
}

fn lastfm_scrobble_task(
    state: &Plaza,
    track: &crate::state::ScrobbleTrack,
    now: Instant,
) -> Task<Msg> {
    let Some(sk) = state
        .config
        .lastfm
        .is_active()
        .then(|| state.config.lastfm.session_key.clone())
        .flatten()
    else {
        return Task::none();
    };
    let Some(start_unix) = track.start_unix else {
        return Task::none();
    };
    if track.artist.is_empty() || track.title.is_empty() {
        return Task::none();
    }
    if track.duration > 0.0 && track.duration < 30.0 {
        return Task::none();
    }
    let played = track.total_played(now);
    let threshold = if track.duration > 0.0 {
        (track.duration / 2.0).min(240.0)
    } else {
        30.0
    };
    if played < threshold {
        return Task::none();
    }
    let (artist, title, album) = (
        track.artist.clone(),
        track.title.clone(),
        track.album.clone(),
    );
    Task::perform(
        async move { lastfm::scrobble(&sk, &artist, &title, &album, start_unix).await },
        |_| Msg::Noop,
    )
}

fn discord_update(state: &Plaza) {
    let Some(handle) = state.discord_presence.as_ref() else {
        return;
    };
    let is_playing = state.player.as_ref().map_or(false, |p| p.is_playing());
    let song = &state.status.song;
    if !state.config.discord.is_active() || !is_playing || song.title.is_empty() {
        handle.clear();
        return;
    }
    let now = now_unix() as i64;
    let start = now - song.position.max(0.0) as i64;
    let end = (song.length > 0.0).then(|| start + song.length as i64);
    handle.set(crate::discord::Presence {
        title: song.title.clone(),
        artist: song.artist.clone(),
        album: song.album.clone(),
        cover_url: song.artwork_src.clone().filter(|s| !s.is_empty()),
        start_unix: start,
        end_unix: end,
    });
}

fn main() -> iced::Result {
    let base = iced::daemon(boot, update, win_view)
        .title(win_title)
        .subscription(subscription)
        .theme(win_theme);

    base.font(TAHOMA)
        .font(TAHOMA_BOLD)
        .font(ICONS_FONT)
        .default_font(Font {
            family: iced::font::Family::Name("Tahoma"),
            ..Font::DEFAULT
        })
        .run()
}

fn win_theme(_state: &Plaza, _window: iced::window::Id) -> Theme {
    Theme::custom(
        "Win98".to_string(),
        iced::theme::Palette {
            background: theme::BG_GRAY,
            text: theme::BLACK,
            primary: theme::TITLE_BLUE,
            success: iced::Color::from_rgb(0.0, 0.5, 0.0),
            warning: iced::Color::from_rgb(0.8, 0.5, 0.0),
            danger: iced::Color::from_rgb(0.8, 0.0, 0.0),
        },
    )
}

fn boot() -> (Plaza, Task<Msg>) {
    let (media_tx, media_rx) = futures_mpsc::unbounded::<MediaControlEvent>();
    *MEDIA_RX.lock().unwrap() = Some(media_rx);
    let player = AudioPlayer::new(Some(media_tx)).ok().map(Arc::new);
    let (main_id, open_task) = iced::window::open(iced::window::Settings {
        size: Size::new(450.0, 218.0),
        resizable: dev_mode(),
        decorations: dev_mode(),
        icon: app_icon(),
        platform_specific: platform_specific(),
        ..Default::default()
    });
    let mut state = Plaza {
        main_window: main_id,
        child_windows: HashMap::new(),
        status: api::Status::default(),
        player,
        volume: 50.0,
        artwork_handle: None,
        artwork_url: String::new(),
        history: crate::state::HistoryState {
            page: 1,
            page_input: "1".into(),
            ..Default::default()
        },
        ratings: crate::state::RatingsState {
            page: 1,
            page_input: "1".into(),
            range: "overtime".into(),
            ..Default::default()
        },
        song_info: crate::state::SongInfoState::default(),
        login: crate::state::LoginState::default(),
        register: crate::state::RegisterState::default(),
        news: crate::state::NewsState {
            page: 1,
            ..Default::default()
        },
        favorites: crate::state::FavoritesState {
            page: 1,
            page_input: "1".into(),
            ..Default::default()
        },
        export: crate::state::ExportState::default(),
        profile_edit: crate::state::ProfileEditState::default(),
        password: crate::state::PasswordState::default(),
        delete: crate::state::DeleteState::default(),
        timer: crate::state::TimerState::default(),
        elapsed: 0.0,
        last_tick: Instant::now(),
        main_focused: true,
        error_msg: None,
        welcome_until: Some(Instant::now() + Duration::from_secs(2)),
        volume_text: None,
        volume_text_until: None,

        auth_token: None,
        user: None,
        user_stats: None,
        stats_loading: false,
        reaction_rate: 0,
        reaction_song_id: String::new(),

        config: config::load(),
        lastfm_token: None,
        lastfm_busy: false,
        lastfm_status: None,
        scrobble: None,

        discord_presence: discord::DiscordHandle::spawn(),
    };

    let session_task = if let Some(saved) = state.config.session.clone() {
        let token = saved.token.clone();
        let token2 = saved.token.clone();

        state.auth_token = Some(saved.token);
        state.user = Some(saved.user);
        Task::perform(async move { api::get_me(&token).await }, move |r| match r {
            Ok(user) => Msg::SessionRestored(user, token2.clone()),
            Err(_) => Msg::LogoutOk,
        })
    } else {
        Task::none()
    };

    (
        state,
        Task::batch([
            open_task.discard(),
            Task::perform(api::fetch_status(), |r| match r {
                Ok(s) => Msg::StatusOk(s),
                Err(e) => Msg::StatusErr(e.to_string()),
            }),
            session_task,
        ]),
    )
}

fn win_title(state: &Plaza, wid: iced::window::Id) -> String {
    if wid == state.main_window {
        let s = &state.status.song;
        if s.artist.is_empty() {
            "Nightwave Plaza".into()
        } else {
            format!("{} - {} - Nightwave Plaza", s.artist, s.title)
        }
    } else {
        match state.child_windows.get(&wid) {
            Some(wt) => wt.title().to_string(),
            None => "Nightwave Plaza".into(),
        }
    }
}

fn win_view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let (inner, title, show_close) = if wid == state.main_window {
        (
            views::player::view(state),
            "Nightwave Plaza".to_string(),
            true,
        )
    } else {
        let inner = match state.child_windows.get(&wid) {
            Some(WinType::History) => views::history::view(state, wid),
            Some(WinType::About) => views::about::view(wid),
            Some(WinType::Support) => views::support::view(wid),
            Some(WinType::Ratings) => views::ratings::view(state, wid),
            Some(WinType::SongInfo) => views::song_info::view(state, wid),
            Some(WinType::UserLogin) => views::user_login::view(state, wid),
            Some(WinType::UserProfile) => views::user_profile::view(state, wid),
            Some(WinType::UserRegister) => views::user_register::view(state, wid),
            Some(WinType::Credits) => views::credits::view(wid),
            Some(WinType::News) => views::news::view(state, wid),
            Some(WinType::UserFavorites) => views::user_favorites::view(state, wid),
            Some(WinType::UserFavoritesExport) => views::user_favorites_export::view(state, wid),
            Some(WinType::UserProfileEdit) => views::user_profile_edit::view(state, wid),
            Some(WinType::UserPassword) => views::user_password::view(state, wid),
            Some(WinType::UserProfileDelete) => views::user_profile_delete::view(state, wid),
            Some(WinType::PlayerTimer) => views::player_timer::view(state, wid),
            Some(WinType::Settings) => views::settings::view(state, wid),
            None => iced::widget::text("").into(),
        };
        let title = match state.child_windows.get(&wid) {
            Some(wt) => wt.title().to_string(),
            None => "Nightwave Plaza".into(),
        };
        (inner, title, true)
    };

    let wt = if wid == state.main_window {
        None
    } else {
        state.child_windows.get(&wid)
    };
    let title_bar = views::widgets::title_bar(title, wid, wt, true, show_close);

    let framed = iced::widget::column![title_bar, inner]
        .spacing(1)
        .padding(1)
        .width(Fill)
        .height(Fill);

    let window_inner = iced::widget::container(framed)
        .padding(2)
        .width(Fill)
        .height(Fill)
        .style(theme::window_box);

    views::widgets::d3_raised_window(window_inner)
        .width(Fill)
        .height(Fill)
        .into()
}

fn subscription(state: &Plaza) -> Subscription<Msg> {
    let is_playing = state.player.as_ref().map_or(false, |p| p.is_playing());

    let display_active = state.main_focused
        && (is_playing || state.welcome_until.is_some() || state.volume_text_until.is_some());

    let timer_armed = state.timer.until.is_some();

    let tick_period = if display_active {
        Some(Duration::from_millis(500))
    } else if timer_armed {
        Some(Duration::from_secs(1))
    } else {
        None
    };

    let refresh_period = if is_playing {
        Duration::from_secs(5)
    } else {
        Duration::from_secs(30)
    };

    let mut subs = vec![
        iced::time::every(refresh_period).map(|_| Msg::Refresh),
        iced::window::close_events().map(Msg::WinClosed),
        iced::window::resize_events().map(|(id, size)| Msg::WinResized(id, size)),
        iced::event::listen_with(|event, status, id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                ..
            }) if status == iced::event::Status::Ignored => Some(Msg::SpaceToggle(id)),
            iced::Event::Window(iced::window::Event::Focused) => Some(Msg::WinFocus(id, true)),
            iced::Event::Window(iced::window::Event::Unfocused) => Some(Msg::WinFocus(id, false)),
            _ => None,
        }),
        Subscription::run(media_event_stream),
    ];

    if let Some(period) = tick_period {
        subs.push(iced::time::every(period).map(|_| Msg::Tick(Instant::now())));
    }

    Subscription::batch(subs)
}

fn media_event_stream() -> impl futures::Stream<Item = Msg> {
    iced::stream::channel(
        64,
        |mut output: futures::channel::mpsc::Sender<Msg>| async move {
            use futures::SinkExt;
            use futures::StreamExt;

            let Some(mut rx) = MEDIA_RX.lock().ok().and_then(|mut g| g.take()) else {
                return;
            };
            while let Some(event) = rx.next().await {
                let _ = output.send(Msg::Media(event)).await;
            }
        },
    )
}

fn update(state: &mut Plaza, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::StatusOk(status) => {
            let new_art = status.song.artwork_src.clone().unwrap_or_default();
            let need = !new_art.is_empty() && new_art != state.artwork_url;

            let is_playing = state.player.as_ref().map_or(false, |p| p.is_playing());
            let now = Instant::now();
            let new_id = status.song.id.clone();
            let prev = state.scrobble.take();
            let song_changed = prev.as_ref().map_or(true, |p| p.song_id != new_id);
            let mut lf_tasks: Vec<Task<Msg>> = Vec::new();
            if song_changed {
                if let Some(ref old) = prev {
                    lf_tasks.push(lastfm_scrobble_task(state, old, now));
                }
                let mut track = crate::state::ScrobbleTrack::new(&status.song);
                if is_playing {
                    track.resume(now);
                }
                state.scrobble = Some(track);
            } else if let Some(mut same) = prev {
                if is_playing {
                    same.resume(now);
                } else {
                    same.pause(now);
                }
                state.scrobble = Some(same);
            }

            state.status = status;
            state.elapsed = state.status.song.position;
            state.last_tick = Instant::now();
            if let Some(ref p) = state.player {
                p.update_metadata(&state.status.song);
            }
            if song_changed && is_playing {
                lf_tasks.push(lastfm_now_playing_task(state));
            }
            discord_update(state);

            let art_task = if need {
                state.artwork_url = new_art.clone();
                Task::perform(
                    async move {
                        api::fetch_artwork(&new_art)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |r| match r {
                        Ok(b) => Msg::ArtworkOk(b),
                        Err(_) => Msg::ArtworkErr,
                    },
                )
            } else {
                Task::none()
            };

            lf_tasks.push(art_task);
            Task::batch(lf_tasks)
        }
        Msg::StatusErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::Tick(now) => {
            let dt = now.duration_since(state.last_tick).as_secs_f64().min(1.5);
            state.last_tick = now;
            state.elapsed += dt;
            if let Some(until) = state.welcome_until {
                if now >= until {
                    state.welcome_until = None;
                }
            }
            if let Some(until) = state.volume_text_until {
                if now >= until {
                    state.volume_text = None;
                    state.volume_text_until = None;
                }
            }
            if let Some(until) = state.timer.until {
                if now >= until {
                    state.timer.until = None;
                    if let Some(ref p) = state.player {
                        p.stop();
                    }
                }
            }
            Task::none()
        }
        Msg::TogglePlay => {
            let now = Instant::now();
            let was_playing = state.player.as_ref().map_or(false, |p| p.is_playing());
            if let Some(ref p) = state.player {
                if was_playing {
                    p.stop();
                } else {
                    p.play();
                }
            }
            let playing_now = state.player.as_ref().map_or(false, |p| p.is_playing());
            if let Some(s) = state.scrobble.as_mut() {
                if playing_now {
                    s.resume(now);
                } else {
                    s.pause(now);
                }
            }
            discord_update(state);
            if playing_now {
                return lastfm_now_playing_task(state);
            }
            Task::none()
        }
        Msg::Volume(v) => {
            state.volume = v;
            if let Some(ref p) = state.player {
                p.set_volume(v / 100.0);
            }
            state.volume_text = Some(format!("Volume: {}%", v as u32));
            state.volume_text_until = Some(Instant::now() + Duration::from_secs(2));
            state.welcome_until = None;
            Task::none()
        }
        Msg::ArtworkOk(b) => {
            state.artwork_handle = Some(iced::widget::image::Handle::from_bytes(b));
            Task::none()
        }
        Msg::ArtworkErr => {
            state.artwork_handle = None;
            Task::none()
        }

        Msg::OpenWin(wt) => {
            for (&id, &t) in &state.child_windows {
                if t == wt {
                    return iced::window::gain_focus(id);
                }
            }
            let (id, task) = iced::window::open(iced::window::Settings {
                size: wt.size(),
                resizable: dev_mode()
                    || matches!(
                        wt,
                        WinType::History
                            | WinType::Ratings
                            | WinType::News
                            | WinType::UserFavorites
                    ),
                decorations: dev_mode(),
                icon: app_icon(),
                ..Default::default()
            });
            state.child_windows.insert(id, wt);
            let mut tasks = vec![task.discard()];
            match wt {
                WinType::History if state.history.list.is_empty() => {
                    state.history.loading = true;
                    tasks.push(Task::perform(api::fetch_history(1), |r| match r {
                        Ok(h) => Msg::History(crate::state::HistoryMsg::Ok(
                            h.data,
                            h.meta.last_page,
                            h.meta.total,
                            h.date_range,
                        )),
                        Err(e) => Msg::History(crate::state::HistoryMsg::Err(e.to_string())),
                    }));
                }
                WinType::Ratings if state.ratings.list.is_empty() => {
                    state.ratings.loading = true;
                    let range = state.ratings.range.clone();
                    tasks.push(Task::perform(
                        async move { api::fetch_ratings(&range, 1).await },
                        |r| match r {
                            Ok(h) => Msg::Ratings(crate::state::RatingsMsg::Ok(
                                h.data,
                                h.meta.last_page,
                                h.meta.total,
                            )),
                            Err(e) => Msg::Ratings(crate::state::RatingsMsg::Err(e.to_string())),
                        },
                    ));
                }
                WinType::UserProfile => {
                    if let Some(ref token) = state.auth_token {
                        state.stats_loading = true;
                        let token = token.clone();
                        tasks.push(Task::perform(
                            async move { api::get_stats(&token).await },
                            |r| match r {
                                Ok(s) => Msg::StatsOk(s.data),
                                Err(e) => Msg::StatsErr(e),
                            },
                        ));
                    }
                }
                WinType::News if state.news.list.is_empty() => {
                    state.news.loading = true;
                    tasks.push(Task::perform(api::fetch_news(1), |r| match r {
                        Ok(n) => Msg::News(crate::state::NewsMsg::Ok(n.data, n.meta.last_page)),
                        Err(e) => Msg::News(crate::state::NewsMsg::Err(e.to_string())),
                    }));
                }
                WinType::UserFavorites => {
                    state.favorites.deleted.clear();
                    state.favorites.page = 1;
                    state.favorites.page_input = "1".into();
                    if let Some(ref token) = state.auth_token {
                        state.favorites.loading = true;
                        let token = token.clone();
                        tasks.push(Task::perform(
                            async move { api::fetch_favorites(&token, 1).await },
                            |r| match r {
                                Ok(f) => Msg::Favorites(crate::state::FavoritesMsg::Ok(
                                    f.data,
                                    f.meta.last_page,
                                    f.meta.total,
                                )),
                                Err(e) => Msg::Favorites(crate::state::FavoritesMsg::Err(e)),
                            },
                        ));
                    }
                }
                WinType::UserFavoritesExport => {
                    state.export = crate::state::ExportState::default();
                }
                WinType::UserProfileEdit => {
                    if let Some(ref u) = state.user {
                        state.profile_edit.username = u.username.clone();
                        state.profile_edit.email = u.email.clone();
                    }
                    state.profile_edit.current_password.clear();
                    state.profile_edit.error = None;
                }
                WinType::UserPassword => {
                    state.password = crate::state::PasswordState::default();
                }
                WinType::UserProfileDelete => {
                    state.delete = crate::state::DeleteState::default();
                }
                _ => {}
            }
            Task::batch(tasks)
        }
        Msg::CloseWin(id) => {
            if id == state.main_window {
                return iced::exit();
            }
            state.child_windows.remove(&id);
            iced::window::close(id)
        }
        Msg::WinClosed(id) => {
            if id == state.main_window {
                iced::exit()
            } else {
                state.child_windows.remove(&id);
                Task::none()
            }
        }

        Msg::History(h_msg) => update_history(state, h_msg),

        Msg::Ratings(r_msg) => update_ratings(state, r_msg),

        Msg::SongInfo(crate::state::SongInfoMsg::Open(song_id)) => {
            for (&id, &t) in &state.child_windows {
                if t == WinType::SongInfo {
                    state.song_info.data = None;
                    state.song_info.loading = true;
                    state.song_info.artwork = None;
                    state.song_info.favorite_id = None;
                    state.song_info.fav_sending = false;
                    return Task::batch([
                        iced::window::gain_focus(id),
                        Task::perform(
                            async move { api::fetch_song(&song_id).await },
                            |r| match r {
                                Ok(s) => Msg::SongInfo(crate::state::SongInfoMsg::Ok(s)),
                                Err(e) => {
                                    Msg::SongInfo(crate::state::SongInfoMsg::Err(e.to_string()))
                                }
                            },
                        ),
                    ]);
                }
            }
            let (id, task) = iced::window::open(iced::window::Settings {
                size: WinType::SongInfo.size(),
                resizable: dev_mode(),
                decorations: dev_mode(),
                icon: app_icon(),
                ..Default::default()
            });
            state.child_windows.insert(id, WinType::SongInfo);
            state.song_info.data = None;
            state.song_info.loading = true;
            state.song_info.artwork = None;
            state.song_info.favorite_id = None;
            state.song_info.fav_sending = false;
            Task::batch([
                task.discard(),
                Task::perform(
                    async move { api::fetch_song(&song_id).await },
                    |r| match r {
                        Ok(s) => Msg::SongInfo(crate::state::SongInfoMsg::Ok(s)),
                        Err(e) => Msg::SongInfo(crate::state::SongInfoMsg::Err(e.to_string())),
                    },
                ),
            ])
        }
        Msg::SongInfo(crate::state::SongInfoMsg::Ok(resp)) => {
            state.song_info.loading = false;
            let art_url = resp.data.artwork_src.clone().unwrap_or_default();
            state.song_info.data = Some(resp);
            if !art_url.is_empty() && Some(&art_url) != Some(&state.artwork_url) {
                Task::perform(
                    async move {
                        api::fetch_artwork(&art_url)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |r| match r {
                        Ok(b) => Msg::SongInfo(crate::state::SongInfoMsg::ArtworkOk(b)),
                        Err(_) => Msg::SongInfo(crate::state::SongInfoMsg::ArtworkErr),
                    },
                )
            } else {
                state.song_info.artwork = state.artwork_handle.clone();
                Task::none()
            }
        }
        Msg::SongInfo(crate::state::SongInfoMsg::Err(e)) => {
            state.song_info.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::SongInfo(crate::state::SongInfoMsg::ArtworkOk(b)) => {
            state.song_info.artwork = Some(iced::widget::image::Handle::from_bytes(b));
            Task::none()
        }
        Msg::SongInfo(crate::state::SongInfoMsg::ArtworkErr) => {
            state.song_info.artwork = None;
            Task::none()
        }
        Msg::SongInfo(crate::state::SongInfoMsg::ToggleFavorite) => {
            let Some(token) = state.auth_token.clone() else {
                state.error_msg = Some("Please sign in to favorite songs.".into());
                return Task::none();
            };
            if state.song_info.fav_sending {
                return Task::none();
            }
            let Some(song_id) = state
                .song_info
                .data
                .as_ref()
                .map(|d| d.data.id.clone())
                .filter(|id| !id.is_empty())
            else {
                return Task::none();
            };

            state.song_info.fav_sending = true;
            match state.song_info.favorite_id {
                Some(fav_id) => Task::perform(
                    async move { api::delete_favorite(&token, fav_id).await },
                    |r| match r {
                        Ok(()) => Msg::SongInfo(crate::state::SongInfoMsg::FavoriteRemoved),
                        Err(e) => Msg::SongInfo(crate::state::SongInfoMsg::FavoriteErr(e)),
                    },
                ),
                None => Task::perform(
                    async move { api::add_favorite(&token, &song_id).await },
                    |r| match r {
                        Ok(id) => Msg::SongInfo(crate::state::SongInfoMsg::FavoriteAdded(id)),
                        Err(e) => Msg::SongInfo(crate::state::SongInfoMsg::FavoriteErr(e)),
                    },
                ),
            }
        }
        Msg::SongInfo(crate::state::SongInfoMsg::FavoriteAdded(id)) => {
            state.song_info.fav_sending = false;
            state.song_info.favorite_id = Some(id);
            Task::none()
        }
        Msg::SongInfo(crate::state::SongInfoMsg::FavoriteRemoved) => {
            state.song_info.fav_sending = false;
            state.song_info.favorite_id = None;
            Task::none()
        }
        Msg::SongInfo(crate::state::SongInfoMsg::FavoriteErr(e)) => {
            state.song_info.fav_sending = false;
            state.error_msg = Some(e);
            Task::none()
        }

        Msg::SessionRestored(user, token) => {
            state.config.session = Some(config::Session {
                token: token.clone(),
                user: user.clone(),
            });
            config::save(&state.config);
            state.auth_token = Some(token);
            state.user = Some(user);
            Task::none()
        }

        Msg::Login(crate::state::LoginMsg::Username(s)) => {
            state.login.username = s;
            Task::none()
        }
        Msg::Login(crate::state::LoginMsg::Password(s)) => {
            state.login.password = s;
            Task::none()
        }
        Msg::Login(crate::state::LoginMsg::Remember(b)) => {
            state.login.remember = b;
            Task::none()
        }
        Msg::Login(crate::state::LoginMsg::Submit) => {
            if state.login.username.is_empty() || state.login.password.is_empty() {
                state.login.error = Some("Please enter a username and password.".into());
                return Task::none();
            }
            state.login.loading = true;
            state.login.error = None;
            let username = state.login.username.clone();
            let password = state.login.password.clone();
            Task::perform(
                async move { api::login(&username, &password).await },
                |r| match r {
                    Ok(resp) => Msg::Login(crate::state::LoginMsg::Ok(resp)),
                    Err(e) => Msg::Login(crate::state::LoginMsg::Err(e)),
                },
            )
        }
        Msg::Login(crate::state::LoginMsg::Ok(resp)) => {
            state.login.loading = false;

            if state.login.remember {
                if let Some(ref token) = resp.token {
                    state.config.session = Some(config::Session {
                        token: token.clone(),
                        user: resp.data.clone(),
                    });
                    config::save(&state.config);
                }
            }
            state.auth_token = resp.token;
            state.user = Some(resp.data);
            state.login.username.clear();
            state.login.password.clear();
            state.login.remember = false;
            state.login.error = None;

            let mut close_task = Task::none();
            for (&id, &t) in &state.child_windows {
                if t == WinType::UserLogin {
                    close_task = iced::window::close(id);
                    break;
                }
            }
            state.child_windows.retain(|_, t| *t != WinType::UserLogin);
            close_task
        }
        Msg::Login(crate::state::LoginMsg::Err(e)) => {
            state.login.loading = false;
            state.login.error = Some(e);
            Task::none()
        }

        Msg::Register(crate::state::RegisterMsg::Username(s)) => {
            state.register.username = s;
            Task::none()
        }
        Msg::Register(crate::state::RegisterMsg::Email(s)) => {
            state.register.email = s;
            Task::none()
        }
        Msg::Register(crate::state::RegisterMsg::Password(s)) => {
            state.register.password = s;
            Task::none()
        }
        Msg::Register(crate::state::RegisterMsg::PasswordRepeat(s)) => {
            state.register.password_repeat = s;
            Task::none()
        }
        Msg::Register(crate::state::RegisterMsg::Submit) => {
            if !state
                .register
                .username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                state.register.error =
                    Some("Username may only contain letters, numbers, and underscores.".into());
                return Task::none();
            }
            if state.register.username.len() < 4 {
                state.register.error = Some("Username is too short.".into());
                return Task::none();
            }
            if state.register.username.len() > 32 {
                state.register.error = Some("Username is too long.".into());
                return Task::none();
            }
            if state.register.password.len() < 3 {
                state.register.error = Some("Password is too short.".into());
                return Task::none();
            }
            if state.register.password != state.register.password_repeat {
                state.register.error = Some("Passwords do not match.".into());
                return Task::none();
            }
            if state.register.email.is_empty() {
                state.register.error = Some("Email is required.".into());
                return Task::none();
            }
            state.register.loading = true;
            state.register.error = None;
            let username = state.register.username.clone();
            let email = state.register.email.clone();
            let password = state.register.password.clone();
            Task::perform(
                async move { api::register(&username, &email, &password).await },
                |r| match r {
                    Ok(user) => Msg::Register(crate::state::RegisterMsg::Ok(user)),
                    Err(e) => Msg::Register(crate::state::RegisterMsg::Err(e)),
                },
            )
        }
        Msg::Register(crate::state::RegisterMsg::Ok(_user)) => {
            state.register.loading = false;
            state.register.error = None;
            state.register.username.clear();
            state.register.email.clear();
            state.register.password.clear();
            state.register.password_repeat.clear();

            let mut close_task = Task::none();
            for (&id, &t) in &state.child_windows {
                if t == WinType::UserRegister {
                    close_task = iced::window::close(id);
                    break;
                }
            }
            state
                .child_windows
                .retain(|_, t| *t != WinType::UserRegister);
            state.error_msg = Some("Registration successful! You can now log in.".into());
            close_task
        }
        Msg::Register(crate::state::RegisterMsg::Err(e)) => {
            state.register.loading = false;
            state.register.error = Some(e);
            Task::none()
        }

        Msg::Logout => {
            if let Some(ref token) = state.auth_token {
                let token = token.clone();
                Task::perform(async move { api::logout(&token).await }, |r| match r {
                    Ok(()) => Msg::LogoutOk,
                    Err(e) => Msg::LogoutErr(e),
                })
            } else {
                update(state, Msg::LogoutOk)
            }
        }
        Msg::LogoutOk => {
            state.config.session = None;
            config::save(&state.config);
            state.auth_token = None;
            state.user = None;
            state.user_stats = None;
            state.reaction_rate = 0;
            state.reaction_song_id.clear();

            let mut close_task = Task::none();
            for (&id, &t) in &state.child_windows {
                if t == WinType::UserProfile {
                    close_task = iced::window::close(id);
                    break;
                }
            }
            state
                .child_windows
                .retain(|_, t| *t != WinType::UserProfile);
            close_task
        }
        Msg::LogoutErr(e) => {
            state.config.session = None;
            config::save(&state.config);
            state.auth_token = None;
            state.user = None;
            state.error_msg = Some(e);
            Task::none()
        }

        Msg::StatsOk(data) => {
            state.stats_loading = false;
            state.user_stats = Some(data);
            Task::none()
        }
        Msg::StatsErr(e) => {
            state.stats_loading = false;
            state.error_msg = Some(e);
            Task::none()
        }

        Msg::React => {
            let Some(ref token) = state.auth_token else {
                state.error_msg = Some(
                    "Please sign in to your Nightwave Plaza account to access this feature.".into(),
                );
                return Task::none();
            };
            let current_song_id = state.status.song.id.clone();
            if current_song_id.is_empty() {
                return Task::none();
            }

            if state.reaction_song_id != current_song_id {
                state.reaction_rate = 0;
                state.reaction_song_id = current_song_id;
            }

            let next_rate = match state.reaction_rate {
                0 => 1,
                1 => 2,
                2 => 0,
                _ => 1,
            };

            let token = token.clone();
            Task::perform(
                async move { api::react(&token, next_rate).await },
                move |r| match r {
                    Ok(resp) => Msg::ReactOk(resp.reactions),
                    Err(e) => Msg::ReactErr(e),
                },
            )
        }
        Msg::ReactOk(new_count) => {
            state.reaction_rate = match state.reaction_rate {
                0 => 1,
                1 => 2,
                2 => 0,
                _ => 1,
            };
            state.status.song.reactions = new_count;
            Task::none()
        }
        Msg::ReactErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }

        Msg::News(crate::state::NewsMsg::Ok(articles, pages)) => {
            state.news.loading = false;
            state.news.list = articles
                .into_iter()
                .map(crate::views::news::ParsedNewsArticle::from)
                .collect();
            state.news.pages = pages;
            Task::none()
        }
        Msg::News(crate::state::NewsMsg::Err(e)) => {
            state.news.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::News(crate::state::NewsMsg::Page(p)) => {
            state.news.page = p;
            state.news.loading = true;
            Task::perform(api::fetch_news(p), |r| match r {
                Ok(n) => Msg::News(crate::state::NewsMsg::Ok(n.data, n.meta.last_page)),
                Err(e) => Msg::News(crate::state::NewsMsg::Err(e.to_string())),
            })
        }

        Msg::Favorites(f_msg) => update_favorites(state, f_msg),
        Msg::Export(e_msg) => update_export(state, e_msg),
        Msg::ProfileEdit(p_msg) => update_profile_edit(state, p_msg),
        Msg::Password(p_msg) => update_password(state, p_msg),
        Msg::DeleteAccount(d_msg) => update_delete(state, d_msg),
        Msg::Lastfm(l_msg) => update_lastfm(state, l_msg),
        Msg::Discord(d_msg) => {
            use crate::state::DiscordMsg;
            match d_msg {
                DiscordMsg::ToggleEnabled(b) => {
                    state.config.discord.enabled = b;
                    config::save(&state.config);
                    discord_update(state);
                }
            }
            Task::none()
        }
        Msg::Timer(t_msg) => {
            use crate::state::TimerMsg;
            match t_msg {
                TimerMsg::Input(s) => {
                    if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                        state.timer.minutes_input = s;
                    }
                }
                TimerMsg::Add(delta) => {
                    let current = state.timer.minutes_input.parse::<i32>().unwrap_or(0);
                    let next = (current + delta).max(1);
                    state.timer.minutes_input = next.to_string();
                }
                TimerMsg::Start => {
                    if let Ok(mins) = state.timer.minutes_input.parse::<u64>() {
                        if mins > 0 {
                            state.timer.until =
                                Some(Instant::now() + Duration::from_secs(mins * 60));
                        }
                    }
                }
                TimerMsg::Stop => state.timer.until = None,
            }
            Task::none()
        }

        Msg::MinimizeWin(id) => iced::window::minimize(id, true),
        Msg::DragWin(id) => iced::window::drag(id),
        Msg::OpenUrl(url) => {
            open_url(&url);
            Task::none()
        }

        Msg::Refresh => Task::perform(api::fetch_status(), |r| match r {
            Ok(s) => Msg::StatusOk(s),
            Err(e) => Msg::StatusErr(e.to_string()),
        }),
        Msg::DismissErr => {
            state.error_msg = None;
            Task::none()
        }

        Msg::Noop => Task::none(),

        Msg::SpaceToggle(id) => {
            if id == state.main_window {
                return update(state, Msg::TogglePlay);
            }
            Task::none()
        }

        Msg::WinResized(id, size) => {
            if dev_mode() {
                let label = if id == state.main_window {
                    "Main".to_string()
                } else {
                    state
                        .child_windows
                        .get(&id)
                        .map(|w| format!("WinType::{:?}", w))
                        .unwrap_or_else(|| "?".to_string())
                };
                eprintln!(
                    "[winsize] {} => iced::Size::new({:.1}, {:.1}),",
                    label, size.width, size.height
                );
            }
            Task::none()
        }

        Msg::WinFocus(id, focused) => {
            if id != state.main_window {
                return Task::none();
            }
            state.main_focused = focused;
            if focused {
                state.last_tick = Instant::now();
                let playing = state.player.as_ref().map_or(false, |p| p.is_playing());
                if playing {
                    return Task::perform(api::fetch_status(), |r| match r {
                        Ok(s) => Msg::StatusOk(s),
                        Err(e) => Msg::StatusErr(e.to_string()),
                    });
                }
            }
            Task::none()
        }

        Msg::Media(event) => {
            use souvlaki::MediaControlEvent as E;
            let now = Instant::now();
            let started_playing = {
                let Some(ref p) = state.player else {
                    return Task::none();
                };
                match event {
                    E::Toggle => {
                        if p.is_playing() {
                            p.stop();
                            false
                        } else {
                            p.play();
                            true
                        }
                    }
                    E::Play => {
                        if !p.is_playing() {
                            p.play();
                            true
                        } else {
                            false
                        }
                    }
                    E::Pause | E::Stop => {
                        if p.is_playing() {
                            p.stop();
                        }
                        false
                    }
                    _ => false,
                }
            };
            let playing_now = state.player.as_ref().map_or(false, |p| p.is_playing());
            if let Some(s) = state.scrobble.as_mut() {
                if playing_now {
                    s.resume(now);
                } else {
                    s.pause(now);
                }
            }
            discord_update(state);
            if started_playing {
                return lastfm_now_playing_task(state);
            }
            Task::none()
        }
    }
}

fn close_windows_of(state: &mut Plaza, wt: WinType) -> Task<Msg> {
    let ids: Vec<_> = state
        .child_windows
        .iter()
        .filter(|(_, &t)| t == wt)
        .map(|(&id, _)| id)
        .collect();
    state.child_windows.retain(|_, t| *t != wt);
    Task::batch(ids.into_iter().map(iced::window::close))
}

fn update_lastfm(state: &mut Plaza, msg: crate::state::LastfmMsg) -> Task<Msg> {
    use crate::state::LastfmMsg;
    match msg {
        LastfmMsg::ToggleEnabled(b) => {
            state.config.lastfm.enabled = b;
            config::save(&state.config);
            if b {
                lastfm_begin_current(state)
            } else {
                Task::none()
            }
        }
        LastfmMsg::Connect => {
            if !lastfm::is_configured() {
                return Task::none();
            }
            state.lastfm_busy = true;
            state.lastfm_status = None;
            Task::perform(async { lastfm::get_token().await }, |r| match r {
                Ok(token) => Msg::Lastfm(LastfmMsg::TokenReady(token)),
                Err(e) => Msg::Lastfm(LastfmMsg::Err(e)),
            })
        }
        LastfmMsg::TokenReady(token) => {
            state.lastfm_busy = false;
            open_url(&lastfm::auth_url(&token));
            state.lastfm_token = Some(token);
            state.lastfm_status = None;
            Task::none()
        }
        LastfmMsg::Finish => {
            let Some(token) = state.lastfm_token.clone() else {
                return Task::none();
            };
            state.lastfm_busy = true;
            state.lastfm_status = None;
            Task::perform(
                async move { lastfm::get_session(&token).await },
                |r| match r {
                    Ok((name, key)) => Msg::Lastfm(LastfmMsg::SessionOk(name, key)),
                    Err(e) => Msg::Lastfm(LastfmMsg::Err(e)),
                },
            )
        }
        LastfmMsg::SessionOk(username, key) => {
            state.lastfm_busy = false;
            state.lastfm_token = None;
            state.config.lastfm.username = Some(username);
            state.config.lastfm.session_key = Some(key);
            state.config.lastfm.enabled = true;
            state.lastfm_status = Some("Connected. Scrobbling is on.".into());
            config::save(&state.config);
            lastfm_begin_current(state)
        }
        LastfmMsg::Disconnect => {
            state.config.lastfm.session_key = None;
            state.config.lastfm.username = None;
            state.config.lastfm.enabled = false;
            state.lastfm_token = None;
            state.lastfm_status = None;
            config::save(&state.config);
            Task::none()
        }
        LastfmMsg::Err(e) => {
            state.lastfm_busy = false;
            state.lastfm_status = Some(e);
            Task::none()
        }
    }
}

fn update_history(state: &mut Plaza, msg: crate::state::HistoryMsg) -> Task<Msg> {
    use crate::state::HistoryMsg;
    match msg {
        HistoryMsg::Ok(songs, pages, total, date_range) => {
            state.history.loading = false;
            state.history.list = songs;
            state.history.pages = pages;
            state.history.total = total;
            state.history.page_input = state.history.page.to_string();
            if let Some(dr) = date_range {
                state.history.date_from = views::widgets::format_date(dr.from_date);
                state.history.date_to = views::widgets::format_date(dr.to_date);
            }
            Task::none()
        }
        HistoryMsg::Err(e) => {
            state.history.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        HistoryMsg::Page(p) => {
            state.history.page = p;
            state.history.page_input = p.to_string();
            state.history.loading = true;
            Task::perform(api::fetch_history(p), |r| match r {
                Ok(h) => Msg::History(HistoryMsg::Ok(
                    h.data,
                    h.meta.last_page,
                    h.meta.total,
                    h.date_range,
                )),
                Err(e) => Msg::History(HistoryMsg::Err(e.to_string())),
            })
        }
        HistoryMsg::PageInput(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                state.history.page_input = s;
            }
            Task::none()
        }
        HistoryMsg::PageSubmit => {
            if let Ok(p) = state.history.page_input.parse::<u32>() {
                let p = p.clamp(1, state.history.pages.max(1));
                if p != state.history.page {
                    return update_history(state, HistoryMsg::Page(p));
                }
            }
            state.history.page_input = state.history.page.to_string();
            Task::none()
        }
    }
}

fn update_ratings(state: &mut Plaza, msg: crate::state::RatingsMsg) -> Task<Msg> {
    use crate::state::RatingsMsg;
    match msg {
        RatingsMsg::Ok(songs, pages, total) => {
            state.ratings.loading = false;
            state.ratings.list = songs;
            state.ratings.pages = pages;
            state.ratings.total = total;
            state.ratings.page_input = state.ratings.page.to_string();
            Task::none()
        }
        RatingsMsg::Err(e) => {
            state.ratings.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        RatingsMsg::Page(p) => {
            state.ratings.page = p;
            state.ratings.page_input = p.to_string();
            state.ratings.loading = true;
            let range = state.ratings.range.clone();
            Task::perform(
                async move { api::fetch_ratings(&range, p).await },
                |r| match r {
                    Ok(h) => Msg::Ratings(RatingsMsg::Ok(h.data, h.meta.last_page, h.meta.total)),
                    Err(e) => Msg::Ratings(RatingsMsg::Err(e.to_string())),
                },
            )
        }
        RatingsMsg::Range(range) => {
            state.ratings.range = range.clone();
            state.ratings.page = 1;
            state.ratings.page_input = "1".into();
            state.ratings.list.clear();
            state.ratings.loading = true;
            Task::perform(
                async move { api::fetch_ratings(&range, 1).await },
                |r| match r {
                    Ok(h) => Msg::Ratings(RatingsMsg::Ok(h.data, h.meta.last_page, h.meta.total)),
                    Err(e) => Msg::Ratings(RatingsMsg::Err(e.to_string())),
                },
            )
        }
        RatingsMsg::PageInput(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                state.ratings.page_input = s;
            }
            Task::none()
        }
        RatingsMsg::PageSubmit => {
            if let Ok(p) = state.ratings.page_input.parse::<u32>() {
                let p = p.clamp(1, state.ratings.pages.max(1));
                if p != state.ratings.page {
                    return update_ratings(state, RatingsMsg::Page(p));
                }
            }
            state.ratings.page_input = state.ratings.page.to_string();
            Task::none()
        }
    }
}

fn update_favorites(state: &mut Plaza, msg: crate::state::FavoritesMsg) -> Task<Msg> {
    use crate::state::FavoritesMsg;
    match msg {
        FavoritesMsg::Ok(list, pages, total) => {
            state.favorites.loading = false;
            state.favorites.pages = pages;
            state.favorites.total = total;
            state.favorites.page_input = state.favorites.page.to_string();

            let mut seen = std::collections::HashSet::new();
            let art_tasks: Vec<Task<Msg>> = list
                .iter()
                .filter_map(|f| f.song.thumb_url())
                .filter(|url| {
                    !state.favorites.artwork.contains_key(*url) && seen.insert(url.to_string())
                })
                .map(|url| {
                    let url = url.to_string();
                    Task::perform(
                        async move {
                            let bytes = api::fetch_artwork(&url).await;
                            (url, bytes)
                        },
                        |(url, bytes)| match bytes {
                            Ok(b) => Msg::Favorites(FavoritesMsg::ArtworkOk(url, b)),
                            Err(_) => Msg::Noop,
                        },
                    )
                })
                .collect();

            state.favorites.list = list;
            Task::batch(art_tasks)
        }
        FavoritesMsg::ArtworkOk(url, bytes) => {
            state
                .favorites
                .artwork
                .insert(url, iced::widget::image::Handle::from_bytes(bytes));
            Task::none()
        }
        FavoritesMsg::Err(e) => {
            state.favorites.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        FavoritesMsg::Page(p) => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.favorites.page = p;
            state.favorites.page_input = p.to_string();
            state.favorites.loading = true;
            state.favorites.deleted.clear();
            Task::perform(
                async move { api::fetch_favorites(&token, p).await },
                |r| match r {
                    Ok(f) => {
                        Msg::Favorites(FavoritesMsg::Ok(f.data, f.meta.last_page, f.meta.total))
                    }
                    Err(e) => Msg::Favorites(FavoritesMsg::Err(e)),
                },
            )
        }
        FavoritesMsg::PageInput(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                state.favorites.page_input = s;
            }
            Task::none()
        }
        FavoritesMsg::PageSubmit => {
            if let Ok(p) = state.favorites.page_input.parse::<u32>() {
                let p = p.clamp(1, state.favorites.pages.max(1));
                if p != state.favorites.page {
                    return update_favorites(state, FavoritesMsg::Page(p));
                }
            }
            state.favorites.page_input = state.favorites.page.to_string();
            Task::none()
        }
        FavoritesMsg::Delete(id) => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            Task::perform(
                async move { api::delete_favorite(&token, id).await },
                move |r| match r {
                    Ok(()) => Msg::Favorites(FavoritesMsg::DeleteOk(id)),
                    Err(e) => Msg::Favorites(FavoritesMsg::DeleteErr(e)),
                },
            )
        }
        FavoritesMsg::DeleteOk(id) => {
            if !state.favorites.deleted.contains(&id) {
                state.favorites.deleted.push(id);
            }
            Task::none()
        }
        FavoritesMsg::DeleteErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }
    }
}

fn update_export(state: &mut Plaza, msg: crate::state::ExportMsg) -> Task<Msg> {
    use crate::state::ExportMsg;
    match msg {
        ExportMsg::Start => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.export.loading = true;
            state.export.link = None;
            state.export.error = None;
            Task::perform(
                async move { api::export_favorites(&token).await },
                |r| match r {
                    Ok(link) => Msg::Export(ExportMsg::Ok(link)),
                    Err(e) => Msg::Export(ExportMsg::Err(e)),
                },
            )
        }
        ExportMsg::Ok(link) => {
            state.export.loading = false;
            state.export.link = Some(link);
            Task::none()
        }
        ExportMsg::Err(e) => {
            state.export.loading = false;
            state.export.error = Some(e);
            Task::none()
        }
    }
}

fn update_profile_edit(state: &mut Plaza, msg: crate::state::ProfileEditMsg) -> Task<Msg> {
    use crate::state::ProfileEditMsg;
    match msg {
        ProfileEditMsg::Username(s) => {
            state.profile_edit.username = s;
            Task::none()
        }
        ProfileEditMsg::Email(s) => {
            state.profile_edit.email = s;
            Task::none()
        }
        ProfileEditMsg::CurrentPassword(s) => {
            state.profile_edit.current_password = s;
            Task::none()
        }
        ProfileEditMsg::Submit => {
            if state.profile_edit.current_password.is_empty() {
                state.profile_edit.error = Some("Current password is required.".into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.profile_edit.loading = true;
            state.profile_edit.error = None;
            let current = state.profile_edit.current_password.clone();
            let username = state.profile_edit.username.clone();
            let email = state.profile_edit.email.clone();
            Task::perform(
                async move { api::update_profile(&token, &current, &username, &email).await },
                |r| match r {
                    Ok(()) => Msg::ProfileEdit(ProfileEditMsg::Ok),
                    Err(e) => Msg::ProfileEdit(ProfileEditMsg::Err(e)),
                },
            )
        }
        ProfileEditMsg::Ok => {
            state.profile_edit.loading = false;
            if let Some(ref mut u) = state.user {
                u.username = state.profile_edit.username.clone();
                u.email = state.profile_edit.email.clone();
                if let Some(ref token) = state.auth_token {
                    state.config.session = Some(config::Session {
                        token: token.clone(),
                        user: u.clone(),
                    });
                    config::save(&state.config);
                }
            }
            state.profile_edit.current_password.clear();
            state.error_msg = Some("Profile has been updated.".into());
            close_windows_of(state, WinType::UserProfileEdit)
        }
        ProfileEditMsg::Err(e) => {
            state.profile_edit.loading = false;
            state.profile_edit.error = Some(e);
            Task::none()
        }
    }
}

fn update_password(state: &mut Plaza, msg: crate::state::PasswordMsg) -> Task<Msg> {
    use crate::state::PasswordMsg;
    match msg {
        PasswordMsg::Current(s) => {
            state.password.current_password = s;
            Task::none()
        }
        PasswordMsg::New(s) => {
            state.password.password = s;
            Task::none()
        }
        PasswordMsg::Repeat(s) => {
            state.password.password_repeat = s;
            Task::none()
        }
        PasswordMsg::Submit => {
            if state.password.current_password.is_empty() {
                state.password.error = Some("Current password is required.".into());
                return Task::none();
            }
            if state.password.password.len() < 3 {
                state.password.error = Some("Password is too short.".into());
                return Task::none();
            }
            if state.password.password != state.password.password_repeat {
                state.password.error = Some("Passwords do not match.".into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.password.loading = true;
            state.password.error = None;
            let current = state.password.current_password.clone();
            let new = state.password.password.clone();
            Task::perform(
                async move { api::update_password(&token, &current, &new).await },
                |r| match r {
                    Ok(()) => Msg::Password(PasswordMsg::Ok),
                    Err(e) => Msg::Password(PasswordMsg::Err(e)),
                },
            )
        }
        PasswordMsg::Ok => {
            state.password = crate::state::PasswordState::default();
            state.config.session = None;
            config::save(&state.config);
            state.auth_token = None;
            state.user = None;
            state.user_stats = None;
            state.error_msg = Some("Password updated. Please log in again.".into());
            Task::batch([
                close_windows_of(state, WinType::UserPassword),
                close_windows_of(state, WinType::UserProfile),
            ])
        }
        PasswordMsg::Err(e) => {
            state.password.loading = false;
            state.password.error = Some(e);
            Task::none()
        }
    }
}

fn update_delete(state: &mut Plaza, msg: crate::state::DeleteMsg) -> Task<Msg> {
    use crate::state::DeleteMsg;
    match msg {
        DeleteMsg::Password(s) => {
            state.delete.current_password = s;
            Task::none()
        }
        DeleteMsg::Confirm(b) => {
            state.delete.confirm = b;
            Task::none()
        }
        DeleteMsg::Submit => {
            if !state.delete.confirm {
                state.delete.error = Some("You must confirm account deletion.".into());
                return Task::none();
            }
            if state.delete.current_password.is_empty() {
                state.delete.error = Some("Current password is required.".into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.delete.loading = true;
            state.delete.error = None;
            let current = state.delete.current_password.clone();
            Task::perform(
                async move { api::delete_profile(&token, &current).await },
                |r| match r {
                    Ok(()) => Msg::DeleteAccount(DeleteMsg::Ok),
                    Err(e) => Msg::DeleteAccount(DeleteMsg::Err(e)),
                },
            )
        }
        DeleteMsg::Ok => {
            state.delete = crate::state::DeleteState::default();
            state.config.session = None;
            config::save(&state.config);
            state.auth_token = None;
            state.user = None;
            state.user_stats = None;
            state.error_msg = Some("Your account has been deleted.".into());
            Task::batch([
                close_windows_of(state, WinType::UserProfileDelete),
                close_windows_of(state, WinType::UserProfileEdit),
                close_windows_of(state, WinType::UserProfile),
            ])
        }
        DeleteMsg::Err(e) => {
            state.delete.loading = false;
            state.delete.error = Some(e);
            Task::none()
        }
    }
}
