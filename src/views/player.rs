use crate::state::{Msg, Plaza, SongInfoMsg, WinType};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::volume::volume_slider;
use crate::views::{button, d3_thin_sunken, format_time, menu_bar, shaped, status_bar, Png, BOLD};
use iced::widget::text::{LineHeight, Wrapping};
use iced::widget::{column, container, image, mouse_area, row, text, Space};
use iced::{Alignment, Element, Fill, Length, Padding, Pixels};

static VOLUME: Png = Png::new(include_bytes!("../assets/img/volume.png"));
static PERSON: Png = Png::new(include_bytes!("../assets/icons/person.png"));
static GEARS: Png = Png::new(include_bytes!("../assets/icons/gears.png"));
static HEART: Png = Png::new(include_bytes!("../assets/icons/heart.png"));
static HEART_GRAY: Png = Png::new(include_bytes!("../assets/icons/heart_gray.png"));
static STAR: Png = Png::new(include_bytes!("../assets/icons/star.png"));

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

    let player = container(
        d3_thin_sunken(
            container(row![cover(state), metadata(state)].align_y(Alignment::Center))
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

    let mut col = column![menu, player, status(state)];
    if let Some(err) = &state.error_msg {
        col = col.push(error_bar(err));
    }
    col.into()
}

fn cover(state: &Plaza) -> Element<'_, Msg> {
    let art: Element<Msg> = match &state.artwork {
        Some(handle) => image(handle.clone()).width(112).height(112).into(),
        None => Space::new().width(112).height(112).into(),
    };
    let song_id = &state.status.song.id;
    let area = mouse_area(d3_thin_sunken(container(art).style(theme::cover)))
        .interaction(iced::mouse::Interaction::Pointer);
    if song_id.is_empty() {
        area.into()
    } else {
        area.on_press(Msg::SongInfo(SongInfoMsg::Open(song_id.clone())))
            .into()
    }
}

fn metadata(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;
    let artist = if song.artist.is_empty() {
        "..."
    } else {
        &song.artist
    };
    column![
        Space::new().height(2),
        shaped(artist).size(14).line_height(LH14).font(BOLD),
        Space::new().height(8),
        shaped(&song.title).size(14).line_height(LH14),
        Space::new().height(12),
        time_and_volume(state),
        Space::new().height(12),
        controls(state),
    ]
    .width(Fill)
    .padding(Padding {
        left: 8.0,
        ..Padding::ZERO
    })
    .into()
}

fn time_and_volume(state: &Plaza) -> Element<'_, Msg> {
    let length = state.status.song.length;
    let time_str = match &state.time_notice {
        Some((notice, _)) => notice.clone(),
        None if length > 0.0 => format!(
            "{} / {}",
            format_time(state.song_position()),
            format_time(length)
        ),
        None => "...".into(),
    };
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
    let volume = volume_slider(
        state.volume,
        VOLUME.image().width(11).height(16),
        Msg::Volume,
    );
    row![
        container(time_field).width(Length::FillPortion(7)),
        Space::new().width(8),
        container(volume).width(Length::FillPortion(5)),
    ]
    .into()
}

fn controls(state: &Plaza) -> Element<'_, Msg> {
    let song = &state.status.song;
    let play_label = match (state.player.is_playing(), state.player.is_streaming()) {
        (true, false) => "Loading…",
        (true, true) => "Stop",
        (false, _) => "Play",
    };
    let play_btn = bevel_button(
        text(play_label)
            .size(11)
            .line_height(LH16)
            .center()
            .width(Fill),
    )
    .on_press(Msg::TogglePlay)
    .width(Fill);

    let react_icon = match state.reaction.rate_for(&song.id) {
        2 => &STAR,
        1 => &HEART,
        _ => &HEART_GRAY,
    };
    let react_btn = bevel_button(
        container(
            row![
                react_icon.image().width(16).height(16),
                text(song.reactions.to_string()).size(11).line_height(LH16),
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
        .center_x(Fill),
    )
    .on_press(Msg::React)
    .width(Fill);

    let icon_btn = |png: &'static Png, msg| {
        bevel_button(container(png.image().width(16).height(16)).center_x(Fill))
            .on_press(msg)
            .width(Fill)
    };
    let user_win = if state.user().is_some() {
        WinType::UserProfile
    } else {
        WinType::UserLogin
    };
    let user_btn = icon_btn(&PERSON, Msg::OpenWin(user_win));
    let settings_btn = icon_btn(&GEARS, Msg::OpenWin(WinType::Settings));

    let left = row![
        container(play_btn).width(Length::FillPortion(7)),
        container(react_btn).width(Length::FillPortion(5)),
    ]
    .spacing(4);
    let right = row![
        container(user_btn).width(Fill),
        container(settings_btn).width(Fill),
    ]
    .spacing(4);

    row![
        container(left).width(Length::FillPortion(7)),
        Space::new().width(8),
        container(right).width(Length::FillPortion(5)),
    ]
    .into()
}

fn status(state: &Plaza) -> Element<'_, Msg> {
    let mut cells: Vec<(Element<Msg>, u16)> = vec![(
        text(format!("Listeners: {}", state.status.listeners))
            .size(11)
            .line_height(LH11)
            .into(),
        8,
    )];
    if let Some(user) = state.user() {
        cells.push((
            text(format!("Logged in as: {}", user.username))
                .size(11)
                .line_height(LH11)
                .wrapping(Wrapping::None)
                .into(),
            4,
        ));
    }
    status_bar(cells)
}

fn error_bar(err: &str) -> Element<'_, Msg> {
    container(
        row![
            text(err).size(10).color(theme::ERROR_RED),
            Space::new().width(Fill),
            button("x", Length::Shrink)
                .on_press(Msg::DismissErr)
                .padding(2),
        ]
        .align_y(Alignment::Center)
        .padding([2, 4]),
    )
    .style(theme::panel)
    .width(Fill)
    .into()
}
