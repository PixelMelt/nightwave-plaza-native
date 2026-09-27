use crate::api::{self, RatingsRange, Reaction, Status, User};
use crate::article::Article;
use crate::audio::AudioPlayer;
use crate::config::{Config, Session};
use crate::discord::Discord;
use crate::lastfm::Scrobble;
use crate::media_controls::MediaSession;
use crate::message::{Msg, PageMsg};
use crate::window::WindowKind;
use futures::channel::mpsc::UnboundedSender;
use iced::widget::image;
use iced::window::Id;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const DEFAULT_VOLUME: f32 = 50.0;
const NOTICE_DURATION: Duration = Duration::from_secs(2);
const MIN_PASSWORD_LEN: usize = 3;

pub struct Plaza {
    pub main_window: Id,
    pub windows: HashMap<Id, WindowKind>,
    pub focused: Option<Id>,

    pub status: Status,
    pub status_at: Instant,
    pub player: AudioPlayer,
    pub media: MediaSession,
    pub volume: f32,
    pub artwork: Option<image::Handle>,
    pub artwork_url: String,
    pub notice: Option<Notice>,
    pub alert: Option<String>,

    pub session: Option<Session>,
    pub user_stats: Option<api::UserStats>,
    pub stats_loading: bool,
    pub reaction: SongReaction,

    pub history: HistoryState,
    pub ratings: RatingsState,
    pub song_info: SongInfoState,
    pub login: LoginForm,
    pub register: RegisterForm,
    pub news: NewsState,
    pub favorites: FavoritesState,
    pub export: ExportState,
    pub profile_edit: ProfileEditForm,
    pub password: PasswordForm,
    pub delete_account: DeleteAccountForm,
    pub timer: TimerState,
    pub lastfm: LastfmState,

    pub config: Config,
    pub scrobble: Option<Scrobble>,
    pub discord: Discord,
}

impl Plaza {
    pub fn new(main_window: Id, config: Config, events: UnboundedSender<Msg>) -> Self {
        let player = AudioPlayer::spawn(events.clone());
        player.set_volume(DEFAULT_VOLUME / 100.0);
        let mut media = MediaSession::new(events);
        media.set_playing(player.is_playing());

        Self {
            main_window,
            windows: HashMap::new(),
            focused: Some(main_window),
            status: Status::default(),
            status_at: Instant::now(),
            player,
            media,
            volume: DEFAULT_VOLUME,
            artwork: None,
            artwork_url: String::new(),
            notice: Some(Notice::new("Welcome back!")),
            alert: None,
            session: config.session.clone(),
            user_stats: None,
            stats_loading: false,
            reaction: SongReaction::default(),
            history: HistoryState::default(),
            ratings: RatingsState::default(),
            song_info: SongInfoState::default(),
            login: LoginForm::default(),
            register: RegisterForm::default(),
            news: NewsState::default(),
            favorites: FavoritesState::default(),
            export: ExportState::default(),
            profile_edit: ProfileEditForm::default(),
            password: PasswordForm::default(),
            delete_account: DeleteAccountForm::default(),
            timer: TimerState::default(),
            lastfm: LastfmState::default(),
            config,
            scrobble: None,
            discord: Discord::spawn(),
        }
    }

    pub fn is_main_focused(&self) -> bool {
        self.focused == Some(self.main_window)
    }

    pub fn window_of(&self, kind: WindowKind) -> Option<Id> {
        self.windows
            .iter()
            .find_map(|(&id, &k)| (k == kind).then_some(id))
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

pub struct Notice {
    pub text: String,
    pub until: Instant,
}

impl Notice {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            until: Instant::now() + NOTICE_DURATION,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SongReaction {
    pub song_id: String,
    pub reaction: Reaction,
}

impl SongReaction {
    pub fn for_song(&self, song_id: &str) -> Reaction {
        if !song_id.is_empty() && self.song_id == song_id {
            self.reaction
        } else {
            Reaction::None
        }
    }
}

pub fn accept_digits(field: &mut String, input: String) {
    if input.chars().all(|c| c.is_ascii_digit()) {
        *field = input;
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
    pub fn start_loading(&mut self, page: u32) {
        self.page = page;
        self.input = page.to_string();
        self.loading = true;
    }

    pub fn finish_loading(&mut self, meta: &api::PageMeta) {
        self.loading = false;
        self.pages = meta.last_page;
        self.total = meta.total;
        self.input = self.page.to_string();
    }

    pub fn apply(&mut self, msg: PageMsg) -> Option<u32> {
        match msg {
            PageMsg::Go(page) => Some(page),
            PageMsg::Input(input) => {
                accept_digits(&mut self.input, input);
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

#[derive(Default)]
pub struct RatingsState {
    pub list: Vec<api::RatingEntry>,
    pub pager: Pager,
    pub range: RatingsRange,
}

#[derive(Default)]
pub struct NewsState {
    pub list: Vec<Article>,
    pub pager: Pager,
}

#[derive(Default)]
pub struct SongInfoState {
    pub data: Option<api::SongResponse>,
    pub error: Option<String>,
    pub artwork: Option<image::Handle>,
    pub favorite_id: Option<u64>,
    pub favorite_pending: bool,
}

#[derive(Default)]
pub struct FavoritesState {
    pub list: Vec<api::FavoriteEntry>,
    pub removed: Vec<u64>,
    pub pager: Pager,
    pub artwork: HashMap<String, image::Handle>,
}

#[derive(Default)]
pub struct ExportState {
    pub loading: bool,
    pub link: Option<String>,
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

#[derive(Default)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub remember: bool,
    pub loading: bool,
    pub error: Option<String>,
}

impl LoginForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.username.is_empty() || self.password.is_empty() {
            return Err("Please enter a username and password.");
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_repeat: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl RegisterForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        let allowed = |c: char| c.is_alphanumeric() || c == '-' || c == '_';
        if !self.username.chars().all(allowed) {
            Err("Username may only contain letters, numbers, and underscores.")
        } else if self.username.len() < 4 {
            Err("Username is too short.")
        } else if self.username.len() > 32 {
            Err("Username is too long.")
        } else if self.password.len() < MIN_PASSWORD_LEN {
            Err("Password is too short.")
        } else if self.password != self.password_repeat {
            Err("Passwords do not match.")
        } else if self.email.is_empty() {
            Err("Email is required.")
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct ProfileEditForm {
    pub username: String,
    pub email: String,
    pub current_password: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl ProfileEditForm {
    pub fn for_user(user: Option<&User>) -> Self {
        Self {
            username: user.map(|u| u.username.clone()).unwrap_or_default(),
            email: user.map(|u| u.email.clone()).unwrap_or_default(),
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.current_password.is_empty() {
            return Err("Current password is required.");
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct PasswordForm {
    pub current_password: String,
    pub password: String,
    pub password_repeat: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl PasswordForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.current_password.is_empty() {
            Err("Current password is required.")
        } else if self.password.len() < MIN_PASSWORD_LEN {
            Err("Password is too short.")
        } else if self.password != self.password_repeat {
            Err("Passwords do not match.")
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct DeleteAccountForm {
    pub current_password: String,
    pub confirmed: bool,
    pub loading: bool,
    pub error: Option<String>,
}

impl DeleteAccountForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.confirmed {
            Err("You must confirm account deletion.")
        } else if self.current_password.is_empty() {
            Err("Current password is required.")
        } else {
            Ok(())
        }
    }
}
