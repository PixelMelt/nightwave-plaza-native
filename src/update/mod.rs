mod account;
mod lists;
mod playback;
mod settings;

use crate::state::{Msg, Plaza, WinType};
use iced::window::Id;
use iced::Task;
use std::future::Future;

pub use playback::fetch_status;

fn task<T, E>(
    fut: impl Future<Output = Result<T, E>> + Send + 'static,
    wrap: impl Fn(Result<T, String>) -> Msg + Send + 'static,
) -> Task<Msg>
where
    T: Send + 'static,
    E: Into<String> + Send + 'static,
{
    Task::perform(fut, move |r| wrap(r.map_err(Into::into)))
}

pub fn update(state: &mut Plaza, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Refresh => {
            crate::heap::trim();
            fetch_status()
        }
        Msg::Status(result) => playback::status(state, result),
        Msg::Tick => playback::tick(state),
        Msg::TogglePlay => playback::toggle(state),
        Msg::StreamChanged => Task::none(),
        Msg::Media(event) => playback::media(state, event),
        Msg::Volume(v) => playback::volume(state, v),
        Msg::Artwork(result) => {
            state.artwork = playback::artwork(result);
            Task::none()
        }
        Msg::React => playback::react(state),
        Msg::Reacted(reaction, result) => playback::reacted(state, reaction, result),

        Msg::History(msg) => lists::history(state, msg),
        Msg::Ratings(msg) => lists::ratings(state, msg),
        Msg::News(msg) => lists::news(state, msg),
        Msg::Favorites(msg) => lists::favorites(state, msg),
        Msg::Export(msg) => lists::export(state, msg),
        Msg::SongInfo(msg) => lists::song_info(state, msg),
        Msg::Account(msg) => account::update(state, msg),
        Msg::Login(msg) => account::login(state, msg),
        Msg::Register(msg) => account::register(state, msg),
        Msg::ProfileEdit(msg) => account::profile_edit(state, msg),
        Msg::Password(msg) => account::password(state, msg),
        Msg::DeleteAccount(msg) => account::delete(state, msg),
        Msg::Lastfm(msg) => settings::lastfm(state, msg),
        Msg::DiscordEnabled(enabled) => settings::discord(state, enabled),
        Msg::Timer(msg) => settings::timer(state, msg),

        Msg::OpenWin(wt) => open(state, wt),
        Msg::CloseWin(id) => iced::window::close(id),
        Msg::WinClosed(id) => {
            if id == state.main_window {
                quit();
            }
            state.child_windows.remove(&id);
            if state.focused == Some(id) {
                state.focused = None;
            }
            Task::none()
        }
        Msg::WinFocus(id, focused) => {
            if focused {
                state.focused = Some(id);
            } else if state.focused == Some(id) {
                state.focused = None;
            }
            Task::none()
        }
        Msg::WinResized(id, size) => {
            let label = match state.child_windows.get(&id) {
                Some(wt) => format!("WinType::{wt:?}"),
                None => "Main".to_string(),
            };
            eprintln!(
                "[winsize] {label} => ({:.1}, {:.1}),",
                size.width, size.height
            );
            Task::none()
        }
        Msg::MinimizeWin(id) => iced::window::minimize(id, true),
        Msg::DragWin(id) => iced::window::drag(id),
        Msg::SpaceToggle(id) if id == state.main_window => playback::toggle(state),
        Msg::SpaceToggle(_) => Task::none(),
        Msg::OpenUrl(url) => {
            open_url(&url);
            Task::none()
        }
        Msg::DismissErr => {
            state.error_msg = None;
            Task::none()
        }
    }
}

pub fn open(state: &mut Plaza, wt: WinType) -> Task<Msg> {
    if let Some(id) = state.window_of(wt) {
        return iced::window::gain_focus(id);
    }
    let (id, opened) = iced::window::open(crate::window_settings(wt.size(), wt.resizable()));
    state.child_windows.insert(id, wt);

    let prepare = match wt {
        WinType::History if state.history.list.is_empty() => lists::load_history(state, 1),
        WinType::Ratings if state.ratings.list.is_empty() => lists::load_ratings(state, 1),
        WinType::News if state.news.list.is_empty() => lists::load_news(state, 1),
        WinType::UserFavorites => lists::load_favorites(state, 1),
        WinType::UserProfile => account::load_stats(state),
        WinType::UserFavoritesExport => {
            state.export = Default::default();
            Task::none()
        }
        WinType::UserProfileEdit => {
            let (username, email) = state
                .user()
                .map(|u| (u.username.clone(), u.email.clone()))
                .unwrap_or_default();
            state.profile_edit = crate::state::ProfileEditState {
                username,
                email,
                ..Default::default()
            };
            Task::none()
        }
        WinType::UserPassword => {
            state.password = Default::default();
            Task::none()
        }
        WinType::UserProfileDelete => {
            state.delete = Default::default();
            Task::none()
        }
        _ => Task::none(),
    };
    Task::batch([opened.discard(), prepare])
}

fn close_windows_of(state: &Plaza, wt: WinType) -> Task<Msg> {
    let ids: Vec<Id> = state
        .child_windows
        .iter()
        .filter(|(_, &t)| t == wt)
        .map(|(&id, _)| id)
        .collect();
    Task::batch(ids.into_iter().map(iced::window::close))
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

fn quit() -> ! {
    std::process::exit(0)
}
