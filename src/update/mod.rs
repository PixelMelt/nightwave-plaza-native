mod account;
mod lists;
mod playback;
mod settings;
mod song_info;

use crate::api;
use crate::message::Msg;
use crate::platform;
use crate::state::{DeleteAccountForm, ExportState, PasswordForm, Plaza, ProfileEditForm};
use crate::window::WindowKind;
use iced::widget::image;
use iced::{Task, window};

pub use playback::fetch_status;

const COVER_PX: u32 = 256;
const THUMB_PX: u32 = 128;

pub fn update(state: &mut Plaza, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Refresh => {
            platform::trim_heap();
            fetch_status()
        }
        Msg::Status(result) => playback::status(state, result),
        Msg::Tick => playback::tick(state),
        Msg::TogglePlay => playback::toggle(state),
        Msg::StreamChanged => Task::none(),
        Msg::Media(event) => playback::media(state, &event),
        Msg::Volume(volume) => {
            playback::set_volume(state, volume);
            Task::none()
        }
        Msg::Artwork(result) => {
            state.artwork = artwork_or_log(result);
            Task::none()
        }
        Msg::React => playback::react(state),
        Msg::Reacted(reaction, result) => {
            playback::reacted(state, reaction, result);
            Task::none()
        }

        Msg::History(msg) => lists::history(state, msg),
        Msg::Ratings(msg) => lists::ratings(state, msg),
        Msg::News(msg) => lists::news(state, msg),
        Msg::Favorites(msg) => lists::favorites(state, msg),
        Msg::Export(msg) => lists::export(state, msg),
        Msg::SongInfo(msg) => song_info::update(state, msg),
        Msg::Account(msg) => account::update(state, msg),
        Msg::Login(msg) => account::login(state, msg),
        Msg::Register(msg) => account::register(state, msg),
        Msg::ProfileEdit(msg) => account::profile_edit(state, msg),
        Msg::Password(msg) => account::password(state, msg),
        Msg::DeleteAccount(msg) => account::delete(state, msg),
        Msg::Lastfm(msg) => settings::lastfm(state, msg),
        Msg::DiscordEnabled(enabled) => {
            settings::set_discord_enabled(state, enabled);
            Task::none()
        }
        Msg::Timer(msg) => {
            settings::timer(state, msg);
            Task::none()
        }

        Msg::OpenWindow(kind) => open_window(state, kind),
        Msg::CloseWindow(id) => window::close(id),
        Msg::WindowClosed(id) => {
            if id == state.main_window {
                std::process::exit(0);
            }
            state.windows.remove(&id);
            if state.focused == Some(id) {
                state.focused = None;
            }
            Task::none()
        }
        Msg::WindowFocused(id) => {
            state.focused = Some(id);
            Task::none()
        }
        Msg::WindowUnfocused(id) => {
            if state.focused == Some(id) {
                state.focused = None;
            }
            Task::none()
        }
        Msg::WindowResized(id, size) => {
            let label = match state.windows.get(&id) {
                Some(kind) => format!("Self::{kind:?}"),
                None => "Main".into(),
            };
            eprintln!(
                "[winsize] {label} => ({:.1}, {:.1}),",
                size.width, size.height
            );
            Task::none()
        }
        Msg::MinimizeWindow(id) => window::minimize(id, true),
        Msg::DragWindow(id) => window::drag(id),
        Msg::SpacePressed(id) => {
            if id == state.main_window {
                playback::toggle(state)
            } else {
                Task::none()
            }
        }
        Msg::OpenUrl(url) => {
            platform::open_url(&url);
            Task::none()
        }
        Msg::DismissAlert => {
            state.alert = None;
            Task::none()
        }
    }
}

pub fn open_window(state: &mut Plaza, kind: WindowKind) -> Task<Msg> {
    if let Some(id) = state.window_of(kind) {
        return window::gain_focus(id);
    }
    let (id, opened) = window::open(kind.settings());
    state.windows.insert(id, kind);

    let prepare = match kind {
        WindowKind::History if state.history.list.is_empty() => lists::load_history(state, 1),
        WindowKind::Ratings if state.ratings.list.is_empty() => lists::load_ratings(state, 1),
        WindowKind::News if state.news.list.is_empty() => lists::load_news(state, 1),
        WindowKind::UserFavorites => lists::load_favorites(state, 1),
        WindowKind::UserProfile => account::load_stats(state),
        WindowKind::UserFavoritesExport => {
            state.export = ExportState::default();
            Task::none()
        }
        WindowKind::UserProfileEdit => {
            state.profile_edit = ProfileEditForm::for_user(state.user());
            Task::none()
        }
        WindowKind::UserPassword => {
            state.password = PasswordForm::default();
            Task::none()
        }
        WindowKind::UserProfileDelete => {
            state.delete_account = DeleteAccountForm::default();
            Task::none()
        }
        _ => Task::none(),
    };
    Task::batch([opened.discard(), prepare])
}

fn close_windows(state: &Plaza, kinds: &[WindowKind]) -> Task<Msg> {
    let ids: Vec<_> = state
        .windows
        .iter()
        .filter(|&(_, kind)| kinds.contains(kind))
        .map(|(&id, _)| id)
        .collect();
    Task::batch(ids.into_iter().map(window::close))
}

fn artwork_or_log(result: api::Result<image::Handle>) -> Option<image::Handle> {
    result
        .map_err(|e| eprintln!("Artwork download failed: {e}"))
        .ok()
}
