#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod api;
mod audio;
mod config;
mod discord;
mod fonts;
mod heap;
mod lastfm;
mod net;
mod news;
mod state;
mod theme;
mod update;
mod views;

use audio::AudioPlayer;
use iced::{Font, Size, Subscription, Task};
use state::{AccountMsg, Msg, Plaza};
use std::sync::LazyLock;
use std::time::Duration;

static APP_ICON: LazyLock<iced::window::Icon> = LazyLock::new(|| {
    let img = image::load_from_memory(include_bytes!("assets/icons/favicon-32x32.png"))
        .expect("app icon is a valid PNG")
        .into_rgba8();
    let (w, h) = img.dimensions();
    iced::window::icon::from_rgba(img.into_raw(), w, h).expect("app icon is valid RGBA")
});

static DEV_MODE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("NIGHTWAVE_DEV").is_some());
static BENCH_MODE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("NIGHTWAVE_BENCH").is_some());

pub fn window_settings(size: Size, resizable: bool) -> iced::window::Settings {
    #[cfg(target_os = "linux")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        application_id: "nightwave-plaza".into(),
        ..Default::default()
    };
    #[cfg(not(target_os = "linux"))]
    let platform_specific = iced::window::settings::PlatformSpecific::default();

    iced::window::Settings {
        size,
        resizable: resizable || *DEV_MODE,
        decorations: *DEV_MODE,
        icon: Some(APP_ICON.clone()),
        platform_specific,
        ..Default::default()
    }
}

fn main() -> iced::Result {
    heap::limit_arenas();
    fonts::install();
    iced::daemon(boot, update::update, views::view)
        .title(title)
        .subscription(subscription)
        .theme(|_: &Plaza, _| theme::app_theme())
        .default_font(Font {
            family: iced::font::Family::Name("Tahoma"),
            ..Font::DEFAULT
        })
        .run()
}

fn boot() -> (Plaza, Task<Msg>) {
    let (events_tx, events_rx) = futures::channel::mpsc::unbounded();
    let player = AudioPlayer::new(events_tx);
    let (main_id, open) = iced::window::open(window_settings(Size::new(450.0, 218.0), false));
    let state = Plaza::new(main_id, player, config::load());

    let check_session = match state.token() {
        Some(token) => Task::perform(api::get_me(token), |r| Msg::Account(AccountMsg::Checked(r))),
        None => Task::none(),
    };
    (
        state,
        Task::batch([
            open.discard(),
            Task::run(events_rx, |msg| msg),
            update::fetch_status(),
            check_session,
        ]),
    )
}

fn title(state: &Plaza, wid: iced::window::Id) -> String {
    if let Some(wt) = state.child_windows.get(&wid) {
        return wt.title().to_string();
    }
    let song = &state.status.song;
    if song.artist.is_empty() {
        "Nightwave Plaza".into()
    } else {
        format!("{} - {} - Nightwave Plaza", song.artist, song.title)
    }
}

fn subscription(state: &Plaza) -> Subscription<Msg> {
    let playing = state.player.is_playing();

    let clock_visible = state.main_focused() && (playing || state.time_notice.is_some());
    let tick = clock_visible || state.timer.until.is_some();

    let song = &state.status.song;
    let remaining = if playing && song.length > 0.0 {
        Duration::from_secs_f64((song.length - song.position).max(0.0)) + Duration::from_secs(1)
    } else {
        Duration::MAX
    };
    let refresh = remaining.clamp(Duration::from_secs(2), Duration::from_secs(30));

    let mut subs = vec![
        iced::time::every(refresh).map(|_| Msg::Refresh),
        iced::window::close_events().map(Msg::WinClosed),
        iced::event::listen_with(|event, status, id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                ..
            }) if status == iced::event::Status::Ignored => Some(Msg::SpaceToggle(id)),
            iced::Event::Window(iced::window::Event::Opened { .. }) if *BENCH_MODE => {
                Some(Msg::WinClosed(id))
            }
            iced::Event::Window(iced::window::Event::Focused) => Some(Msg::WinFocus(id, true)),
            iced::Event::Window(iced::window::Event::Unfocused) => Some(Msg::WinFocus(id, false)),
            _ => None,
        }),
    ];
    if tick {
        subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Msg::Tick));
    }
    if *DEV_MODE {
        subs.push(iced::window::resize_events().map(|(id, size)| Msg::WinResized(id, size)));
    }
    Subscription::batch(subs)
}
