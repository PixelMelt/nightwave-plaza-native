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
}

impl WinType {
    /// Window sizes — must account for custom title bar (~26px) since OS decorations are disabled.
    /// Widths match webapp windowsConfig.ts exactly.
    pub fn size(&self) -> iced::Size {
        match self {
            WinType::History => iced::Size::new(400.0, 630.0),
            WinType::About => iced::Size::new(380.0, 505.0),
            WinType::Ratings => iced::Size::new(440.0, 630.0),
            WinType::Support => iced::Size::new(450.0, 270.0),
            WinType::SongInfo => iced::Size::new(360.0, 250.0),
            WinType::UserLogin => iced::Size::new(480.0, 230.0),
            WinType::UserProfile => iced::Size::new(290.0, 360.0),
            WinType::UserRegister => iced::Size::new(430.0, 340.0),
            WinType::Credits => iced::Size::new(420.0, 230.0),
            WinType::News => iced::Size::new(350.0, 430.0),
        }
    }

    /// Window title matching webapp locale keys
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
    // History
    pub history: Vec<HistoryEntry>,
    pub history_page: u32,
    pub history_pages: u32,
    pub history_total: u32,
    pub history_date_from: String,
    pub history_date_to: String,
    pub history_page_input: String,
    // Ratings
    pub ratings: Vec<RatingEntry>,
    pub ratings_page: u32,
    pub ratings_pages: u32,
    pub ratings_range: String,
    pub ratings_total: u32,
    pub ratings_page_input: String,
    // Loading flags
    pub history_loading: bool,
    pub ratings_loading: bool,
    // Song Info
    pub song_info: Option<api::SongResponse>,
    pub song_info_loading: bool,
    pub song_info_artwork: Option<image::Handle>,
    // Timing
    pub elapsed: f64,
    pub last_tick: Instant,
    pub error_msg: Option<String>,
    // UI transient state
    pub welcome_until: Option<Instant>,
    pub volume_text: Option<String>,
    pub volume_text_until: Option<Instant>,
    // ── Auth / User ─────────────────────────────────────────────
    pub auth_token: Option<String>,
    pub user: Option<api::User>,
    pub user_stats: Option<api::UserStatsData>,
    pub stats_loading: bool,
    // Reaction state (0=none, 1=like, 2=favorite)
    pub reaction_rate: u8,
    pub reaction_song_id: String,
    // Login form
    pub login_username: String,
    pub login_password: String,
    pub login_remember: bool,
    pub login_loading: bool,
    pub login_error: Option<String>,
    // Register form
    pub register_username: String,
    pub register_email: String,
    pub register_password: String,
    pub register_password_repeat: String,
    pub register_loading: bool,
    pub register_error: Option<String>,
    // News
    pub news: Vec<api::NewsArticle>,
    pub news_page: u32,
    pub news_pages: u32,
    pub news_loading: bool,
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
    // History
    HistoryOk(Vec<HistoryEntry>, u32, u32, Option<api::DateRange>),
    HistoryErr(String),
    HistoryPage(u32),
    HistoryPageInput(String),
    HistoryPageSubmit,
    // Ratings
    RatingsOk(Vec<RatingEntry>, u32, u32),
    RatingsErr(String),
    RatingsPage(u32),
    RatingsRange(String),
    RatingsPageInput(String),
    RatingsPageSubmit,
    // Song Info
    OpenSongInfo(String),
    SongInfoOk(api::SongResponse),
    SongInfoErr(String),
    SongInfoArtworkOk(Vec<u8>),
    SongInfoArtworkErr,
    // ── Auth ─────────────────────────────────────────────────────
    SessionRestored(api::User, String), // (user, token) — loaded from disk on startup
    // Login form
    LoginUsername(String),
    LoginPassword(String),
    LoginRemember(bool),
    LoginSubmit,
    LoginOk(api::LoginResponse),
    LoginErr(String),
    // Register form
    RegisterUsername(String),
    RegisterEmail(String),
    RegisterPassword(String),
    RegisterPasswordRepeat(String),
    RegisterSubmit,
    RegisterOk(api::User),
    RegisterErr(String),
    // Logout
    Logout,
    LogoutOk,
    LogoutErr(String),
    // User stats
    StatsOk(api::UserStatsData),
    StatsErr(String),
    // Reactions (like/favorite cycle)
    React,
    ReactOk(u32),
    ReactErr(String),
    // News
    NewsOk(Vec<api::NewsArticle>, u32),
    NewsErr(String),
    NewsPage(u32),
    // Window chrome (custom title bar)
    MinimizeWin(iced::window::Id),
    DragWin(iced::window::Id),
    OpenUrl(String),
    // General
    Refresh,
    DismissErr,
}
