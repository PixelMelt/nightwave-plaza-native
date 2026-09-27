use crate::api::{self, RatingsRange, Status, User};
use crate::audio::AudioPlayer;
use crate::config::{Config, Session};
use crate::discord::Discord;
use crate::lastfm::Scrobble;
use crate::news::Article;
use iced::widget::image;
use iced::window::Id;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const DEFAULT_VOLUME: f32 = 50.0;
pub const NOTICE_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WinType {
    History,
    About,
    Ratings,
    Support,
    SongInfo,
    UserLogin,
    UserProfile,
    UserRegister,
    Credits,
    News,
    UserFavorites,
    UserFavoritesExport,
    UserProfileEdit,
    UserPassword,
    UserProfileDelete,
    PlayerTimer,
    Settings,
}

impl WinType {
    pub fn size(self) -> iced::Size {
        let (w, h) = match self {
            WinType::History => (400.0, 630.0),
            WinType::About => (380.0, 518.0),
            WinType::Ratings => (440.0, 630.0),
            WinType::Support => (450.0, 270.0),
            WinType::SongInfo => (360.0, 225.0),
            WinType::UserLogin => (480.0, 150.0),
            WinType::UserProfile => (290.0, 250.0),
            WinType::UserRegister => (430.0, 240.0),
            WinType::Credits => (420.0, 195.0),
            WinType::News => (350.0, 300.0),
            WinType::UserFavorites => (450.0, 600.0),
            WinType::UserFavoritesExport => (320.0, 175.0),
            WinType::UserProfileEdit => (290.0, 293.0),
            WinType::UserPassword => (280.0, 232.0),
            WinType::UserProfileDelete => (340.0, 296.0),
            WinType::PlayerTimer => (280.0, 150.0),
            WinType::Settings => (360.0, 340.0),
        };
        iced::Size::new(w, h)
    }

    pub fn resizable(self) -> bool {
        matches!(
            self,
            WinType::History | WinType::Ratings | WinType::News | WinType::UserFavorites
        )
    }

    pub fn title(self) -> &'static str {
        match self {
            WinType::History => "Play History",
            WinType::About => "About",
            WinType::Ratings => "Ratings",
            WinType::Support => "Support Us",
            WinType::SongInfo => "Song Info",
            WinType::UserLogin => "Log In",
            WinType::UserProfile => "My Profile",
            WinType::UserRegister => "Registration",
            WinType::Credits => "Credits",
            WinType::News => "News",
            WinType::UserFavorites => "My Favorites",
            WinType::UserFavoritesExport => "Export Favorites",
            WinType::UserProfileEdit => "Edit Profile",
            WinType::UserPassword => "Change Password",
            WinType::UserProfileDelete => "Delete Account",
            WinType::PlayerTimer => "Sleep Timer",
            WinType::Settings => "Settings",
        }
    }
}

pub fn digits_input(field: &mut String, s: String) {
    if s.chars().all(|c| c.is_ascii_digit()) {
        *field = s;
    }
}

pub struct Pager {
    pub page: u32,
    pub pages: u32,
    pub total: u32,
    pub input: String,
    pub loading: bool,
}

impl Default for Pager {
    fn default() -> Self {
        Self {
            page: 1,
            pages: 1,
            total: 0,
            input: "1".into(),
            loading: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PageMsg {
    Go(u32),
    Input(String),
    Submit,
}

impl Pager {
    pub fn goto(&mut self, page: u32) {
        self.page = page;
        self.input = page.to_string();
        self.loading = true;
    }

    pub fn loaded(&mut self, meta: &api::PaginatedMeta) {
        self.loading = false;
        self.pages = meta.last_page;
        self.total = meta.total;
        self.input = self.page.to_string();
    }

    pub fn apply(&mut self, msg: PageMsg) -> Option<u32> {
        match msg {
            PageMsg::Go(p) => Some(p),
            PageMsg::Input(s) => {
                digits_input(&mut self.input, s);
                None
            }
            PageMsg::Submit => {
                let page = self
                    .input
                    .parse::<u32>()
                    .ok()
                    .map(|p| p.clamp(1, self.pages.max(1)))
                    .filter(|&p| p != self.page);
                if page.is_none() {
                    self.input = self.page.to_string();
                }
                page
            }
        }
    }
}

#[derive(Default)]
pub struct HistoryState {
    pub list: Vec<api::HistoryEntry>,
    pub pager: Pager,
    pub date_range: Option<api::DateRange>,
}

pub struct RatingsState {
    pub list: Vec<api::RatingEntry>,
    pub pager: Pager,
    pub range: RatingsRange,
}

impl Default for RatingsState {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            pager: Pager::default(),
            range: RatingsRange::AllTime,
        }
    }
}

#[derive(Default)]
pub struct SongInfoState {
    pub data: Option<api::SongResponse>,
    pub error: Option<String>,
    pub artwork: Option<image::Handle>,
    pub favorite_id: Option<u64>,
    pub fav_sending: bool,
}

#[derive(Default)]
pub struct LoginState {
    pub username: String,
    pub password: String,
    pub remember: bool,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct RegisterState {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_repeat: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct NewsState {
    pub list: Vec<Article>,
    pub pager: Pager,
}

#[derive(Default)]
pub struct FavoritesState {
    pub list: Vec<api::FavoriteEntry>,
    pub deleted: Vec<u64>,
    pub pager: Pager,
    pub artwork: HashMap<String, image::Handle>,
}

#[derive(Default)]
pub struct ExportState {
    pub loading: bool,
    pub link: Option<String>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct ProfileEditState {
    pub username: String,
    pub email: String,
    pub current_password: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct PasswordState {
    pub current_password: String,
    pub password: String,
    pub password_repeat: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct DeleteState {
    pub current_password: String,
    pub confirm: bool,
    pub loading: bool,
    pub error: Option<String>,
}

pub struct TimerState {
    pub minutes_input: String,
    pub until: Option<Instant>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            minutes_input: "20".into(),
            until: None,
        }
    }
}

#[derive(Default)]
pub struct LastfmState {
    pub token: Option<String>,
    pub busy: bool,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Reaction {
    pub song_id: String,
    pub rate: u8,
}

impl Reaction {
    pub fn rate_for(&self, song_id: &str) -> u8 {
        if !song_id.is_empty() && self.song_id == song_id {
            self.rate
        } else {
            0
        }
    }
}

pub struct Plaza {
    pub main_window: Id,
    pub child_windows: HashMap<Id, WinType>,
    pub focused: Option<Id>,

    pub status: Status,
    pub status_at: Instant,
    pub player: AudioPlayer,
    pub volume: f32,
    pub artwork: Option<image::Handle>,
    pub artwork_url: String,
    pub time_notice: Option<(String, Instant)>,
    pub error_msg: Option<String>,

    pub session: Option<Session>,
    pub user_stats: Option<api::UserStats>,
    pub stats_loading: bool,
    pub reaction: Reaction,

    pub history: HistoryState,
    pub ratings: RatingsState,
    pub song_info: SongInfoState,
    pub login: LoginState,
    pub register: RegisterState,
    pub news: NewsState,
    pub favorites: FavoritesState,
    pub export: ExportState,
    pub profile_edit: ProfileEditState,
    pub password: PasswordState,
    pub delete: DeleteState,
    pub timer: TimerState,
    pub lastfm: LastfmState,

    pub config: Config,
    pub scrobble: Option<Scrobble>,
    pub discord: Discord,
}

impl Plaza {
    pub fn new(main_window: Id, player: AudioPlayer, config: Config) -> Self {
        player.set_volume(DEFAULT_VOLUME / 100.0);
        Self {
            main_window,
            child_windows: HashMap::new(),
            focused: Some(main_window),
            status: Status::default(),
            status_at: Instant::now(),
            player,
            volume: DEFAULT_VOLUME,
            artwork: None,
            artwork_url: String::new(),
            time_notice: Some(("Welcome back!".into(), Instant::now() + NOTICE_DURATION)),
            error_msg: None,
            session: config.session.clone(),
            user_stats: None,
            stats_loading: false,
            reaction: Reaction::default(),
            history: HistoryState::default(),
            ratings: RatingsState::default(),
            song_info: SongInfoState::default(),
            login: LoginState::default(),
            register: RegisterState::default(),
            news: NewsState::default(),
            favorites: FavoritesState::default(),
            export: ExportState::default(),
            profile_edit: ProfileEditState::default(),
            password: PasswordState::default(),
            delete: DeleteState::default(),
            timer: TimerState::default(),
            lastfm: LastfmState::default(),
            config,
            scrobble: None,
            discord: Discord::spawn(),
        }
    }

    pub fn main_focused(&self) -> bool {
        self.focused == Some(self.main_window)
    }

    pub fn window_of(&self, wt: WinType) -> Option<Id> {
        self.child_windows
            .iter()
            .find(|(_, &t)| t == wt)
            .map(|(&id, _)| id)
    }

    pub fn user(&self) -> Option<&User> {
        self.session.as_ref().map(|s| &s.user)
    }

    pub fn token(&self) -> Option<String> {
        self.session.as_ref().map(|s| s.token.clone())
    }

    pub fn song_position(&self) -> f64 {
        let song = &self.status.song;
        (song.position + self.status_at.elapsed().as_secs_f64()).min(song.length)
    }
}

#[derive(Debug, Clone)]
pub enum HistoryMsg {
    Loaded(Result<api::HistoryResponse, String>),
    Page(PageMsg),
}

#[derive(Debug, Clone)]
pub enum RatingsMsg {
    Loaded(Result<api::Paginated<api::RatingEntry>, String>),
    Page(PageMsg),
    Range(RatingsRange),
}

#[derive(Debug, Clone)]
pub enum SongInfoMsg {
    Open(String),
    Loaded(Result<api::SongResponse, String>),
    Artwork(Result<image::Handle, String>),
    ToggleFavorite,
    FavoriteAdded(Result<u64, String>),
    FavoriteRemoved(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum LoginMsg {
    Username(String),
    Password(String),
    Remember(bool),
    Submit,
    Done(Result<api::LoginResponse, String>),
}

#[derive(Debug, Clone)]
pub enum RegisterMsg {
    Username(String),
    Email(String),
    Password(String),
    PasswordRepeat(String),
    Submit,
    Done(Result<User, String>),
}

#[derive(Debug, Clone)]
pub enum NewsMsg {
    Loaded(Result<api::Paginated<api::NewsArticle>, String>),
    Page(PageMsg),
}

#[derive(Debug, Clone)]
pub enum FavoritesMsg {
    Loaded(Result<api::Paginated<api::FavoriteEntry>, String>),
    Artwork(String, Result<image::Handle, String>),
    Page(PageMsg),
    Delete(u64),
    Deleted(u64, Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum ExportMsg {
    Start,
    Done(Result<String, String>),
}

#[derive(Debug, Clone)]
pub enum ProfileEditMsg {
    Username(String),
    Email(String),
    CurrentPassword(String),
    Submit,
    Done(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum PasswordMsg {
    Current(String),
    New(String),
    Repeat(String),
    Submit,
    Done(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum DeleteMsg {
    Password(String),
    Confirm(bool),
    Submit,
    Done(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum AccountMsg {
    Checked(Result<User, api::Error>),
    Logout,
    LoggedOut(Result<(), String>),
    Stats(Result<api::UserStats, String>),
}

#[derive(Debug, Clone)]
pub enum LastfmMsg {
    ToggleEnabled(bool),
    Connect,
    Token(Result<String, String>),
    Finish,
    Session(Result<(String, String), String>),
    Disconnect,
}

#[derive(Debug, Clone)]
pub enum TimerMsg {
    Input(String),
    Add(i32),
    Start,
    Stop,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Refresh,
    Status(Result<Status, String>),
    Tick,
    TogglePlay,
    StreamChanged,
    Media(souvlaki::MediaControlEvent),
    Volume(f32),
    Artwork(Result<image::Handle, String>),
    React,
    Reacted(Reaction, Result<u32, String>),

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
    DeleteAccount(DeleteMsg),
    Account(AccountMsg),
    Lastfm(LastfmMsg),
    DiscordEnabled(bool),
    Timer(TimerMsg),

    OpenWin(WinType),
    CloseWin(Id),
    WinClosed(Id),
    WinFocus(Id, bool),
    WinResized(Id, iced::Size),
    MinimizeWin(Id),
    DragWin(Id),
    SpaceToggle(Id),
    OpenUrl(String),
    DismissErr,
}
