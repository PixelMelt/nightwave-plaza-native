use crate::state::{Msg, Plaza, WinType};
use crate::theme::{self, ERROR_RED};
use crate::views::bevel_button;
use crate::views::volume::volume_slider;
use crate::views::{
    bold_font, d3_thin_sunken, format_time, menu_bar, shaped, static_image, status_bar,
};
use iced::widget::text::LineHeight;
use iced::widget::{column, container, image, mouse_area, row, text, Row, Space};
use iced::{Alignment, Element, Fill, Length, Padding, Pixels};
use std::time::Instant;

const VOLUME_IMG: &[u8] = include_bytes!("../assets/img/volume.png");
const PERSON_IMG: &[u8] = include_bytes!("../assets/icons/person.png");
const GEARS_IMG: &[u8] = include_bytes!("../assets/icons/gears.png");
const HEART_IMG: &[u8] = include_bytes!("../assets/icons/heart.png");
const HEART_GRAY_IMG: &[u8] = include_bytes!("../assets/icons/heart_gray.png");
const STAR_IMG: &[u8] = include_bytes!("../assets/icons/star.png");

const LH11: LineHeight = LineHeight::Absolute(Pixels(11.0));
const LH14: LineHeight = LineHeight::Absolute(Pixels(14.0));
const LH16: LineHeight = LineHeight::Absolute(Pixels(16.0));
const LH24: LineHeight = LineHeight::Absolute(Pixels(24.0));

pub fn view(state: &Plaza) -> Element<'_, Msg> {
    let menu = menu_bar([
        ("About", Msg::OpenWin(WinType::About)),
        ("Play History", Msg::OpenWin(WinType::History)),
        ("Ratings", Msg::OpenWin(WinType::Ratings)),
        ("Support Us", Msg::OpenWin(WinType::Support)),
    ]);

    let cover = render_cover(state);
    let meta = render_metadata(state);

    // .player-container: thin sunken border, 3px padding, margin 1px 1px 0 1px
    let content = container(
        d3_thin_sunken(
            container(row![cover, meta].align_y(Alignment::Center))
                .style(theme::panel)
                .width(Fill)
                .padding(3),
        )
        .width(Fill),
    )
    .padding(Padding {
        top: 1.0,
        right: 1.0,
        bottom: 0.0,
        left: 1.0,
    });

    let status = render_status(state);

    let mut col = column![menu, content, status];

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

    let area = mouse_area(img).interaction(iced::mouse::Interaction::Pointer);
    if song.id.is_empty() {
        area.into()
    } else {
        area.on_press(Msg::SongInfo(crate::state::SongInfoMsg::Open(
            song.id.clone(),
        )))
        .into()
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
    .line_height(LH14)
    .font(bold_font());

    let title = shaped(&song.title).size(14).line_height(LH14);

    // .player-meta: padding-left 8px (ps-sm-2)
    column![
        Space::new().height(2),
        artist,
        Space::new().height(8),
        title,
        Space::new().height(12),
        render_time_vol(state),
        Space::new().height(12),
        render_controls(state),
    ]
    .width(Fill)
    .padding(Padding {
        left: 8.0,
        ..Padding::ZERO
    })
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

    // .text-field .player-time: thin sunken, gray bg, 14px text on a 24px line
    let time_field = d3_thin_sunken(
        container(
            text(time_str)
                .size(14)
                .line_height(LH24)
                .center()
                .width(Fill),
        )
        .width(Fill)
        .style(theme::panel),
    );

    let vol_icon = image(static_image(VOLUME_IMG)).width(11).height(16);
    let vol = volume_slider(state.volume, vol_icon, Msg::Volume);

    row![
        container(time_field).width(Length::FillPortion(7)),
        Space::new().width(8),
        container(vol).width(Length::FillPortion(5)),
    ]
    .align_y(Alignment::Start)
    .into()
}

fn render_controls(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;
    let is_playing = state.is_playing();
    let is_streaming = state.is_streaming();
    let play_txt = if is_playing && !is_streaming {
        "Loading…"
    } else if is_playing {
        "Stop"
    } else {
        "Play"
    };
    let play_btn = bevel_button(
        text(play_txt)
            .size(11)
            .line_height(LH16)
            .center()
            .width(Fill),
    )
    .on_press(Msg::TogglePlay)
    .width(Fill);

    let is_current_song = state.reaction_song_id == song.id && !song.id.is_empty();
    let react_icon = match (is_current_song, state.reaction_rate) {
        (true, 2) => STAR_IMG,
        (true, 1) => HEART_IMG,
        _ => HEART_GRAY_IMG,
    };

    let react_btn = bevel_button(
        container(
            row![
                image(static_image(react_icon)).width(16).height(16),
                text(format!("{}", song.reactions))
                    .size(11)
                    .line_height(LH16),
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
        .center_x(Fill),
    )
    .on_press(Msg::React)
    .width(Fill);

    let left_btns = Row::new()
        .spacing(4)
        .push(container(play_btn).width(Length::FillPortion(7)))
        .push(container(react_btn).width(Length::FillPortion(5)));

    let user_msg = if state.user.is_some() {
        Msg::OpenWin(WinType::UserProfile)
    } else {
        Msg::OpenWin(WinType::UserLogin)
    };
    let user_btn = bevel_button(
        container(image(static_image(PERSON_IMG)).width(16).height(16)).center_x(Fill),
    )
    .on_press(user_msg)
    .width(Fill);

    let settings_btn =
        bevel_button(container(image(static_image(GEARS_IMG)).width(16).height(16)).center_x(Fill))
            .on_press(Msg::OpenWin(WinType::Settings))
            .width(Fill);

    let right_btns = Row::new()
        .spacing(4)
        .push(container(user_btn).width(Length::FillPortion(1)))
        .push(container(settings_btn).width(Length::FillPortion(1)));

    row![
        container(left_btns).width(Length::FillPortion(7)),
        Space::new().width(8),
        container(right_btns).width(Length::FillPortion(5)),
    ]
    .align_y(Alignment::Start)
    .into()
}

fn render_status(state: &Plaza) -> Element<'_, Msg> {
    let mut status_cells: Vec<(Element<Msg>, u16)> = vec![(
        text(format!("Listeners: {}", state.status.listeners))
            .size(11)
            .line_height(LH11)
            .into(),
        8,
    )];
    if let Some(ref u) = state.user {
        status_cells.push((
            text(format!("Logged in as: {}", u.username))
                .size(11)
                .line_height(LH11)
                .wrapping(iced::widget::text::Wrapping::None)
                .into(),
            4,
        ));
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
