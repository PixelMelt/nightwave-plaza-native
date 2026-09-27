#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod api;
mod article;
mod audio;
mod config;
mod discord;
mod fonts;
mod lastfm;
mod media_controls;
mod message;
mod net;
mod platform;
mod state;
mod theme;
mod update;
mod views;
mod window;

use iced::keyboard::{self, key};
use iced::{Event, Font, Subscription, Task, event};
use message::{AccountMsg, Msg};
use state::Plaza;
use std::sync::LazyLock;
use std::time::Duration;

static BENCH_MODE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("NIGHTWAVE_BENCH").is_some());

fn main() -> iced::Result {
    platform::limit_malloc_arenas();
    fonts::install();
    iced::daemon(boot, update::update, views::view)
        .title(title)
        .subscription(subscription)
        .theme(|_: &Plaza, _| theme::app_theme())
        .default_font(Font::with_name("Tahoma"))
        .run()
}

fn boot() -> (Plaza, Task<Msg>) {
    let (events_tx, events_rx) = futures::channel::mpsc::unbounded();
    let (main_window, open) = iced::window::open(window::settings(window::MAIN_SIZE, false));
    let state = Plaza::new(main_window, config::load(), events_tx);

    let check_session = match state.token() {
        Some(token) => Task::perform(api::fetch_me(token), |r| {
            Msg::Account(AccountMsg::Checked(r))
        }),
        None => Task::none(),
    };
    let tasks = Task::batch([
        open.discard(),
        Task::run(events_rx, |msg| msg),
        update::fetch_status(),
        check_session,
    ]);
    (state, tasks)
}

fn title(state: &Plaza, id: iced::window::Id) -> String {
    if let Some(kind) = state.windows.get(&id) {
        return kind.title().into();
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

    let song = &state.status.song;
    let until_song_ends = if playing && song.length > 0.0 {
        Duration::from_secs_f64((song.length - song.position).max(0.0)) + Duration::from_secs(1)
    } else {
        Duration::MAX
    };
    let refresh = until_song_ends.clamp(Duration::from_secs(2), Duration::from_secs(30));

    let mut subscriptions = vec![
        iced::time::every(refresh).map(|_| Msg::Refresh),
        iced::window::close_events().map(Msg::WindowClosed),
        event::listen_with(|event, status, id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Space),
                ..
            }) if status == event::Status::Ignored => Some(Msg::SpacePressed(id)),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Enter | key::Named::Escape),
                ..
            }) if status == event::Status::Ignored => Some(Msg::DismissPressed(id)),
            Event::Window(iced::window::Event::Opened { .. }) if *BENCH_MODE => {
                Some(Msg::WindowClosed(id))
            }
            Event::Window(iced::window::Event::Focused) => Some(Msg::WindowFocused(id)),
            Event::Window(iced::window::Event::Unfocused) => Some(Msg::WindowUnfocused(id)),
            _ => None,
        }),
    ];

    let clock_visible = state.is_main_focused() && (playing || state.notice.is_some());
    if clock_visible || state.timer.until.is_some() {
        subscriptions.push(iced::time::every(Duration::from_secs(1)).map(|_| Msg::Tick));
    }
    if *window::DEV_MODE {
        subscriptions
            .push(iced::window::resize_events().map(|(id, size)| Msg::WindowResized(id, size)));
    }
    Subscription::batch(subscriptions)
}
