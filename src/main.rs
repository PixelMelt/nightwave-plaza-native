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
use iced::{Element, Fill, Font, Size, Subscription, Task};
use state::{
    DeleteMsg, DiscordMsg, ExportMsg, FavoritesMsg, HistoryMsg, LastfmMsg, LoginMsg, Msg, NewsMsg,
    PasswordMsg, Plaza, ProfileEditMsg, RatingsMsg, RegisterMsg, ScrobbleTrack, SongInfoMsg,
    TimerMsg, WinType,
};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Messages produced by the audio thread (media keys, stream state); handed to
/// the subscription that forwards them into the update loop.
static AUDIO_EVENTS: Mutex<Option<futures_mpsc::UnboundedReceiver<Msg>>> = Mutex::new(None);

const APP_ICON: &[u8] = include_bytes!("assets/icons/favicon-32x32.png");
const TAHOMA: &[u8] = include_bytes!("assets/fonts/subset-Tahoma.ttf");
const TAHOMA_BOLD: &[u8] = include_bytes!("assets/fonts/subset-Tahoma-Bold.ttf");
const ICONS_FONT: &[u8] = include_bytes!("assets/fonts/icons.ttf");

/// Largest edge of the decoded cover art; drawn at 112 logical px at most.
const COVER_PX: u32 = 256;
/// Largest edge of favorites thumbnails; drawn at 54 logical px.
const THUMB_PX: u32 = 128;

fn app_icon() -> Option<iced::window::Icon> {
    static ICON: OnceLock<Option<iced::window::Icon>> = OnceLock::new();
    ICON.get_or_init(|| {
        let img = ::image::load_from_memory(APP_ICON).ok()?.into_rgba8();
        let (w, h) = img.dimensions();
        iced::window::icon::from_rgba(img.into_raw(), w, h).ok()
    })
    .clone()
}

fn dev_mode() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| std::env::var_os("NIGHTWAVE_DEV").is_some())
}

fn bench_mode() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| std::env::var_os("NIGHTWAVE_BENCH").is_some())
}

fn window_settings(size: Size, resizable: bool) -> iced::window::Settings {
    #[cfg(target_os = "linux")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        application_id: "nightwave-plaza".into(),
        ..Default::default()
    };
    #[cfg(not(target_os = "linux"))]
    let platform_specific = iced::window::settings::PlatformSpecific::default();

    iced::window::Settings {
        size,
        resizable: resizable || dev_mode(),
        decorations: dev_mode(),
        icon: app_icon(),
        platform_specific,
        ..Default::default()
    }
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

/// Exits immediately. A graceful shutdown would join the media-controls
/// D-Bus thread, which polls with a one second timeout; nothing needs
/// flushing (config is written on every change), so do not make the user wait.
fn quit() -> ! {
    std::process::exit(0)
}

pub(crate) fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn main() -> iced::Result {
    iced::daemon(boot, update, win_view)
        .title(win_title)
        .subscription(subscription)
        .theme(|_: &Plaza, _| theme::app_theme())
        .font(TAHOMA)
        .font(TAHOMA_BOLD)
        .font(ICONS_FONT)
        .default_font(Font {
            family: iced::font::Family::Name("Tahoma"),
            ..Font::DEFAULT
        })
        .run()
}

fn boot() -> (Plaza, Task<Msg>) {
    let (events_tx, events_rx) = futures_mpsc::unbounded();
    *AUDIO_EVENTS.lock().unwrap() = Some(events_rx);
    let player = AudioPlayer::new(events_tx);

    let (main_id, open_task) = iced::window::open(window_settings(Size::new(450.0, 218.0), false));
    let mut state = Plaza::new(main_id, player, config::load());

    let session_task = match state.config.session.clone() {
        Some(saved) => {
            let token = saved.token.clone();
            state.auth_token = Some(saved.token);
            state.user = Some(saved.user);
            result_task(
                async move {
                    let user = api::get_me(&token).await?;
                    Ok((user, token))
                },
                |(user, token)| Msg::SessionRestored(user, token),
                |_| Msg::LogoutOk,
            )
        }
        None => Task::none(),
    };

    (
        state,
        Task::batch([open_task.discard(), fetch_status_task(), session_task]),
    )
}

fn win_title(state: &Plaza, wid: iced::window::Id) -> String {
    let song = &state.status.song;
    match state.child_windows.get(&wid) {
        Some(wt) => wt.title().to_string(),
        None if wid == state.main_window && !song.artist.is_empty() => {
            format!("{} - {} - Nightwave Plaza", song.artist, song.title)
        }
        None => "Nightwave Plaza".into(),
    }
}

fn win_view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let wt = state.child_windows.get(&wid);
    let inner = if wid == state.main_window {
        views::player::view(state)
    } else {
        match wt {
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
        }
    };
    let title = wt.map_or("Nightwave Plaza", WinType::title).to_string();
    let title_bar = views::title_bar(title, wid, wt, state.focused == Some(wid));

    let framed = iced::widget::column![title_bar, inner]
        .spacing(1)
        .padding(1)
        .width(Fill)
        .height(Fill);

    let window_inner = iced::widget::container(framed)
        .padding(2)
        .width(Fill)
        .height(Fill)
        .style(theme::panel);

    views::d3_raised_window(window_inner)
        .width(Fill)
        .height(Fill)
        .into()
}

fn subscription(state: &Plaza) -> Subscription<Msg> {
    let is_playing = state.is_playing();

    // The clock shows whole seconds, so one tick per second is enough, and
    // only while the clock is visible and changing (or the sleep timer runs).
    let display_active = state.main_focused()
        && (is_playing || state.welcome_until.is_some() || state.volume_text_until.is_some());
    let tick_period =
        (display_active || state.timer.until.is_some()).then(|| Duration::from_secs(1));

    // Poll status when the current song should have ended, and otherwise
    // every 30 s for the listener and like counts. The period is derived from
    // the last status only, so the subscription restarts exactly when a new
    // status arrives and its countdown starts from that moment.
    let refresh_period = {
        let song = &state.status.song;
        let remaining = if is_playing && song.length > 0.0 {
            Duration::from_secs_f64((song.length - song.position).max(0.0)) + Duration::from_secs(1)
        } else {
            Duration::MAX
        };
        remaining.clamp(Duration::from_secs(2), Duration::from_secs(30))
    };

    let mut subs = vec![
        iced::time::every(refresh_period).map(|_| Msg::Refresh),
        iced::window::close_events().map(Msg::WinClosed),
        iced::event::listen_with(|event, status, id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                ..
            }) if status == iced::event::Status::Ignored => Some(Msg::SpaceToggle(id)),
            iced::Event::Window(iced::window::Event::Opened { .. }) if bench_mode() => {
                Some(Msg::WinClosed(id))
            }
            iced::Event::Window(iced::window::Event::Focused) => Some(Msg::WinFocus(id, true)),
            iced::Event::Window(iced::window::Event::Unfocused) => Some(Msg::WinFocus(id, false)),
            _ => None,
        }),
        Subscription::run(audio_event_stream),
    ];

    if let Some(period) = tick_period {
        subs.push(iced::time::every(period).map(|_| Msg::Tick(Instant::now())));
    }

    if dev_mode() {
        subs.push(iced::window::resize_events().map(|(id, size)| Msg::WinResized(id, size)));
    }

    Subscription::batch(subs)
}

fn audio_event_stream() -> impl futures::Stream<Item = Msg> {
    iced::stream::channel(64, |mut output: futures_mpsc::Sender<Msg>| async move {
        use futures::{SinkExt, StreamExt};

        let Some(mut rx) = AUDIO_EVENTS.lock().ok().and_then(|mut g| g.take()) else {
            return;
        };
        while let Some(msg) = rx.next().await {
            let _ = output.send(msg).await;
        }
    })
}

fn update(state: &mut Plaza, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::StatusOk(status) => {
            let now = Instant::now();
            let playing = state.is_playing();
            let song_changed = state
                .scrobble
                .as_ref()
                .is_none_or(|t| t.song_id != status.song.id);
            let mut tasks = Vec::new();

            if song_changed {
                if let Some(old) = state.scrobble.take() {
                    tasks.push(lastfm_scrobble_task(state, &old, now));
                }
                state.scrobble = Some(ScrobbleTrack::new(&status.song));
            }
            if let Some(track) = state.scrobble.as_mut() {
                if playing {
                    track.resume(now);
                } else {
                    track.pause(now);
                }
            }

            let art = artwork_to_fetch(&status.song.artwork_src, &state.artwork_url);
            state.status = status;
            state.elapsed = state.status.song.position;
            state.last_tick = now;
            state.player.update_metadata(&state.status.song);
            if song_changed && playing {
                tasks.push(lastfm_now_playing_task(state));
            }
            discord_update(state);

            if let Some(url) = art {
                state.artwork_url = url.clone();
                tasks.push(result_task(
                    async move { api::fetch_artwork(&url, COVER_PX).await },
                    Msg::ArtworkOk,
                    |_| Msg::ArtworkErr,
                ));
            }
            Task::batch(tasks)
        }
        Msg::StatusErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }
        Msg::Tick(now) => {
            let dt = now.duration_since(state.last_tick).as_secs_f64().min(1.5);
            state.last_tick = now;
            state.elapsed += dt;
            if state.welcome_until.is_some_and(|until| now >= until) {
                state.welcome_until = None;
            }
            if state.volume_text_until.is_some_and(|until| now >= until) {
                state.volume_text = None;
                state.volume_text_until = None;
            }
            if state.timer.until.is_some_and(|until| now >= until) {
                state.timer.until = None;
                state.player.stop();
                return playback_changed(state, false);
            }
            Task::none()
        }
        Msg::StreamChanged => Task::none(),

        Msg::TogglePlay => {
            let start = !state.is_playing();
            if start {
                state.player.play();
            } else {
                state.player.stop();
            }
            playback_changed(state, start)
        }
        Msg::Media(event) => {
            use souvlaki::MediaControlEvent as E;
            let was_playing = state.is_playing();
            match event {
                E::Toggle if was_playing => state.player.stop(),
                E::Toggle | E::Play => state.player.play(),
                E::Pause | E::Stop => state.player.stop(),
                _ => {}
            }
            playback_changed(state, !was_playing && state.is_playing())
        }
        Msg::Volume(v) => {
            state.volume = v;
            state.player.set_volume(v / 100.0);
            state.volume_text = Some(format!("Volume: {}%", v as u32));
            state.volume_text_until = Some(Instant::now() + Duration::from_secs(2));
            state.welcome_until = None;
            Task::none()
        }
        Msg::ArtworkOk(handle) => {
            state.artwork_handle = Some(handle);
            Task::none()
        }
        Msg::ArtworkErr => {
            state.artwork_handle = None;
            Task::none()
        }

        Msg::OpenWin(wt) => {
            if let Some(id) = state.window_of(wt) {
                return iced::window::gain_focus(id);
            }
            let mut tasks = vec![open_window(state, wt)];
            match wt {
                WinType::History if state.history.list.is_empty() => {
                    state.history.pager.goto(1);
                    tasks.push(fetch_history_task(1));
                }
                WinType::Ratings if state.ratings.list.is_empty() => {
                    state.ratings.pager.goto(1);
                    tasks.push(fetch_ratings_task(state.ratings.range.clone(), 1));
                }
                WinType::UserProfile => {
                    if let Some(token) = state.auth_token.clone() {
                        state.stats_loading = true;
                        tasks.push(result_task(
                            async move { api::get_stats(&token).await },
                            |s: api::UserStatsResponse| Msg::StatsOk(s.data),
                            Msg::StatsErr,
                        ));
                    }
                }
                WinType::News if state.news.list.is_empty() => {
                    state.news.pager.goto(1);
                    tasks.push(fetch_news_task(1));
                }
                WinType::UserFavorites => {
                    state.favorites.deleted.clear();
                    if let Some(token) = state.auth_token.clone() {
                        state.favorites.pager.goto(1);
                        tasks.push(fetch_favorites_task(token, 1));
                    }
                }
                WinType::UserFavoritesExport => {
                    state.export = state::ExportState::default();
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
                    state.password = state::PasswordState::default();
                }
                WinType::UserProfileDelete => {
                    state.delete = state::DeleteState::default();
                }
                _ => {}
            }
            Task::batch(tasks)
        }
        Msg::CloseWin(id) => {
            forget_window(state, id);
            iced::window::close(id)
        }
        Msg::WinClosed(id) => {
            forget_window(state, id);
            Task::none()
        }

        Msg::History(msg) => update_history(state, msg),
        Msg::Ratings(msg) => update_ratings(state, msg),
        Msg::SongInfo(msg) => update_song_info(state, msg),
        Msg::Login(msg) => update_login(state, msg),
        Msg::Register(msg) => update_register(state, msg),
        Msg::News(msg) => update_news(state, msg),
        Msg::Favorites(msg) => update_favorites(state, msg),
        Msg::Export(msg) => update_export(state, msg),
        Msg::ProfileEdit(msg) => update_profile_edit(state, msg),
        Msg::Password(msg) => update_password(state, msg),
        Msg::DeleteAccount(msg) => update_delete(state, msg),
        Msg::Lastfm(msg) => update_lastfm(state, msg),
        Msg::Discord(DiscordMsg::ToggleEnabled(b)) => {
            state.config.discord.enabled = b;
            config::save(&state.config);
            discord_update(state);
            Task::none()
        }
        Msg::Timer(msg) => {
            match msg {
                TimerMsg::Input(s) => state::digits_input(&mut state.timer.minutes_input, s),
                TimerMsg::Add(delta) => {
                    let current = state.timer.minutes_input.parse::<i32>().unwrap_or(0);
                    state.timer.minutes_input = (current + delta).max(1).to_string();
                }
                TimerMsg::Start => {
                    if let Ok(mins @ 1..) = state.timer.minutes_input.parse::<u64>() {
                        state.timer.until = Some(Instant::now() + Duration::from_secs(mins * 60));
                    }
                }
                TimerMsg::Stop => state.timer.until = None,
            }
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
        Msg::Logout => match state.auth_token.clone() {
            Some(token) => result_task(
                async move { api::logout(&token).await },
                |()| Msg::LogoutOk,
                Msg::LogoutErr,
            ),
            None => update(state, Msg::LogoutOk),
        },
        Msg::LogoutOk => {
            clear_session(state);
            state.reaction_rate = 0;
            state.reaction_song_id.clear();
            close_windows_of(state, WinType::UserProfile)
        }
        Msg::LogoutErr(e) => {
            clear_session(state);
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
            let Some(token) = state.auth_token.clone() else {
                state.error_msg = Some(
                    "Please sign in to your Nightwave Plaza account to access this feature.".into(),
                );
                return Task::none();
            };
            let song_id = state.status.song.id.clone();
            if song_id.is_empty() {
                return Task::none();
            }
            if state.reaction_song_id != song_id {
                state.reaction_rate = 0;
                state.reaction_song_id = song_id;
            }
            let next_rate = next_reaction_rate(state.reaction_rate);
            result_task(
                async move { api::react(&token, next_rate).await },
                |resp| Msg::ReactOk(resp.reactions),
                Msg::ReactErr,
            )
        }
        Msg::ReactOk(new_count) => {
            state.reaction_rate = next_reaction_rate(state.reaction_rate);
            state.status.song.reactions = new_count;
            Task::none()
        }
        Msg::ReactErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }

        Msg::MinimizeWin(id) => iced::window::minimize(id, true),
        Msg::DragWin(id) => iced::window::drag(id),
        Msg::OpenUrl(url) => {
            open_url(&url);
            Task::none()
        }
        Msg::Refresh => fetch_status_task(),
        Msg::DismissErr => {
            state.error_msg = None;
            Task::none()
        }
        Msg::Noop => Task::none(),
        Msg::SpaceToggle(id) if id == state.main_window => update(state, Msg::TogglePlay),
        Msg::SpaceToggle(_) => Task::none(),

        Msg::WinResized(id, size) => {
            let label = match state.child_windows.get(&id) {
                Some(w) => format!("WinType::{w:?}"),
                None => "Main".to_string(),
            };
            eprintln!(
                "[winsize] {label} => iced::Size::new({:.1}, {:.1}),",
                size.width, size.height
            );
            Task::none()
        }
        Msg::WinFocus(id, focused) => {
            // Focus can arrive before the previous window's unfocus, so only
            // clear when the unfocused window is the one we think is focused.
            if focused {
                state.focused = Some(id);
            } else if state.focused == Some(id) {
                state.focused = None;
            }
            if focused && id == state.main_window {
                state.last_tick = Instant::now();
                if state.is_playing() {
                    return fetch_status_task();
                }
            }
            Task::none()
        }
    }
}

/// Keeps the scrobble clock, Discord presence and Last.fm "now playing" in
/// step after playback started or stopped.
fn playback_changed(state: &mut Plaza, started: bool) -> Task<Msg> {
    let now = Instant::now();
    let playing = state.is_playing();
    if let Some(track) = state.scrobble.as_mut() {
        if playing {
            track.resume(now);
        } else {
            track.pause(now);
        }
    }
    discord_update(state);
    if started {
        lastfm_now_playing_task(state)
    } else {
        Task::none()
    }
}

fn lastfm_now_playing_task(state: &Plaza) -> Task<Msg> {
    let Some(sk) = state.config.lastfm.active_session_key().map(str::to_owned) else {
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

fn lastfm_scrobble_task(state: &Plaza, track: &ScrobbleTrack, now: Instant) -> Task<Msg> {
    let Some(sk) = state.config.lastfm.active_session_key().map(str::to_owned) else {
        return Task::none();
    };
    let Some(start_unix) = track.start_unix else {
        return Task::none();
    };
    if track.artist.is_empty() || track.title.is_empty() {
        return Task::none();
    }
    // Last.fm rules: tracks under 30s never scrobble; otherwise half the
    // track or 4 minutes, whichever comes first (30s if length is unknown).
    if track.duration > 0.0 && track.duration < 30.0 {
        return Task::none();
    }
    let threshold = if track.duration > 0.0 {
        (track.duration / 2.0).min(240.0)
    } else {
        30.0
    };
    if track.total_played(now) < threshold {
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
    let handle = &state.discord_presence;
    let song = &state.status.song;
    if !state.config.discord.enabled || !state.is_playing() || song.title.is_empty() {
        handle.clear();
        return;
    }
    let start = now_unix() as i64 - song.position.max(0.0) as i64;
    let end = (song.length > 0.0).then(|| start + song.length as i64);
    handle.set(discord::Presence {
        title: song.title.clone(),
        artist: song.artist.clone(),
        album: song.album.clone(),
        cover_url: song.artwork_src.clone().filter(|s| !s.is_empty()),
        start_unix: start,
        end_unix: end,
    });
}

fn next_reaction_rate(rate: u8) -> u8 {
    (rate + 1) % 3
}

fn artwork_to_fetch(src: &Option<String>, current: &str) -> Option<String> {
    src.as_deref()
        .filter(|s| !s.is_empty() && *s != current)
        .map(str::to_owned)
}

fn fetch_status_task() -> Task<Msg> {
    result_task(api::fetch_status(), Msg::StatusOk, Msg::StatusErr)
}

fn clear_session(state: &mut Plaza) {
    state.config.session = None;
    config::save(&state.config);
    state.auth_token = None;
    state.user = None;
    state.user_stats = None;
}

fn result_task<T: Send + 'static>(
    fut: impl std::future::Future<Output = Result<T, String>> + Send + 'static,
    ok: impl Fn(T) -> Msg + Send + 'static,
    err: impl Fn(String) -> Msg + Send + 'static,
) -> Task<Msg> {
    Task::perform(fut, move |r| match r {
        Ok(v) => ok(v),
        Err(e) => err(e),
    })
}

/// Opens a new window of the given type; callers check `window_of` first if
/// they only want to focus an existing one.
fn open_window(state: &mut Plaza, wt: WinType) -> Task<Msg> {
    let (id, task) = iced::window::open(window_settings(wt.size(), wt.resizable()));
    state.child_windows.insert(id, wt);
    task.discard()
}

/// Drops bookkeeping for a window that is closing; quits if it is the main one.
fn forget_window(state: &mut Plaza, id: iced::window::Id) {
    if id == state.main_window {
        quit();
    }
    state.child_windows.remove(&id);
    if state.focused == Some(id) {
        state.focused = None;
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

fn update_song_info(state: &mut Plaza, msg: SongInfoMsg) -> Task<Msg> {
    let info = &mut state.song_info;
    match msg {
        SongInfoMsg::Open(song_id) => {
            *info = state::SongInfoState {
                loading: true,
                ..Default::default()
            };
            let window = match state.window_of(WinType::SongInfo) {
                Some(id) => iced::window::gain_focus(id),
                None => open_window(state, WinType::SongInfo),
            };
            Task::batch([
                window,
                result_task(
                    async move { api::fetch_song(&song_id).await },
                    |s| Msg::SongInfo(SongInfoMsg::Ok(s)),
                    |e| Msg::SongInfo(SongInfoMsg::Err(e)),
                ),
            ])
        }
        SongInfoMsg::Ok(resp) => {
            info.loading = false;
            let art = artwork_to_fetch(&resp.data.artwork_src, &state.artwork_url);
            info.data = Some(resp);
            match art {
                Some(url) => result_task(
                    async move { api::fetch_artwork(&url, COVER_PX).await },
                    |h| Msg::SongInfo(SongInfoMsg::ArtworkOk(h)),
                    |_| Msg::SongInfo(SongInfoMsg::ArtworkErr),
                ),
                None => {
                    info.artwork = state.artwork_handle.clone();
                    Task::none()
                }
            }
        }
        SongInfoMsg::Err(e) => {
            info.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        SongInfoMsg::ArtworkOk(handle) => {
            info.artwork = Some(handle);
            Task::none()
        }
        SongInfoMsg::ArtworkErr => {
            info.artwork = None;
            Task::none()
        }
        SongInfoMsg::ToggleFavorite => {
            let Some(token) = state.auth_token.clone() else {
                state.error_msg = Some("Please sign in to favorite songs.".into());
                return Task::none();
            };
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
                Some(fav_id) => result_task(
                    async move { api::delete_favorite(&token, fav_id).await },
                    |()| Msg::SongInfo(SongInfoMsg::FavoriteRemoved),
                    |e| Msg::SongInfo(SongInfoMsg::FavoriteErr(e)),
                ),
                None => result_task(
                    async move { api::add_favorite(&token, &song_id).await },
                    |id| Msg::SongInfo(SongInfoMsg::FavoriteAdded(id)),
                    |e| Msg::SongInfo(SongInfoMsg::FavoriteErr(e)),
                ),
            }
        }
        SongInfoMsg::FavoriteAdded(id) => {
            info.fav_sending = false;
            info.favorite_id = Some(id);
            Task::none()
        }
        SongInfoMsg::FavoriteRemoved => {
            info.fav_sending = false;
            info.favorite_id = None;
            Task::none()
        }
        SongInfoMsg::FavoriteErr(e) => {
            info.fav_sending = false;
            state.error_msg = Some(e);
            Task::none()
        }
    }
}

fn update_login(state: &mut Plaza, msg: LoginMsg) -> Task<Msg> {
    let login = &mut state.login;
    match msg {
        LoginMsg::Username(s) => login.username = s,
        LoginMsg::Password(s) => login.password = s,
        LoginMsg::Remember(b) => login.remember = b,
        LoginMsg::Submit => {
            if login.username.is_empty() || login.password.is_empty() {
                login.error = Some("Please enter a username and password.".into());
                return Task::none();
            }
            login.loading = true;
            login.error = None;
            let (username, password, remember) = (
                login.username.clone(),
                login.password.clone(),
                login.remember,
            );
            return result_task(
                async move { api::login(&username, &password, remember).await },
                |resp| Msg::Login(LoginMsg::Ok(resp)),
                |e| Msg::Login(LoginMsg::Err(e)),
            );
        }
        LoginMsg::Ok(resp) => {
            if login.remember {
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
            state.login = state::LoginState::default();
            return close_windows_of(state, WinType::UserLogin);
        }
        LoginMsg::Err(e) => {
            login.loading = false;
            login.error = Some(e);
        }
    }
    Task::none()
}

fn update_register(state: &mut Plaza, msg: RegisterMsg) -> Task<Msg> {
    let reg = &mut state.register;
    match msg {
        RegisterMsg::Username(s) => reg.username = s,
        RegisterMsg::Email(s) => reg.email = s,
        RegisterMsg::Password(s) => reg.password = s,
        RegisterMsg::PasswordRepeat(s) => reg.password_repeat = s,
        RegisterMsg::Submit => {
            let problem = if !reg
                .username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                Some("Username may only contain letters, numbers, and underscores.")
            } else if reg.username.len() < 4 {
                Some("Username is too short.")
            } else if reg.username.len() > 32 {
                Some("Username is too long.")
            } else if reg.password.len() < 3 {
                Some("Password is too short.")
            } else if reg.password != reg.password_repeat {
                Some("Passwords do not match.")
            } else if reg.email.is_empty() {
                Some("Email is required.")
            } else {
                None
            };
            if let Some(problem) = problem {
                reg.error = Some(problem.into());
                return Task::none();
            }
            reg.loading = true;
            reg.error = None;
            let (username, email, password) = (
                reg.username.clone(),
                reg.email.clone(),
                reg.password.clone(),
            );
            return result_task(
                async move { api::register(&username, &email, &password).await },
                |user| Msg::Register(RegisterMsg::Ok(user)),
                |e| Msg::Register(RegisterMsg::Err(e)),
            );
        }
        RegisterMsg::Ok(_user) => {
            state.register = state::RegisterState::default();
            state.error_msg = Some("Registration successful! You can now log in.".into());
            return close_windows_of(state, WinType::UserRegister);
        }
        RegisterMsg::Err(e) => {
            reg.loading = false;
            reg.error = Some(e);
        }
    }
    Task::none()
}

fn update_lastfm(state: &mut Plaza, msg: LastfmMsg) -> Task<Msg> {
    match msg {
        LastfmMsg::ToggleEnabled(b) => {
            state.config.lastfm.enabled = b;
            config::save(&state.config);
            let playing = state.is_playing();
            playback_changed(state, b && playing)
        }
        LastfmMsg::Connect => {
            state.lastfm_busy = true;
            state.lastfm_status = None;
            result_task(
                lastfm::get_token(),
                |token| Msg::Lastfm(LastfmMsg::TokenReady(token)),
                |e| Msg::Lastfm(LastfmMsg::Err(e)),
            )
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
            result_task(
                async move { lastfm::get_session(&token).await },
                |(name, key)| Msg::Lastfm(LastfmMsg::SessionOk(name, key)),
                |e| Msg::Lastfm(LastfmMsg::Err(e)),
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
            let playing = state.is_playing();
            playback_changed(state, playing)
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

fn fetch_news_task(page: u32) -> Task<Msg> {
    result_task(
        api::fetch_news(page),
        |n| Msg::News(NewsMsg::Ok(n.data, n.meta.last_page)),
        |e| Msg::News(NewsMsg::Err(e)),
    )
}

fn fetch_history_task(page: u32) -> Task<Msg> {
    result_task(
        api::fetch_history(page),
        |h| Msg::History(HistoryMsg::Ok(h)),
        |e| Msg::History(HistoryMsg::Err(e)),
    )
}

fn fetch_ratings_task(range: String, page: u32) -> Task<Msg> {
    result_task(
        async move { api::fetch_ratings(&range, page).await },
        |r| Msg::Ratings(RatingsMsg::Ok(r.data, r.meta.last_page, r.meta.total)),
        |e| Msg::Ratings(RatingsMsg::Err(e)),
    )
}

fn fetch_favorites_task(token: String, page: u32) -> Task<Msg> {
    result_task(
        async move { api::fetch_favorites(&token, page).await },
        |f| Msg::Favorites(FavoritesMsg::Ok(f.data, f.meta.last_page, f.meta.total)),
        |e| Msg::Favorites(FavoritesMsg::Err(e)),
    )
}

fn update_news(state: &mut Plaza, msg: NewsMsg) -> Task<Msg> {
    match msg {
        NewsMsg::Ok(articles, pages) => {
            state.news.list = articles
                .into_iter()
                .map(state::ParsedNewsArticle::from)
                .collect();
            state.news.pager.loaded(pages, 0);
            Task::none()
        }
        NewsMsg::Err(e) => {
            state.news.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        NewsMsg::Page(msg) => state
            .news
            .pager
            .apply(msg)
            .map_or_else(Task::none, fetch_news_task),
    }
}

fn update_history(state: &mut Plaza, msg: HistoryMsg) -> Task<Msg> {
    match msg {
        HistoryMsg::Ok(resp) => {
            state.history.list = resp.data;
            state
                .history
                .pager
                .loaded(resp.meta.last_page, resp.meta.total);
            if let Some(dr) = resp.date_range {
                state.history.date_from = views::format_date(dr.from_date);
                state.history.date_to = views::format_date(dr.to_date);
            }
            Task::none()
        }
        HistoryMsg::Err(e) => {
            state.history.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        HistoryMsg::Page(msg) => state
            .history
            .pager
            .apply(msg)
            .map_or_else(Task::none, fetch_history_task),
    }
}

fn update_ratings(state: &mut Plaza, msg: RatingsMsg) -> Task<Msg> {
    match msg {
        RatingsMsg::Ok(songs, pages, total) => {
            state.ratings.list = songs;
            state.ratings.pager.loaded(pages, total);
            Task::none()
        }
        RatingsMsg::Err(e) => {
            state.ratings.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        RatingsMsg::Page(msg) => match state.ratings.pager.apply(msg) {
            Some(p) => fetch_ratings_task(state.ratings.range.clone(), p),
            None => Task::none(),
        },
        RatingsMsg::Range(range) => {
            state.ratings.range = range;
            state.ratings.list.clear();
            state.ratings.pager.goto(1);
            fetch_ratings_task(state.ratings.range.clone(), 1)
        }
    }
}

fn update_favorites(state: &mut Plaza, msg: FavoritesMsg) -> Task<Msg> {
    let favs = &mut state.favorites;
    match msg {
        FavoritesMsg::Ok(list, pages, total) => {
            favs.pager.loaded(pages, total);

            // Keep only thumbnails for the page being shown.
            let wanted: std::collections::HashSet<&str> =
                list.iter().filter_map(|f| f.song.thumb_url()).collect();
            favs.artwork.retain(|url, _| wanted.contains(url.as_str()));

            let art_tasks: Vec<Task<Msg>> = wanted
                .into_iter()
                .filter(|url| !favs.artwork.contains_key(*url))
                .map(|url| {
                    let url = url.to_string();
                    Task::perform(
                        async move {
                            let handle = api::fetch_artwork(&url, THUMB_PX).await;
                            (url, handle)
                        },
                        |(url, handle)| match handle {
                            Ok(h) => Msg::Favorites(FavoritesMsg::ArtworkOk(url, h)),
                            Err(_) => Msg::Noop,
                        },
                    )
                })
                .collect();

            favs.list = list;
            Task::batch(art_tasks)
        }
        FavoritesMsg::ArtworkOk(url, handle) => {
            favs.artwork.insert(url, handle);
            Task::none()
        }
        FavoritesMsg::Err(e) => {
            favs.pager.loading = false;
            state.error_msg = Some(e);
            Task::none()
        }
        FavoritesMsg::Page(msg) => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            match favs.pager.apply(msg) {
                Some(p) => {
                    favs.deleted.clear();
                    fetch_favorites_task(token, p)
                }
                None => Task::none(),
            }
        }
        FavoritesMsg::Delete(id) => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            result_task(
                async move { api::delete_favorite(&token, id).await },
                move |()| Msg::Favorites(FavoritesMsg::DeleteOk(id)),
                |e| Msg::Favorites(FavoritesMsg::DeleteErr(e)),
            )
        }
        FavoritesMsg::DeleteOk(id) => {
            if !favs.deleted.contains(&id) {
                favs.deleted.push(id);
            }
            Task::none()
        }
        FavoritesMsg::DeleteErr(e) => {
            state.error_msg = Some(e);
            Task::none()
        }
    }
}

fn update_export(state: &mut Plaza, msg: ExportMsg) -> Task<Msg> {
    match msg {
        ExportMsg::Start => {
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            state.export = state::ExportState {
                loading: true,
                ..Default::default()
            };
            result_task(
                async move { api::export_favorites(&token).await },
                |link| Msg::Export(ExportMsg::Ok(link)),
                |e| Msg::Export(ExportMsg::Err(e)),
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

fn update_profile_edit(state: &mut Plaza, msg: ProfileEditMsg) -> Task<Msg> {
    let edit = &mut state.profile_edit;
    match msg {
        ProfileEditMsg::Username(s) => edit.username = s,
        ProfileEditMsg::Email(s) => edit.email = s,
        ProfileEditMsg::CurrentPassword(s) => edit.current_password = s,
        ProfileEditMsg::Submit => {
            if edit.current_password.is_empty() {
                edit.error = Some("Current password is required.".into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            edit.loading = true;
            edit.error = None;
            let (current, username, email) = (
                edit.current_password.clone(),
                edit.username.clone(),
                edit.email.clone(),
            );
            return result_task(
                async move { api::update_profile(&token, &current, &username, &email).await },
                |()| Msg::ProfileEdit(ProfileEditMsg::Ok),
                |e| Msg::ProfileEdit(ProfileEditMsg::Err(e)),
            );
        }
        ProfileEditMsg::Ok => {
            edit.loading = false;
            if let Some(ref mut u) = state.user {
                u.username = edit.username.clone();
                u.email = edit.email.clone();
                if let Some(ref token) = state.auth_token {
                    state.config.session = Some(config::Session {
                        token: token.clone(),
                        user: u.clone(),
                    });
                    config::save(&state.config);
                }
            }
            edit.current_password.clear();
            state.error_msg = Some("Profile has been updated.".into());
            return close_windows_of(state, WinType::UserProfileEdit);
        }
        ProfileEditMsg::Err(e) => {
            edit.loading = false;
            edit.error = Some(e);
        }
    }
    Task::none()
}

fn update_password(state: &mut Plaza, msg: PasswordMsg) -> Task<Msg> {
    let pw = &mut state.password;
    match msg {
        PasswordMsg::Current(s) => pw.current_password = s,
        PasswordMsg::New(s) => pw.password = s,
        PasswordMsg::Repeat(s) => pw.password_repeat = s,
        PasswordMsg::Submit => {
            let problem = if pw.current_password.is_empty() {
                Some("Current password is required.")
            } else if pw.password.len() < 3 {
                Some("Password is too short.")
            } else if pw.password != pw.password_repeat {
                Some("Passwords do not match.")
            } else {
                None
            };
            if let Some(problem) = problem {
                pw.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            pw.loading = true;
            pw.error = None;
            let (current, new) = (pw.current_password.clone(), pw.password.clone());
            return result_task(
                async move { api::update_password(&token, &current, &new).await },
                |()| Msg::Password(PasswordMsg::Ok),
                |e| Msg::Password(PasswordMsg::Err(e)),
            );
        }
        PasswordMsg::Ok => {
            state.password = state::PasswordState::default();
            clear_session(state);
            state.error_msg = Some("Password updated. Please log in again.".into());
            return Task::batch([
                close_windows_of(state, WinType::UserPassword),
                close_windows_of(state, WinType::UserProfile),
            ]);
        }
        PasswordMsg::Err(e) => {
            pw.loading = false;
            pw.error = Some(e);
        }
    }
    Task::none()
}

fn update_delete(state: &mut Plaza, msg: DeleteMsg) -> Task<Msg> {
    let del = &mut state.delete;
    match msg {
        DeleteMsg::Password(s) => del.current_password = s,
        DeleteMsg::Confirm(b) => del.confirm = b,
        DeleteMsg::Submit => {
            let problem = if !del.confirm {
                Some("You must confirm account deletion.")
            } else if del.current_password.is_empty() {
                Some("Current password is required.")
            } else {
                None
            };
            if let Some(problem) = problem {
                del.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.auth_token.clone() else {
                return Task::none();
            };
            del.loading = true;
            del.error = None;
            let current = del.current_password.clone();
            return result_task(
                async move { api::delete_profile(&token, &current).await },
                |()| Msg::DeleteAccount(DeleteMsg::Ok),
                |e| Msg::DeleteAccount(DeleteMsg::Err(e)),
            );
        }
        DeleteMsg::Ok => {
            state.delete = state::DeleteState::default();
            clear_session(state);
            state.error_msg = Some("Your account has been deleted.".into());
            return Task::batch([
                close_windows_of(state, WinType::UserProfileDelete),
                close_windows_of(state, WinType::UserProfileEdit),
                close_windows_of(state, WinType::UserProfile),
            ]);
        }
        DeleteMsg::Err(e) => {
            del.loading = false;
            del.error = Some(e);
        }
    }
    Task::none()
}
