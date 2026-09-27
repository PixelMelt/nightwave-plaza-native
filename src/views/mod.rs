mod about;
mod bevel;
mod credits;
mod frame;
mod history;
mod news;
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

use crate::state::{Msg, Plaza, WinType};
use iced::window::Id;
use iced::Element;
use widgets::*;

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let wt = state.child_windows.get(&wid).copied();
    let content = match wt {
        None => player::view(state),
        Some(WinType::History) => history::view(state, wid),
        Some(WinType::About) => about::view(wid),
        Some(WinType::Support) => support::view(wid),
        Some(WinType::Ratings) => ratings::view(state, wid),
        Some(WinType::SongInfo) => song_info::view(state, wid),
        Some(WinType::UserLogin) => user_login::view(state, wid),
        Some(WinType::UserProfile) => user_profile::view(state, wid),
        Some(WinType::UserRegister) => user_register::view(state, wid),
        Some(WinType::Credits) => credits::view(wid),
        Some(WinType::News) => news::view(state, wid),
        Some(WinType::UserFavorites) => user_favorites::view(state, wid),
        Some(WinType::UserFavoritesExport) => user_favorites_export::view(state, wid),
        Some(WinType::UserProfileEdit) => user_profile_edit::view(state, wid),
        Some(WinType::UserPassword) => user_password::view(state, wid),
        Some(WinType::UserProfileDelete) => user_profile_delete::view(state, wid),
        Some(WinType::PlayerTimer) => player_timer::view(state, wid),
        Some(WinType::Settings) => settings::view(state, wid),
    };
    frame::frame(wid, wt, state.focused == Some(wid), content)
}
