mod about;
mod bevel;
mod credits;
mod frame;
mod history;
mod news;
mod paint;
mod pixel;
mod player;
mod player_timer;
mod ratings;
mod settings;
mod song_info;
mod support;
mod user_favorites;
mod user_favorites_export;
mod user_login;
mod user_password;
mod user_profile;
mod user_profile_delete;
mod user_profile_edit;
mod user_register;
mod volume;
mod widgets;

use crate::message::Msg;
use crate::state::Plaza;
use crate::window::WindowKind;
use iced::Element;
use iced::window::Id;

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let kind = state.windows.get(&wid).copied();
    let content = match kind {
        None => player::view(state),
        Some(WindowKind::About) => about::view(wid),
        Some(WindowKind::Credits) => credits::view(wid),
        Some(WindowKind::History) => history::view(state, wid),
        Some(WindowKind::News) => news::view(state, wid),
        Some(WindowKind::PlayerTimer) => player_timer::view(state, wid),
        Some(WindowKind::Ratings) => ratings::view(state, wid),
        Some(WindowKind::Settings) => settings::view(state, wid),
        Some(WindowKind::SongInfo) => song_info::view(state, wid),
        Some(WindowKind::Support) => support::view(wid),
        Some(WindowKind::UserFavorites) => user_favorites::view(state, wid),
        Some(WindowKind::UserFavoritesExport) => user_favorites_export::view(state, wid),
        Some(WindowKind::UserLogin) => user_login::view(state, wid),
        Some(WindowKind::UserPassword) => user_password::view(state, wid),
        Some(WindowKind::UserProfile) => user_profile::view(state, wid),
        Some(WindowKind::UserProfileDelete) => user_profile_delete::view(state, wid),
        Some(WindowKind::UserProfileEdit) => user_profile_edit::view(state, wid),
        Some(WindowKind::UserRegister) => user_register::view(state, wid),
    };
    frame::frame(wid, kind, state.focused == Some(wid), content)
}
