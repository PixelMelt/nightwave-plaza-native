use iced::widget::{button, container, scrollable, slider, text_input};
use iced::{Background, Border, Color, Gradient, Shadow, Theme};

pub const BG_GRAY: Color = Color::from_rgb(0.75, 0.75, 0.75); // #c0c0c0
pub const WHITE: Color = Color::WHITE;
pub const BLACK: Color = Color::BLACK;
pub const DARK_GRAY: Color = Color::from_rgb(0.50, 0.50, 0.50); // #808080
pub const LIGHT_GRAY: Color = Color::from_rgb(0.87, 0.87, 0.87); // #dfdfdf
pub const TITLE_BLUE: Color = Color::from_rgb(0.0, 0.0, 0.50); // #000080
pub const COVER_BG: Color = Color::from_rgb(0.937, 0.902, 0.922); // #efe6eb

// ── Window outer frame ──────────────────────────────────────────
// No border/shadow — d3_raised_window wrapper provides the 3D border.
pub fn window_box(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Title bar (gradient: #000080 → #1084d0, matching webapp) ────
pub const TITLE_BLUE_END: Color = Color::from_rgb(0.063, 0.518, 0.816); // #1084d0

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

// ── Sunken / input field (d3-content) ───────────────────────────
// Used directly by scrollbar container (can't be wrapped in d3_sunken).
// For standalone sunken areas, wrap in d3_sunken + use sunken_inner instead.
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

/// Sunken background only (no border) — for use inside d3_sunken wrappers.
pub fn sunken_inner(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WHITE)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Win-panel (sunken 3D panel with gray bg) ────────────────────
// No border/shadow — d3_sunken wrapper provides the 3D border.
pub fn panel(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Status bar outer ────────────────────────────────────────────
pub fn status_bar(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        text_color: Some(BLACK),
        shadow: Shadow::default(),
    }
}

// ── Individual status bar cell (sunken cell look) ───────────────
// No border/shadow — d3_sunken wrapper provides the 3D border.
pub fn status_cell(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Raised button (Win98 3D effect) ─────────────────────────────
// No border/shadow here — the d3_raised wrapper provides the 3D border.
// Pressed: add a 1px black border to flatten the look inside the wrapper.
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

// ── Menu button (flat, border on hover) ─────────────────────────
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

// ── Clickable list row (transparent, highlight on hover) ────────
// Webapp: .show-info:hover { text-decoration: underline }
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

// ── Active tab button ───────────────────────────────────────────
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

// ── Scrollbar ───────────────────────────────────────────────────
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

// ── Group box label (gray bg to mask the border line behind it) ─
// Webapp: .win-group-box__label span { background-color: var(--button-bg) }
pub fn group_label(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Text field (gray bg, sunken border — webapp .text-field) ────
// No border/shadow — d3_sunken wrapper provides the 3D border.
pub fn text_field(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_GRAY)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Some(BLACK),
    }
}

// ── Text input (sunken, for pagination) ─────────────────────────
// Sunken input: border=WHITE for light bottom-right edge (dark top-left via shadow not available
// on text_input, so we use DARK_GRAY border as best approximation)
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

// ── Volume slider (Win98 noUiSlider style) ──────────────────────
// Webapp: .noUi-handle uses @include d3-object (raised bevel),
//   width: 12px, height: 24px, border-radius: 0, top: 1px
// Track: 4px tall .player-volume-line with simple-border (sunken)
pub fn volume_slider(_t: &Theme, _s: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            // Left side (filled) = gray, right side (empty) = gray
            // The visual track in webapp is a separate sunken div behind the slider;
            // here we approximate with a thin gray rail + sunken border
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
            // d3-object: border-right/bottom = #000 (black)
            background: Background::Color(BG_GRAY),
            border_color: BLACK,
            border_width: 1.0,
        },
    }
}
