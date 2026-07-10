use crate::state::{Msg, Plaza, WinType};
use crate::theme::{self, ERROR_RED, FAVORITE_GOLD, HEART_RED};
use crate::views::bevel_button;
use crate::views::{
    bold_font, d3_thin_sunken, format_time, menu_bar, shaped, static_image, status_bar, ICON_FONT,
    IC_COG, IC_FAVORITE, IC_LIKE, IC_USER,
};
use iced::widget::{column, container, image, mouse_area, row, slider, text, Space};
use iced::{Element, Fill};
use std::time::Instant;

const VOLUME_IMG: &[u8] = include_bytes!("../assets/img/volume.png");

pub fn view(state: &Plaza) -> Element<'_, Msg> {
    let menu = menu_bar([
        ("About", Msg::OpenWin(WinType::About)),
        ("Play History", Msg::OpenWin(WinType::History)),
        ("Ratings", Msg::OpenWin(WinType::Ratings)),
        ("Support Us", Msg::OpenWin(WinType::Support)),
    ]);

    let cover = render_cover(state);
    let meta = render_metadata(state);

    let content = d3_thin_sunken(
        container(row![cover, Space::new().width(6), meta].padding(2))
            .style(theme::panel)
            .width(Fill)
            .padding(3),
    )
    .width(Fill);

    let status = render_status(state);

    let mut col = column![menu, content, Space::new().height(1), status].padding(2);

    if let Some(ref err) = state.error_msg {
        col = col.push(render_error(err));
    }

    col.into()
}

fn render_cover(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;

    let inner: Element<Msg> = if let Some(ref h) = state.artwork_handle {
        image(h.clone()).width(112).height(112).into()
    } else {
        Space::new().width(112).height(112).into()
    };

    let img = d3_thin_sunken(container(inner).style(theme::cover));

    if state.artwork_handle.is_some() && !song.id.is_empty() {
        mouse_area(img)
            .interaction(iced::mouse::Interaction::Pointer)
            .on_press(Msg::SongInfo(crate::state::SongInfoMsg::Open(
                song.id.clone(),
            )))
            .into()
    } else {
        img.into()
    }
}

fn render_metadata(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;

    let artist = shaped(if song.artist.is_empty() {
        "..."
    } else {
        &song.artist
    })
    .size(14)
    .font(bold_font());

    let title = shaped(&song.title).size(14);

    column![
        Space::new().height(2),
        artist,
        Space::new().height(8),
        title,
        Space::new().height(6),
        render_time_vol(state),
        Space::new().height(3),
        render_controls(state),
    ]
    .width(Fill)
    .into()
}

fn render_time_vol(state: &Plaza) -> Element<'_, Msg> {
    let time_str = match (
        state.welcome_until,
        state.volume_text.as_ref(),
        state.volume_text_until,
    ) {
        (Some(until), _, _) if Instant::now() < until => "Welcome back!".to_string(),
        (_, Some(vol), Some(until)) if Instant::now() < until => vol.clone(),
        _ => format_time_display(state),
    };

    let time_field = d3_thin_sunken(
        container(text(time_str).size(14).center().width(Fill))
            .width(Fill)
            .height(24)
            .center_y(24)
            .style(theme::panel),
    );

    let vol = slider(0.0..=100.0, state.volume, Msg::Volume)
        .width(Fill)
        .step(1.0_f32)
        .style(theme::volume_slider);

    let vol_icon = image(static_image(VOLUME_IMG)).width(11).height(16);
    let vol_row = row![vol, Space::new().width(5), vol_icon].align_y(iced::Alignment::Center);

    row![
        container(time_field).width(iced::Length::FillPortion(7)),
        Space::new().width(4),
        container(vol_row)
            .width(iced::Length::FillPortion(5))
            .center_y(24),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

fn render_controls(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;
    let is_playing = state.is_playing();
    let is_streaming = state.is_streaming();

    let play_txt = if is_playing && !is_streaming {
        "Loading..."
    } else if is_playing {
        "Pause"
    } else {
        "Play"
    };
    let play_btn = bevel_button(text(play_txt).size(11).center().width(Fill))
        .on_press(Msg::TogglePlay)
        .width(Fill);

    let is_current_song = state.reaction_song_id == song.id && !song.id.is_empty();
    let (react_icon_char, react_color) = match (is_current_song, state.reaction_rate) {
        (true, 2) => (IC_FAVORITE, FAVORITE_GOLD),
        (true, 1) => (IC_LIKE, HEART_RED),
        _ => (IC_LIKE, theme::BLACK),
    };

    let react_btn = bevel_button(
        container(
            row![
                text(react_icon_char)
                    .font(ICON_FONT)
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
    .width(Fill);

    let user_msg = if state.user.is_some() {
        Msg::OpenWin(WinType::UserProfile)
    } else {
        Msg::OpenWin(WinType::UserLogin)
    };
    let user_btn = bevel_button(
        text(IC_USER)
            .font(ICON_FONT)
            .size(11)
            .center()
            .width(Fill)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .on_press(user_msg)
    .width(Fill);

    let settings_btn = bevel_button(
        text(IC_COG)
            .font(ICON_FONT)
            .size(11)
            .center()
            .width(Fill)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .on_press(Msg::OpenWin(WinType::Settings))
    .width(Fill);

    let left_btns = row![
        container(play_btn).width(iced::Length::FillPortion(8)),
        container(react_btn).width(iced::Length::FillPortion(4)),
    ];
    let right_btns = row![
        container(user_btn).width(iced::Length::FillPortion(1)),
        container(settings_btn).width(iced::Length::FillPortion(1)),
    ];
    row![
        container(left_btns).width(iced::Length::FillPortion(7)),
        Space::new().width(8),
        container(right_btns).width(iced::Length::FillPortion(5)),
    ]
    .into()
}

fn render_status(state: &Plaza) -> Element<'_, Msg> {
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
    status_bar(status_cells)
}

fn render_error(err: &str) -> Element<'_, Msg> {
    container(
        row![
            text(err).size(10).color(ERROR_RED),
            Space::new().width(iced::Fill),
            bevel_button(text("x").size(10))
                .on_press(Msg::DismissErr)
                .padding(2),
        ]
        .align_y(iced::Alignment::Center)
        .padding([2, 4]),
    )
    .style(theme::panel)
    .width(Fill)
    .into()
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
