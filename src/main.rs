mod api;
mod audio;
mod session;
mod state;
mod theme;
mod views;

use audio::AudioPlayer;
use iced::{Element, Fill, Font, Size, Subscription, Task, Theme};
use state::{Msg, Plaza, WinType};

const APP_ICON: &[u8] = include_bytes!("../assets/icons/favicon-32x32.png");
const TAHOMA: &[u8] = include_bytes!("../assets/fonts/subset-Tahoma.ttf");
const TAHOMA_BOLD: &[u8] = include_bytes!("../assets/fonts/subset-Tahoma-Bold.ttf");
const ICONS_FONT: &[u8] = include_bytes!("../assets/fonts/icons.ttf");

fn app_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(APP_ICON, Some(::image::ImageFormat::Png)).ok()
}
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn load_cjk_font() -> Option<Vec<u8>> {
    let paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/OTF/NotoSansCJK-Regular.ttc",
    ];
    for path in &paths {
        if let Ok(data) = std::fs::read(path) {
            eprintln!("Loaded CJK font from {}", path);
            return Some(data);
        }
    }
    eprintln!("Warning: No CJK font found, Japanese text may not render");
    None
}

fn main() -> iced::Result {
    let cjk = load_cjk_font();

    let base = iced::daemon(win_title, update, win_view)
        .subscription(subscription)
        .theme(|_, _| {
            Theme::custom(
                "Win98".to_string(),
                iced::theme::Palette {
                    background: theme::BG_GRAY,
                    text: theme::BLACK,
                    primary: theme::TITLE_BLUE,
                    success: iced::Color::from_rgb(0.0, 0.5, 0.0),
                    danger: iced::Color::from_rgb(0.8, 0.0, 0.0),
                },
            )
        });

    let font_data: Vec<u8> = cjk.unwrap_or_default();

    base.font(TAHOMA)
        .font(TAHOMA_BOLD)
        .font(ICONS_FONT)
        .font(font_data)
        .default_font(Font {
            family: iced::font::Family::Name("Tahoma"),
            ..Font::DEFAULT
        })
        .run_with(|| {
            let player = AudioPlayer::new().ok().map(Arc::new);
            let (main_id, open_task) = iced::window::open(iced::window::Settings {
                size: Size::new(450.0, 200.0),
                resizable: false,
                decorations: false,
                icon: app_icon(),
                platform_specific: iced::window::settings::PlatformSpecific {
                    application_id: "nightwave-plaza".into(),
                    ..Default::default()
                },
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
                history: Vec::new(),
                history_page: 1,
                history_pages: 0,
                history_total: 0,
                history_date_from: String::new(),
                history_date_to: String::new(),
                history_page_input: "1".into(),
                ratings: Vec::new(),
                ratings_page: 1,
                ratings_pages: 0,
                ratings_range: "overtime".into(),
                ratings_total: 0,
                ratings_page_input: "1".into(),
                history_loading: false,
                ratings_loading: false,
                song_info: None,
                song_info_loading: false,
                song_info_artwork: None,
                elapsed: 0.0,
                last_tick: Instant::now(),
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

                login_username: String::new(),
                login_password: String::new(),
                login_remember: false,
                login_loading: false,
                login_error: None,

                register_username: String::new(),
                register_email: String::new(),
                register_password: String::new(),
                register_password_repeat: String::new(),
                register_loading: false,
                register_error: None,

                news: Vec::new(),
                news_page: 1,
                news_pages: 0,
                news_loading: false,
            };

            let session_task = if let Some(saved) = session::load() {
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
        })
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

fn win_view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
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

fn subscription(_state: &Plaza) -> Subscription<Msg> {
    Subscription::batch([
        iced::time::every(Duration::from_millis(500)).map(|_| Msg::Tick(Instant::now())),
        iced::time::every(Duration::from_secs(5)).map(|_| Msg::Refresh),
        iced::window::close_events().map(Msg::WinClosed),
    ])
}

fn update(state: &mut Plaza, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::StatusOk(status) => {
            let new_art = status.song.artwork_src.clone().unwrap_or_default();
            let need = !new_art.is_empty() && new_art != state.artwork_url;
            state.status = status;
            state.elapsed = state.status.song.position;
            state.last_tick = Instant::now();
            if need {
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
            }
        }
        Msg::StatusErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::Tick(now) => {
            let dt = now.duration_since(state.last_tick).as_secs_f64();
            state.last_tick = now;
            if state.player.as_ref().map_or(false, |p| p.is_playing()) {
                state.elapsed += dt;
            }
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
            Task::none()
        }
        Msg::TogglePlay => {
            if let Some(ref p) = state.player {
                if p.is_playing() {
                    p.stop();
                } else {
                    p.play();
                }
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
                resizable: matches!(wt, WinType::History | WinType::Ratings | WinType::News),
                decorations: false,
                icon: app_icon(),
                ..Default::default()
            });
            state.child_windows.insert(id, wt);
            let mut tasks = vec![task.discard()];
            match wt {
                WinType::History if state.history.is_empty() => {
                    state.history_loading = true;
                    tasks.push(Task::perform(api::fetch_history(1), |r| match r {
                        Ok(h) => {
                            Msg::HistoryOk(h.data, h.meta.last_page, h.meta.total, h.date_range)
                        }
                        Err(e) => Msg::HistoryErr(e.to_string()),
                    }));
                }
                WinType::Ratings if state.ratings.is_empty() => {
                    state.ratings_loading = true;
                    let range = state.ratings_range.clone();
                    tasks.push(Task::perform(
                        async move { api::fetch_ratings(&range, 1).await },
                        |r| match r {
                            Ok(h) => Msg::RatingsOk(h.data, h.meta.last_page, h.meta.total),
                            Err(e) => Msg::RatingsErr(e.to_string()),
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
                WinType::News if state.news.is_empty() => {
                    state.news_loading = true;
                    tasks.push(Task::perform(api::fetch_news(1), |r| match r {
                        Ok(n) => Msg::NewsOk(n.data, n.meta.last_page),
                        Err(e) => Msg::NewsErr(e.to_string()),
                    }));
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

        Msg::HistoryOk(songs, pages, total, date_range) => {
            state.history_loading = false;
            state.history = songs;
            state.history_pages = pages;
            state.history_total = total;
            state.history_page_input = state.history_page.to_string();
            if let Some(dr) = date_range {
                state.history_date_from = views::widgets::format_date(dr.from_date);
                state.history_date_to = views::widgets::format_date(dr.to_date);
            }
            Task::none()
        }
        Msg::HistoryErr(e) => {
            state.history_loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::HistoryPage(p) => {
            state.history_page = p;
            state.history_page_input = p.to_string();
            state.history_loading = true;
            Task::perform(api::fetch_history(p), |r| match r {
                Ok(h) => Msg::HistoryOk(h.data, h.meta.last_page, h.meta.total, h.date_range),
                Err(e) => Msg::HistoryErr(e.to_string()),
            })
        }
        Msg::HistoryPageInput(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                state.history_page_input = s;
            }
            Task::none()
        }
        Msg::HistoryPageSubmit => {
            if let Ok(p) = state.history_page_input.parse::<u32>() {
                let p = p.clamp(1, state.history_pages.max(1));
                if p != state.history_page {
                    return update(state, Msg::HistoryPage(p));
                }
            }
            state.history_page_input = state.history_page.to_string();
            Task::none()
        }

        Msg::RatingsOk(songs, pages, total) => {
            state.ratings_loading = false;
            state.ratings = songs;
            state.ratings_pages = pages;
            state.ratings_total = total;
            state.ratings_page_input = state.ratings_page.to_string();
            Task::none()
        }
        Msg::RatingsErr(e) => {
            state.ratings_loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::RatingsPage(p) => {
            state.ratings_page = p;
            state.ratings_page_input = p.to_string();
            state.ratings_loading = true;
            let range = state.ratings_range.clone();
            Task::perform(
                async move { api::fetch_ratings(&range, p).await },
                |r| match r {
                    Ok(h) => Msg::RatingsOk(h.data, h.meta.last_page, h.meta.total),
                    Err(e) => Msg::RatingsErr(e.to_string()),
                },
            )
        }
        Msg::RatingsRange(range) => {
            state.ratings_range = range.clone();
            state.ratings_page = 1;
            state.ratings_page_input = "1".into();
            state.ratings.clear();
            state.ratings_loading = true;
            Task::perform(
                async move { api::fetch_ratings(&range, 1).await },
                |r| match r {
                    Ok(h) => Msg::RatingsOk(h.data, h.meta.last_page, h.meta.total),
                    Err(e) => Msg::RatingsErr(e.to_string()),
                },
            )
        }
        Msg::RatingsPageInput(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) || s.is_empty() {
                state.ratings_page_input = s;
            }
            Task::none()
        }
        Msg::RatingsPageSubmit => {
            if let Ok(p) = state.ratings_page_input.parse::<u32>() {
                let p = p.clamp(1, state.ratings_pages.max(1));
                if p != state.ratings_page {
                    return update(state, Msg::RatingsPage(p));
                }
            }
            state.ratings_page_input = state.ratings_page.to_string();
            Task::none()
        }

        Msg::OpenSongInfo(song_id) => {
            for (&id, &t) in &state.child_windows {
                if t == WinType::SongInfo {
                    state.song_info = None;
                    state.song_info_loading = true;
                    state.song_info_artwork = None;
                    return Task::batch([
                        iced::window::gain_focus(id),
                        Task::perform(
                            async move { api::fetch_song(&song_id).await },
                            |r| match r {
                                Ok(s) => Msg::SongInfoOk(s),
                                Err(e) => Msg::SongInfoErr(e.to_string()),
                            },
                        ),
                    ]);
                }
            }
            let (id, task) = iced::window::open(iced::window::Settings {
                size: WinType::SongInfo.size(),
                resizable: false,
                decorations: false,
                icon: app_icon(),
                ..Default::default()
            });
            state.child_windows.insert(id, WinType::SongInfo);
            state.song_info = None;
            state.song_info_loading = true;
            state.song_info_artwork = None;
            Task::batch([
                task.discard(),
                Task::perform(
                    async move { api::fetch_song(&song_id).await },
                    |r| match r {
                        Ok(s) => Msg::SongInfoOk(s),
                        Err(e) => Msg::SongInfoErr(e.to_string()),
                    },
                ),
            ])
        }
        Msg::SongInfoOk(resp) => {
            state.song_info_loading = false;
            let art_url = resp.data.artwork_src.clone().unwrap_or_default();
            state.song_info = Some(resp);
            if !art_url.is_empty() && Some(&art_url) != Some(&state.artwork_url) {
                Task::perform(
                    async move {
                        api::fetch_artwork(&art_url)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |r| match r {
                        Ok(b) => Msg::SongInfoArtworkOk(b),
                        Err(_) => Msg::SongInfoArtworkErr,
                    },
                )
            } else {
                state.song_info_artwork = state.artwork_handle.clone();
                Task::none()
            }
        }
        Msg::SongInfoErr(e) => {
            state.song_info_loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::SongInfoArtworkOk(b) => {
            state.song_info_artwork = Some(iced::widget::image::Handle::from_bytes(b));
            Task::none()
        }
        Msg::SongInfoArtworkErr => {
            state.song_info_artwork = None;
            Task::none()
        }

        Msg::SessionRestored(user, token) => {
            session::save(&token, &user);
            state.auth_token = Some(token);
            state.user = Some(user);
            Task::none()
        }

        Msg::LoginUsername(s) => {
            state.login_username = s;
            Task::none()
        }
        Msg::LoginPassword(s) => {
            state.login_password = s;
            Task::none()
        }
        Msg::LoginRemember(b) => {
            state.login_remember = b;
            Task::none()
        }
        Msg::LoginSubmit => {
            if state.login_username.is_empty() || state.login_password.is_empty() {
                state.login_error = Some("Please enter a username and password.".into());
                return Task::none();
            }
            state.login_loading = true;
            state.login_error = None;
            let username = state.login_username.clone();
            let password = state.login_password.clone();
            Task::perform(
                async move { api::login(&username, &password).await },
                |r| match r {
                    Ok(resp) => Msg::LoginOk(resp),
                    Err(e) => Msg::LoginErr(e),
                },
            )
        }
        Msg::LoginOk(resp) => {
            state.login_loading = false;

            if state.login_remember {
                if let Some(ref token) = resp.token {
                    session::save(token, &resp.data);
                }
            }
            state.auth_token = resp.token;
            state.user = Some(resp.data);
            state.login_username.clear();
            state.login_password.clear();
            state.login_remember = false;
            state.login_error = None;

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
        Msg::LoginErr(e) => {
            state.login_loading = false;
            state.login_error = Some(e);
            Task::none()
        }

        Msg::RegisterUsername(s) => {
            state.register_username = s;
            Task::none()
        }
        Msg::RegisterEmail(s) => {
            state.register_email = s;
            Task::none()
        }
        Msg::RegisterPassword(s) => {
            state.register_password = s;
            Task::none()
        }
        Msg::RegisterPasswordRepeat(s) => {
            state.register_password_repeat = s;
            Task::none()
        }
        Msg::RegisterSubmit => {
            if !state
                .register_username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                state.register_error =
                    Some("Username may only contain letters, numbers, and underscores.".into());
                return Task::none();
            }
            if state.register_username.len() < 4 {
                state.register_error = Some("Username is too short.".into());
                return Task::none();
            }
            if state.register_username.len() > 32 {
                state.register_error = Some("Username is too long.".into());
                return Task::none();
            }
            if state.register_password.len() < 3 {
                state.register_error = Some("Password is too short.".into());
                return Task::none();
            }
            if state.register_password != state.register_password_repeat {
                state.register_error = Some("Passwords do not match.".into());
                return Task::none();
            }
            if state.register_email.is_empty() {
                state.register_error = Some("Email is required.".into());
                return Task::none();
            }
            state.register_loading = true;
            state.register_error = None;
            let username = state.register_username.clone();
            let email = state.register_email.clone();
            let password = state.register_password.clone();
            Task::perform(
                async move { api::register(&username, &email, &password).await },
                |r| match r {
                    Ok(user) => Msg::RegisterOk(user),
                    Err(e) => Msg::RegisterErr(e),
                },
            )
        }
        Msg::RegisterOk(_user) => {
            state.register_loading = false;
            state.register_error = None;
            state.register_username.clear();
            state.register_email.clear();
            state.register_password.clear();
            state.register_password_repeat.clear();

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
        Msg::RegisterErr(e) => {
            state.register_loading = false;
            state.register_error = Some(e);
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
            session::clear();
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
            session::clear();
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

        Msg::NewsOk(articles, pages) => {
            state.news_loading = false;
            state.news = articles;
            state.news_pages = pages;
            Task::none()
        }
        Msg::NewsErr(e) => {
            state.news_loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::NewsPage(p) => {
            state.news_page = p;
            state.news_loading = true;
            Task::perform(api::fetch_news(p), |r| match r {
                Ok(n) => Msg::NewsOk(n.data, n.meta.last_page),
                Err(e) => Msg::NewsErr(e.to_string()),
            })
        }

        Msg::MinimizeWin(id) => iced::window::minimize(id, true),
        Msg::DragWin(id) => iced::window::drag(id),
        Msg::OpenUrl(url) => {
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
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
    }
}
