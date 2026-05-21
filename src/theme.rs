use iced::widget::{container, scrollable, slider, text_input};
use iced::{Background, Border, Color, Gradient, Shadow, Theme};

// Plaza Win9x palette: BG_GRAY #C0C0C0, DARK_GRAY #808080, LIGHT_GRAY #DFDFDF.
pub const BG_GRAY: Color = Color::from_rgb(0.753, 0.753, 0.753);
pub const WHITE: Color = Color::WHITE;
pub const BLACK: Color = Color::BLACK;
pub const DARK_GRAY: Color = Color::from_rgb(0.502, 0.502, 0.502);
pub const LIGHT_GRAY: Color = Color::from_rgb(0.875, 0.875, 0.875);
pub const TITLE_BLUE: Color = Color::from_rgb(0.0, 0.0, 0.50);
pub const COVER_BG: Color = Color::from_rgb(0.937, 0.902, 0.922);
pub const TITLE_BLUE_END: Color = Color::from_rgb(0.063, 0.518, 0.816);

pub fn solid_border(color: Color) -> Border {
    Border {
        color,
        width: 1.0,
        radius: 0.0.into(),
    }
}

pub fn base_panel() -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

pub fn window_box(_t: &Theme) -> container::Style { base_panel() }
pub fn panel(_t: &Theme) -> container::Style { base_panel() }
pub fn status_bar(_t: &Theme) -> container::Style { base_panel() }
pub fn status_cell(_t: &Theme) -> container::Style { base_panel() }
pub fn group_label(_t: &Theme) -> container::Style { base_panel() }
pub fn text_field(_t: &Theme) -> container::Style { base_panel() }

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

pub fn list_row_btn(
    _t: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: if matches!(status, iced::widget::button::Status::Hovered) {
            Some(Background::Color(Color::from_rgb(0.88, 0.88, 0.88)))
        } else {
            None
        },
        text_color: BLACK,
        border: Border::default(),
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
                border: solid_border(BLACK),
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(BG_GRAY)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BG_GRAY,
                border: solid_border(BLACK),
            },
        },
        gap: None,
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

pub fn volume_slider(_t: &Theme, _s: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (Background::Color(DARK_GRAY), Background::Color(LIGHT_GRAY)),
            width: 4.0,
            border: solid_border(DARK_GRAY),
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
