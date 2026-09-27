use crate::api::{self, RatingsRange, Status, User};
use crate::lastfm;
use crate::net;
use crate::state::SongReaction;
use crate::window::WindowKind;
use iced::widget::image;
use iced::window::Id;

#[derive(Debug, Clone)]
pub enum Msg {
    Refresh,
    Status(api::Result<Status>),
    Tick,
    TogglePlay,
    StreamChanged,
    #[cfg_attr(
        target_os = "windows",
        expect(dead_code, reason = "media controls are not wired up on Windows")
    )]
    Media(souvlaki::MediaControlEvent),
    Volume(f32),
    Artwork(api::Result<image::Handle>),
    React,
    Reacted(SongReaction, api::Result<u32>),

    History(HistoryMsg),
    Ratings(RatingsMsg),
    SongInfo(SongInfoMsg),
    Login(LoginMsg),
    Register(RegisterMsg),
    News(NewsMsg),
    Favorites(FavoritesMsg),
    Export(ExportMsg),
    ProfileEdit(ProfileEditMsg),
    Password(PasswordMsg),
    DeleteAccount(DeleteAccountMsg),
    Account(AccountMsg),
    Lastfm(LastfmMsg),
    DiscordEnabled(bool),
    Timer(TimerMsg),

    OpenWindow(WindowKind),
    CloseWindow(Id),
    WindowClosed(Id),
    WindowFocused(Id),
    WindowUnfocused(Id),
    WindowResized(Id, iced::Size),
    MinimizeWindow(Id),
    DragWindow(Id),
    SpacePressed(Id),
    OpenUrl(String),
    DismissAlert,
}

#[derive(Debug, Clone)]
pub enum PageMsg {
    Go(u32),
    Input(String),
    Submit,
}

#[derive(Debug, Clone)]
pub enum HistoryMsg {
    Loaded(api::Result<api::HistoryResponse>),
    Page(PageMsg),
}

#[derive(Debug, Clone)]
pub enum RatingsMsg {
    Loaded(api::Result<api::Page<api::RatingEntry>>),
    Page(PageMsg),
    Range(RatingsRange),
}

#[derive(Debug, Clone)]
pub enum NewsMsg {
    Loaded(api::Result<api::Page<api::NewsArticle>>),
    Page(PageMsg),
}

#[derive(Debug, Clone)]
pub enum SongInfoMsg {
    Open(String),
    Loaded(api::Result<api::SongResponse>),
    Artwork(api::Result<image::Handle>),
    ToggleFavorite,
    FavoriteAdded(api::Result<u64>),
    FavoriteRemoved(api::Result<()>),
}

#[derive(Debug, Clone)]
pub enum FavoritesMsg {
    Loaded(api::Result<api::Page<api::FavoriteEntry>>),
    Artwork(String, api::Result<image::Handle>),
    Page(PageMsg),
    Remove(u64),
    Removed(u64, api::Result<()>),
}

#[derive(Debug, Clone)]
pub enum ExportMsg {
    Start,
    Done(api::Result<String>),
}

#[derive(Debug, Clone)]
pub enum AccountMsg {
    Checked(api::Result<User>),
    Logout,
    LoggedOut(api::Result<()>),
    Stats(api::Result<api::UserStats>),
}

#[derive(Debug, Clone)]
pub enum LoginMsg {
    Username(String),
    Password(String),
    Remember(bool),
    Submit,
    Done(api::Result<api::LoginResponse>),
}

#[derive(Debug, Clone)]
pub enum RegisterMsg {
    Username(String),
    Email(String),
    Password(String),
    PasswordRepeat(String),
    Submit,
    Done(api::Result<User>),
}

#[derive(Debug, Clone)]
pub enum ProfileEditMsg {
    Username(String),
    Email(String),
    CurrentPassword(String),
    Submit,
    Done(api::Result<()>),
}

#[derive(Debug, Clone)]
pub enum PasswordMsg {
    Current(String),
    New(String),
    Repeat(String),
    Submit,
    Done(api::Result<()>),
}

#[derive(Debug, Clone)]
pub enum DeleteAccountMsg {
    Password(String),
    Confirm(bool),
    Submit,
    Done(api::Result<()>),
}

#[derive(Debug, Clone)]
pub enum LastfmMsg {
    SetEnabled(bool),
    Connect,
    Token(Result<String, net::Error>),
    Finish,
    Session(Result<lastfm::Session, net::Error>),
    Disconnect,
}

#[derive(Debug, Clone)]
pub enum TimerMsg {
    Input(String),
    Add(i32),
    Start,
    Stop,
}
