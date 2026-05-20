use iced::widget::{button, container, scrollable, slider, text_input};
use iced::{Background, Border, Color, Gradient, Shadow, Theme};

pub const BG_GRAY: Color = Color::from_rgb(0.75, 0.75, 0.75);
pub const WHITE: Color = Color::WHITE;
pub const BLACK: Color = Color::BLACK;
pub const DARK_GRAY: Color = Color::from_rgb(0.50, 0.50, 0.50);
pub const LIGHT_GRAY: Color = Color::from_rgb(0.87, 0.87, 0.87);
pub const TITLE_BLUE: Color = Color::from_rgb(0.0, 0.0, 0.50);
pub const COVER_BG: Color = Color::from_rgb(0.937, 0.902, 0.922);

pub fn window_box(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub const TITLE_BLUE_END: Color = Color::from_rgb(0.063, 0.518, 0.816);

pub fn title_bar_bg(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            iced::gradient::Linear::new(iced::Radians(std::f32::consts::FRAC_PI_2))
                .add_stop(0.0, TITLE_BLUE)
                .add_stop(1.0, TITLE_BLUE_END),
        ))),
        text_color: Some(WHITE),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn sunken(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WHITE)),
        border: Border {
            color: DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn sunken_inner(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WHITE)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn panel(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn status_bar(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        text_color: Some(BLACK),
        shadow: Shadow::default(),
    }
}

pub fn status_cell(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn raised(_t: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(BG_GRAY)),
        text_color: BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
    };
    match status {
        button::Status::Pressed => button::Style {
            border: Border {
                color: BLACK,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: DARK_GRAY,
            ..base
        },
        _ => base,
    }
}

pub fn menu_btn(_t: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(BG_GRAY)),
        text_color: BLACK,
        border: if matches!(status, button::Status::Hovered) {
            Border {
                color: DARK_GRAY,
                width: 1.0,
                radius: 0.0.into(),
            }
        } else {
            Border::default()
        },
        shadow: Shadow::default(),
    }
}

pub fn list_row_btn(_t: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: if matches!(status, button::Status::Hovered) {
            Some(Background::Color(Color::from_rgb(0.88, 0.88, 0.88)))
        } else {
            None
        },
        text_color: BLACK,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn active_tab(_t: &Theme, _s: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(BG_GRAY)),
        text_color: BLACK,
        border: Border {
            color: BLACK,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn scrollbar(_t: &Theme, _s: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: sunken(_t),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(BG_GRAY)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BG_GRAY,
                border: Border {
                    color: BLACK,
                    width: 1.0,
                    radius: 0.0.into(),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(BG_GRAY)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BG_GRAY,
                border: Border {
                    color: BLACK,
                    width: 1.0,
                    radius: 0.0.into(),
                },
            },
        },
        gap: None,
    }
}

pub fn group_label(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn text_field(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn page_input(_t: &Theme, _s: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(WHITE),
        border: Border {
            color: DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        icon: BLACK,
        placeholder: DARK_GRAY,
        value: BLACK,
        selection: Color::from_rgb(0.0, 0.0, 0.5),
    }
}

pub fn volume_slider(_t: &Theme, _s: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (Background::Color(DARK_GRAY), Background::Color(LIGHT_GRAY)),
            width: 4.0,
            border: Border {
                color: DARK_GRAY,
                width: 1.0,
                radius: 0.0.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Rectangle {
                width: 12,
                border_radius: 0.0.into(),
            },

            background: Background::Color(BG_GRAY),
            border_color: BLACK,
            border_width: 1.0,
        },
    }
}
