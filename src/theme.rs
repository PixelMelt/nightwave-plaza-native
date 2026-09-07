use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Gradient, Shadow, Theme};

pub const BG_GRAY: Color = Color::from_rgb(0.753, 0.753, 0.753);
pub const WHITE: Color = Color::WHITE;
pub const BLACK: Color = Color::BLACK;
pub const DARK_GRAY: Color = Color::from_rgb(0.502, 0.502, 0.502);
pub const LIGHT_GRAY: Color = Color::from_rgb(0.875, 0.875, 0.875);
pub const TITLE_BLUE: Color = Color::from_rgb(0.0, 0.0, 0.50);
pub const COVER_BG: Color = Color::from_rgb(0.937, 0.902, 0.922);
pub const TITLE_BLUE_END: Color = Color::from_rgb(0.063, 0.518, 0.816);
pub const ERROR_RED: Color = Color::from_rgb(0.8, 0.0, 0.0);

pub const HEART_RED: Color = Color::from_rgb(0.757, 0.153, 0.153);
pub const FAVORITE_GOLD: Color = Color::from_rgb(1.0, 0.827, 0.0);
pub const TIMER_BLUE: Color = Color::from_rgb(0.204, 0.333, 0.859); // #3455DB
pub const LINK_COLOR: Color = Color::from_rgb(0.024, 0.271, 0.678);
pub const MUTED: Color = Color::from_rgb(0.4, 0.4, 0.4);
pub const DISABLED: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const DIVIDER_GRAY: Color = Color::from_rgb(0.784, 0.784, 0.784);
const HOVER_GRAY: Color = Color::from_rgb(0.88, 0.88, 0.88);

#[derive(Clone, Copy)]
pub struct BevelColors {
    pub tl_outer: Color,
    pub tl_inner: Color,
    pub br_inner: Color,
    pub br_outer: Color,
}

pub const BEVEL_RAISED: BevelColors = BevelColors {
    tl_outer: WHITE,
    tl_inner: LIGHT_GRAY,
    br_inner: DARK_GRAY,
    br_outer: BLACK,
};

pub const BEVEL_WINDOW: BevelColors = BevelColors {
    tl_outer: LIGHT_GRAY,
    tl_inner: WHITE,
    br_inner: DARK_GRAY,
    br_outer: BLACK,
};

pub const BEVEL_PRESSED: BevelColors = BevelColors {
    tl_outer: BLACK,
    tl_inner: DARK_GRAY,
    br_inner: DARK_GRAY,
    br_outer: BLACK,
};

pub const BEVEL_SUNKEN: BevelColors = BevelColors {
    tl_outer: DARK_GRAY,
    tl_inner: BLACK,
    br_inner: LIGHT_GRAY,
    br_outer: WHITE,
};

pub const THIN_SUNKEN: (Color, Color) = (DARK_GRAY, WHITE);
pub const THIN_MENU_HOVER: (Color, Color) = (WHITE, DARK_GRAY);

pub fn app_theme() -> Theme {
    static THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();
    THEME
        .get_or_init(|| {
            Theme::custom(
                "Win98".to_string(),
                iced::theme::Palette {
                    background: BG_GRAY,
                    text: BLACK,
                    primary: TITLE_BLUE,
                    success: Color::from_rgb(0.0, 0.5, 0.0),
                    warning: Color::from_rgb(0.8, 0.5, 0.0),
                    danger: ERROR_RED,
                },
            )
        })
        .clone()
}

pub fn solid_border(color: Color) -> Border {
    Border {
        color,
        width: 1.0,
        radius: 0.0.into(),
    }
}

pub fn cover(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COVER_BG)),
        ..base_panel()
    }
}

pub fn separator(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(DIVIDER_GRAY)),
        ..Default::default()
    }
}

pub fn fill(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

pub fn flat_button(text_color: Color) -> button::Style {
    button::Style {
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        text_color,
        snap: false,
    }
}

pub fn base_panel() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
        snap: false,
    }
}

pub fn panel(_t: &Theme) -> container::Style {
    base_panel()
}

pub fn title_bar_bg(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            iced::gradient::Linear::new(iced::Radians(std::f32::consts::FRAC_PI_2))
                .add_stop(0.0, TITLE_BLUE)
                .add_stop(1.0, TITLE_BLUE_END),
        ))),
        text_color: Some(WHITE),
        ..base_panel()
    }
}

pub fn title_bar_bg_inactive(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            iced::gradient::Linear::new(iced::Radians(std::f32::consts::FRAC_PI_2))
                .add_stop(0.0, DARK_GRAY)
                .add_stop(1.0, BG_GRAY),
        ))),
        text_color: Some(BG_GRAY),
        ..base_panel()
    }
}

pub fn sunken_inner(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WHITE)),
        ..base_panel()
    }
}

pub fn sunken(_t: &Theme) -> container::Style {
    container::Style {
        border: solid_border(DARK_GRAY),
        ..sunken_inner(_t)
    }
}

pub fn list_row_btn(_t: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: matches!(status, button::Status::Hovered)
            .then_some(Background::Color(HOVER_GRAY)),
        ..flat_button(BLACK)
    }
}

pub fn scrollbar(_t: &Theme, _s: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: sunken(_t),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(BG_GRAY)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(BG_GRAY),
                border: solid_border(BLACK),
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(BG_GRAY)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(BG_GRAY),
                border: solid_border(BLACK),
            },
        },
        gap: None,

        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(BG_GRAY),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: BLACK,
        },
    }
}

pub fn page_input(_t: &Theme, _s: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(WHITE),
        border: solid_border(DARK_GRAY),
        icon: BLACK,
        placeholder: DARK_GRAY,
        value: BLACK,
        selection: Color::from_rgb(0.0, 0.0, 0.5),
    }
}
