use crate::state::{Msg, Plaza, WinType};
use crate::theme;
use crate::views::widgets::{
    self, d3_raised, d3_sunken, format_time, menu_btn_underline, shaped, status_bar,
};
use iced::widget::{button, column, container, horizontal_space, image, row, slider, text, Space};
use iced::{Element, Fill, Theme};
use std::time::Instant;

const FAVORITE_GOLD: iced::Color = iced::Color::from_rgb(1.0, 0.827, 0.0);
const VOLUME_IMG: &[u8] = include_bytes!("../../assets/icons/volume.png");

pub fn view(state: &Plaza) -> Element<Msg> {
    let song = &state.status.song;
    let is_playing = state.player.as_ref().map_or(false, |p| p.is_playing());
    let is_streaming = state.player.as_ref().map_or(false, |p| p.is_streaming());

    let menu = row![
        menu_btn_underline("About", Msg::OpenWin(WinType::About)),
        menu_btn_underline("Play History", Msg::OpenWin(WinType::History)),
        menu_btn_underline("Ratings", Msg::OpenWin(WinType::Ratings)),
        menu_btn_underline("Support Us", Msg::OpenWin(WinType::Support)),
    ]
    .padding([1, 1]);

    let cover_style = |_: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COVER_BG)),
        border: iced::Border {
            color: theme::DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    };

    let cover: Element<Msg> = if let Some(ref h) = state.artwork_handle {
        let img = container(image(h.clone()).width(112).height(112)).style(cover_style);
        if !song.id.is_empty() {
            button(img)
                .on_press(Msg::OpenSongInfo(song.id.clone()))
                .style(|_: &Theme, _| button::Style {
                    background: None,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: theme::BLACK,
                })
                .padding(0)
                .into()
        } else {
            img.into()
        }
    } else {
        container(Space::new(112, 112)).style(cover_style).into()
    };

    let artist = shaped(if song.artist.is_empty() {
        "..."
    } else {
        &song.artist
    })
    .size(14)
    .font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    });

    let title = shaped(if song.title.is_empty() {
        ""
    } else {
        &song.title
    })
    .size(14);

    let time_str = if let Some(until) = state.welcome_until {
        if Instant::now() < until {
            "Welcome back!".to_string()
        } else {
            format_time_display(state)
        }
    } else if let (Some(ref vol_text), Some(until)) = (&state.volume_text, state.volume_text_until)
    {
        if Instant::now() < until {
            vol_text.clone()
        } else {
            format_time_display(state)
        }
    } else {
        format_time_display(state)
    };

    let time_field = d3_sunken(
        container(text(time_str).size(14).center().width(Fill))
            .width(Fill)
            .height(24)
            .center_y(24)
            .style(theme::text_field),
    );

    let vol = slider(0.0..=100.0, state.volume, Msg::Volume)
        .width(Fill)
        .step(1.0)
        .style(theme::volume_slider);

    let vol_icon = image(image::Handle::from_bytes(VOLUME_IMG))
        .width(11)
        .height(16);
    let vol_row = row![vol, Space::with_width(5), vol_icon].align_y(iced::Alignment::Center);

    let time_vol = row![
        container(time_field).width(iced::Length::FillPortion(7)),
        Space::with_width(4),
        container(vol_row)
            .width(iced::Length::FillPortion(5))
            .center_y(24),
    ]
    .align_y(iced::Alignment::Center);

    let play_txt = if is_playing && !is_streaming {
        "Loading..."
    } else if is_playing {
        "Stop"
    } else {
        "Play"
    };
    let play_btn = button(text(play_txt).size(11).center().width(Fill))
        .on_press(Msg::TogglePlay)
        .width(Fill)
        .style(theme::raised);

    let is_current_song = state.reaction_song_id == song.id && !song.id.is_empty();
    let (react_icon_char, react_color) = if is_current_song {
        match state.reaction_rate {
            2 => (widgets::IC_FAVORITE, FAVORITE_GOLD),
            1 => (widgets::IC_LIKE, widgets::HEART_RED),
            _ => (widgets::IC_LIKE, theme::BLACK),
        }
    } else {
        (widgets::IC_LIKE, theme::BLACK)
    };

    let react_btn = button(
        container(
            row![
                text(react_icon_char)
                    .font(widgets::ICON_FONT)
                    .size(11)
                    .color(react_color)
                    .shaping(iced::widget::text::Shaping::Advanced),
                text(format!(" {}", song.reactions)).size(11),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(2),
        )
        .center_x(Fill),
    )
    .on_press(Msg::React)
    .style(theme::raised)
    .width(Fill);

    let user_msg = if state.user.is_some() {
        Msg::OpenWin(WinType::UserProfile)
    } else {
        Msg::OpenWin(WinType::UserLogin)
    };
    let user_btn = button(
        text(widgets::IC_USER)
            .font(widgets::ICON_FONT)
            .size(11)
            .center()
            .width(Fill)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .on_press(user_msg)
    .style(theme::raised)
    .width(Fill);

    let settings_btn = button(
        text(widgets::IC_COG)
            .font(widgets::ICON_FONT)
            .size(11)
            .center()
            .width(Fill)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .style(theme::raised)
    .width(Fill);

    let left_btns = row![
        d3_raised(play_btn).width(iced::Length::FillPortion(8)),
        d3_raised(react_btn).width(iced::Length::FillPortion(4)),
    ];
    let right_btns = row![
        d3_raised(user_btn).width(iced::Length::FillPortion(1)),
        d3_raised(settings_btn).width(iced::Length::FillPortion(1)),
    ];
    let btn_row = row![
        container(left_btns).width(iced::Length::FillPortion(7)),
        Space::with_width(8),
        container(right_btns).width(iced::Length::FillPortion(5)),
    ];

    let meta = column![
        Space::with_height(2),
        artist,
        Space::with_height(8),
        title,
        Space::with_height(6),
        time_vol,
        Space::with_height(3),
        btn_row,
    ]
    .width(Fill);

    let content = d3_sunken(
        container(row![cover, Space::with_width(6), meta].padding(2))
            .style(theme::text_field)
            .width(Fill)
            .padding(3),
    )
    .width(Fill);

    let mut status_cells: Vec<Element<Msg>> =
        vec![text(format!("Listeners: {}", state.status.listeners))
            .size(11)
            .width(Fill)
            .into()];
    if let Some(ref u) = state.user {
        status_cells.push(
            text(format!("Logged in as: {}", u.username))
                .size(11)
                .into(),
        );
    }
    let status = status_bar(status_cells);

    let mut col = column![menu, content, Space::with_height(1), status].padding(2);

    if let Some(ref err) = state.error_msg {
        col = col.push(
            container(
                row![
                    text(err)
                        .size(10)
                        .color(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                    horizontal_space(),
                    d3_raised(
                        button(text("x").size(10))
                            .on_press(Msg::DismissErr)
                            .style(theme::raised)
                            .padding(2),
                    ),
                ]
                .align_y(iced::Alignment::Center)
                .padding([2, 4]),
            )
            .style(theme::status_bar)
            .width(Fill),
        );
    }

    col.into()
}

fn format_time_display(state: &Plaza) -> String {
    let pos = state.elapsed;
    let dur = state.status.song.length;
    if dur > 0.0 {
        format!("{} / {}", format_time(pos), format_time(dur))
    } else {
        "...".into()
    }
}
