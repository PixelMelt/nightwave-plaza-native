use crate::api::{self, HistoryEntry, RatingEntry, Status};
use crate::audio::AudioPlayer;
use iced::widget::image;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

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
    pub fn size(&self) -> iced::Size {
        match self {
            WinType::History => iced::Size::new(400.0, 630.0),
            WinType::About => iced::Size::new(380.0, 518.0),
            WinType::Ratings => iced::Size::new(440.0, 630.0),
            WinType::Support => iced::Size::new(450.0, 270.0),
            WinType::SongInfo => iced::Size::new(360.0, 225.0),
            WinType::UserLogin => iced::Size::new(480.0, 150.0),
            WinType::UserProfile => iced::Size::new(290.0, 250.0),
            WinType::UserRegister => iced::Size::new(430.0, 240.0),
            WinType::Credits => iced::Size::new(420.0, 195.0),
            WinType::News => iced::Size::new(350.0, 300.0),
            WinType::UserFavorites => iced::Size::new(450.0, 600.0),
            WinType::UserFavoritesExport => iced::Size::new(320.0, 175.0),
            WinType::UserProfileEdit => iced::Size::new(290.0, 293.0),
            WinType::UserPassword => iced::Size::new(280.0, 232.0),
            WinType::UserProfileDelete => iced::Size::new(340.0, 296.0),
            WinType::PlayerTimer => iced::Size::new(280.0, 150.0),
            WinType::Settings => iced::Size::new(360.0, 340.0),
        }
    }

    pub fn title(&self) -> &'static str {
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

impl Pager {
    pub fn goto(&mut self, page: u32) {
        self.page = page;
        self.input = page.to_string();
        self.loading = true;
    }

    pub fn loaded(&mut self, pages: u32, total: u32) {
        self.loading = false;
        self.pages = pages;
        self.total = total;
        self.input = self.page.to_string();
    }

    pub fn accept_input(&mut self, s: String) {
        digits_input(&mut self.input, s);
    }

    pub fn submit(&mut self) -> Option<u32> {
        if let Ok(p) = self.input.parse::<u32>() {
            let p = p.clamp(1, self.pages.max(1));
            if p != self.page {
                return Some(p);
            }
        }
        self.input = self.page.to_string();
        None
    }
}

#[derive(Default)]
pub struct HistoryState {
    pub list: Vec<HistoryEntry>,
    pub pager: Pager,
    pub date_from: String,
    pub date_to: String,
}

pub struct RatingsState {
    pub list: Vec<RatingEntry>,
    pub pager: Pager,
    pub range: String,
}

impl Default for RatingsState {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            pager: Pager::default(),
            range: "overtime".to_string(),
        }
    }
}

#[derive(Default)]
pub struct SongInfoState {
    pub data: Option<api::SongResponse>,
    pub loading: bool,
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
    pub list: Vec<ParsedNewsArticle>,
    pub pager: Pager,
}

#[derive(Debug, Clone)]
pub struct ParsedNewsArticle {
    pub author: String,
    pub created_at: u64,
    pub blocks: Vec<HtmlBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlBlock {
    Heading(String),
    Paragraph(Vec<(String, bool)>),
    ListItem(String),
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

#[derive(Debug, Clone)]
pub struct ScrobbleTrack {
    pub song_id: String,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub duration: f64,
    pub start_unix: Option<u64>,
    pub played_secs: f64,
    pub playing_since: Option<Instant>,
}

impl ScrobbleTrack {
    pub fn new(song: &api::StatusSong) -> Self {
        Self {
            song_id: song.id.clone(),
            artist: song.artist.clone(),
            title: song.title.clone(),
            album: song.album.clone(),
            duration: song.length,
            start_unix: None,
            played_secs: 0.0,
            playing_since: None,
        }
    }

    pub fn total_played(&self, now: Instant) -> f64 {
        self.played_secs
            + self
                .playing_since
                .map_or(0.0, |s| now.duration_since(s).as_secs_f64())
    }

    pub fn resume(&mut self, now: Instant) {
        if self.start_unix.is_none() {
            self.start_unix = Some(crate::now_unix());
        }
        if self.playing_since.is_none() {
            self.playing_since = Some(now);
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if let Some(s) = self.playing_since.take() {
            self.played_secs += now.duration_since(s).as_secs_f64();
        }
    }
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            minutes_input: "20".into(),
            until: None,
        }
    }
}

pub struct Plaza {
    pub main_window: iced::window::Id,
    pub child_windows: HashMap<iced::window::Id, WinType>,
    pub status: Status,
    pub player: Option<Arc<AudioPlayer>>,
    pub volume: f32,
    pub artwork_handle: Option<image::Handle>,
    pub artwork_url: String,

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

    pub elapsed: f64,
    pub last_tick: Instant,

    pub main_focused: bool,
    pub error_msg: Option<String>,

    pub welcome_until: Option<Instant>,
    pub volume_text: Option<String>,
    pub volume_text_until: Option<Instant>,

    pub auth_token: Option<String>,
    pub user: Option<api::User>,
    pub user_stats: Option<api::UserStatsData>,
    pub stats_loading: bool,

    pub reaction_rate: u8,
    pub reaction_song_id: String,

    pub config: crate::config::Config,
    pub lastfm_token: Option<String>,
    pub lastfm_busy: bool,
    pub lastfm_status: Option<String>,
    pub scrobble: Option<ScrobbleTrack>,

    pub discord_presence: crate::discord::DiscordHandle,
}

impl Plaza {
    pub fn new(
        main_window: iced::window::Id,
        player: Option<Arc<AudioPlayer>>,
        config: crate::config::Config,
    ) -> Self {
        Self {
            main_window,
            child_windows: HashMap::new(),
            status: Status::default(),
            player,
            volume: 50.0,
            artwork_handle: None,
            artwork_url: String::new(),
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
            elapsed: 0.0,
            last_tick: Instant::now(),
            main_focused: true,
            error_msg: None,
            welcome_until: Some(Instant::now() + std::time::Duration::from_secs(2)),
            volume_text: None,
            volume_text_until: None,
            auth_token: None,
            user: None,
            user_stats: None,
            stats_loading: false,
            reaction_rate: 0,
            reaction_song_id: String::new(),
            config,
            lastfm_token: None,
            lastfm_busy: false,
            lastfm_status: None,
            scrobble: None,
            discord_presence: crate::discord::DiscordHandle::spawn(),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.player.as_ref().is_some_and(|p| p.is_playing())
    }

    pub fn is_streaming(&self) -> bool {
        self.player.as_ref().is_some_and(|p| p.is_streaming())
    }
}

#[derive(Debug, Clone)]
pub enum HistoryMsg {
    Ok(Vec<HistoryEntry>, u32, u32, Option<api::DateRange>),
    Err(String),
    Page(u32),
    PageInput(String),
    PageSubmit,
}

#[derive(Debug, Clone)]
pub enum RatingsMsg {
    Ok(Vec<RatingEntry>, u32, u32),
    Err(String),
    Page(u32),
    Range(String),
    PageInput(String),
    PageSubmit,
}

#[derive(Debug, Clone)]
pub enum SongInfoMsg {
    Open(String),
    Ok(api::SongResponse),
    Err(String),
    ArtworkOk(Vec<u8>),
    ArtworkErr,
    ToggleFavorite,
    FavoriteAdded(u64),
    FavoriteRemoved,
    FavoriteErr(String),
}

#[derive(Debug, Clone)]
pub enum LoginMsg {
    Username(String),
    Password(String),
    Remember(bool),
    Submit,
    Ok(api::LoginResponse),
    Err(String),
}

#[derive(Debug, Clone)]
pub enum RegisterMsg {
    Username(String),
    Email(String),
    Password(String),
    PasswordRepeat(String),
    Submit,
    Ok(api::User),
    Err(String),
}

#[derive(Debug, Clone)]
pub enum NewsMsg {
    Ok(Vec<api::NewsArticle>, u32),
    Err(String),
    Page(u32),
    PageInput(String),
    PageSubmit,
}

#[derive(Debug, Clone)]
pub enum FavoritesMsg {
    Ok(Vec<api::FavoriteEntry>, u32, u32),
    ArtworkOk(String, Vec<u8>),
    Err(String),
    Page(u32),
    PageInput(String),
    PageSubmit,
    Delete(u64),
    DeleteOk(u64),
    DeleteErr(String),
}

#[derive(Debug, Clone)]
pub enum ExportMsg {
    Start,
    Ok(String),
    Err(String),
}

#[derive(Debug, Clone)]
pub enum ProfileEditMsg {
    Username(String),
    Email(String),
    CurrentPassword(String),
    Submit,
    Ok,
    Err(String),
}

#[derive(Debug, Clone)]
pub enum PasswordMsg {
    Current(String),
    New(String),
    Repeat(String),
    Submit,
    Ok,
    Err(String),
}

#[derive(Debug, Clone)]
pub enum DeleteMsg {
    Password(String),
    Confirm(bool),
    Submit,
    Ok,
    Err(String),
}

#[derive(Debug, Clone)]
pub enum LastfmMsg {
    ToggleEnabled(bool),
    Connect,
    TokenReady(String),
    Finish,
    SessionOk(String, String),
    Disconnect,
    Err(String),
}

#[derive(Debug, Clone)]
pub enum DiscordMsg {
    ToggleEnabled(bool),
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
    StatusOk(Status),
    StatusErr(String),
    Tick(Instant),
    TogglePlay,
    Volume(f32),
    ArtworkOk(Vec<u8>),
    ArtworkErr,
    OpenWin(WinType),
    CloseWin(iced::window::Id),
    WinClosed(iced::window::Id),

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
    Timer(TimerMsg),
    Lastfm(LastfmMsg),
    Discord(DiscordMsg),

    SessionRestored(api::User, String),

    Logout,
    LogoutOk,
    LogoutErr(String),

    StatsOk(api::UserStatsData),
    StatsErr(String),

    React,
    ReactOk(u32),
    ReactErr(String),

    MinimizeWin(iced::window::Id),
    DragWin(iced::window::Id),
    OpenUrl(String),

    Refresh,
    DismissErr,
    Noop,
    SpaceToggle(iced::window::Id),

    Media(souvlaki::MediaControlEvent),

    WinResized(iced::window::Id, iced::Size),
    WinFocus(iced::window::Id, bool),
}
