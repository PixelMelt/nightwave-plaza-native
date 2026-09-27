use iced::Size;
use iced::window::{Icon, Settings, icon};
use std::sync::LazyLock;

pub static DEV_MODE: LazyLock<bool> = LazyLock::new(|| std::env::var_os("NIGHTWAVE_DEV").is_some());

pub const MAIN_SIZE: Size = Size::new(450.0, 218.0);

static APP_ICON: LazyLock<Icon> = LazyLock::new(|| {
    let img = image::load_from_memory(include_bytes!("assets/icons/favicon-32x32.png"))
        .expect("app icon is a valid PNG")
        .into_rgba8();
    let (width, height) = img.dimensions();
    icon::from_rgba(img.into_raw(), width, height).expect("app icon is valid RGBA")
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowKind {
    About,
    Credits,
    History,
    News,
    PlayerTimer,
    Ratings,
    Settings,
    SongInfo,
    Support,
    UserFavorites,
    UserFavoritesExport,
    UserLogin,
    UserPassword,
    UserProfile,
    UserProfileDelete,
    UserProfileEdit,
    UserRegister,
}

impl WindowKind {
    pub fn size(self) -> Size {
        let (width, height) = match self {
            Self::About => (380.0, 518.0),
            Self::Credits => (420.0, 195.0),
            Self::History => (400.0, 630.0),
            Self::News => (350.0, 300.0),
            Self::PlayerTimer => (280.0, 150.0),
            Self::Ratings => (440.0, 630.0),
            Self::Settings => (360.0, 340.0),
            Self::SongInfo => (360.0, 225.0),
            Self::Support => (450.0, 270.0),
            Self::UserFavorites => (450.0, 600.0),
            Self::UserFavoritesExport => (320.0, 175.0),
            Self::UserLogin => (480.0, 150.0),
            Self::UserPassword => (280.0, 232.0),
            Self::UserProfile => (290.0, 250.0),
            Self::UserProfileDelete => (340.0, 296.0),
            Self::UserProfileEdit => (290.0, 293.0),
            Self::UserRegister => (430.0, 240.0),
        };
        Size::new(width, height)
    }

    pub fn is_resizable(self) -> bool {
        matches!(
            self,
            Self::History | Self::Ratings | Self::News | Self::UserFavorites
        )
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::About => "About",
            Self::Credits => "Credits",
            Self::History => "Play History",
            Self::News => "News",
            Self::PlayerTimer => "Sleep Timer",
            Self::Ratings => "Ratings",
            Self::Settings => "Settings",
            Self::SongInfo => "Song Info",
            Self::Support => "Support Us",
            Self::UserFavorites => "My Favorites",
            Self::UserFavoritesExport => "Export Favorites",
            Self::UserLogin => "Log In",
            Self::UserPassword => "Change Password",
            Self::UserProfile => "My Profile",
            Self::UserProfileDelete => "Delete Account",
            Self::UserProfileEdit => "Edit Profile",
            Self::UserRegister => "Registration",
        }
    }

    pub fn settings(self) -> Settings {
        settings(self.size(), self.is_resizable())
    }
}

pub fn settings(size: Size, resizable: bool) -> Settings {
    #[cfg(target_os = "linux")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        application_id: "nightwave-plaza".into(),
        ..Default::default()
    };
    #[cfg(not(target_os = "linux"))]
    let platform_specific = iced::window::settings::PlatformSpecific::default();

    Settings {
        size,
        resizable: resizable || *DEV_MODE,
        decorations: *DEV_MODE,
        icon: Some(APP_ICON.clone()),
        platform_specific,
        ..Default::default()
    }
}
