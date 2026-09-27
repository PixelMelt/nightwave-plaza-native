use crate::state::{Msg, Plaza, SongInfoMsg};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::{
    close_btn, d3_thin_sunken, empty_panel, format_date, format_time, icon, icon_like,
    loading_panel, shaped, status_bar, sunken_panel, BOLD, IC_CLOCK, IC_FAVORITE,
};
use iced::widget::{column, container, image, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let info = &state.song_info;
    let bottom_padding = [4, 2];

    let Some(song) = &info.data else {
        let body = match &info.error {
            Some(err) => empty_panel(err),
            None => loading_panel(),
        };
        let bottom = row![Space::new().width(Fill), close_btn(wid)].padding(bottom_padding);
        return column![body, bottom].spacing(2).padding(2).into();
    };

    let art: Element<Msg> = match &info.artwork {
        Some(handle) => image(handle.clone()).width(100).height(100).into(),
        None => Space::new().width(100).height(100).into(),
    };
    let art = d3_thin_sunken(container(art).style(theme::sunken_inner));

    let label = |s| text(s).size(10).font(BOLD);
    let details = column![
        label("Artist:"),
        shaped(&song.data.artist).size(11),
        Space::new().height(2),
        label("Album:"),
        shaped(&song.data.album).size(11),
        Space::new().height(2),
        label("Title:"),
        shaped(&song.data.title).size(11),
        Space::new().height(4),
        row![
            icon(IC_CLOCK).size(10),
            Space::new().width(2),
            text(format_time(song.data.length)).size(10),
            Space::new().width(8),
            icon_like().size(10),
            text(format!(" {}", song.stats.likes)).size(10),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(1);

    let panel = sunken_panel(row![details, Space::new().width(Fill), art].padding(4));

    let fav_color = if info.favorite_id.is_some() {
        theme::FAVORITE_GOLD
    } else {
        theme::BLACK
    };
    let fav_btn = bevel_button(
        icon(IC_FAVORITE)
            .size(12)
            .center()
            .width(Fill)
            .color(fav_color),
    )
    .maybe_on_press((!info.fav_sending).then_some(Msg::SongInfo(SongInfoMsg::ToggleFavorite)))
    .width(44);
    let bottom = row![fav_btn, Space::new().width(Fill), close_btn(wid)].padding(bottom_padding);

    let first_played = song
        .stats
        .first_played_at
        .map(|ts| format!("First Played: {}", format_date(ts)))
        .unwrap_or_default();
    let status = status_bar(vec![(text(first_played).size(10).into(), 1)]);

    column![panel, bottom, status].spacing(2).padding(2).into()
}
