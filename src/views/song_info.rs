use crate::state::{Msg, Plaza, SongInfoMsg};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{
    self, bold_font, close_btn, d3_sunken, d3_thin_sunken, format_date, format_time, shaped,
    status_bar,
};
use iced::widget::{
    column, container, horizontal_space, image, row, text, vertical_space, Space,
};
use iced::{Element, Fill};

const FAVORITE_GOLD: iced::Color = iced::Color::from_rgb(1.0, 0.827, 0.0);

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    if state.song_info.loading {
        let loading = container(text("Loading...").size(11))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill);
        let bottom = row![horizontal_space(), close_btn(wid)].padding([4, 2]);
        return column![loading, bottom].spacing(2).padding(2).into();
    }

    let (artist, album, title_str, length, likes, first_played, art_handle) =
        if let Some(ref info) = state.song_info.data {
            (
                &info.data.artist,
                &info.data.album,
                &info.data.title,
                info.data.length,
                info.stats.likes,
                info.stats.first_played_at,
                state.song_info.artwork.as_ref().or(state.artwork_handle.as_ref()),
            )
        } else {
            let song = &state.status.song;
            (
                &song.artist,
                &song.album,
                &song.title,
                song.length,
                song.reactions,
                None,
                state.artwork_handle.as_ref(),
            )
        };

    let art_content: Element<Msg> = if let Some(h) = art_handle {
        image(h.clone()).width(100).height(100).into()
    } else {
        Space::new(100, 100).into()
    };
    let art: Element<Msg> = d3_thin_sunken(container(art_content).style(theme::sunken_inner)).into();

    let info = column![
        text("Artist:").size(10).font(bold),
        shaped(artist).size(11),
        vertical_space().height(2),
        text("Album:").size(10).font(bold),
        shaped(album).size(11),
        vertical_space().height(2),
        text("Title:").size(10).font(bold),
        shaped(title_str).size(11),
        vertical_space().height(4),
        row![
            widgets::icon_clock().size(10),
            Space::with_width(2),
            text(format_time(length)).size(10),
            Space::with_width(8),
            widgets::icon_like().size(10),
            text(format!(" {}", likes)).size(10),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(1);

    let info_row = row![info, horizontal_space(), art].padding(4);
    let panel = d3_sunken(
        container(info_row).style(theme::panel).width(Fill).padding(4),
    );

    let favorited = state.song_info.favorite_id.is_some();
    let can_favorite = state.song_info.data.is_some() && !state.song_info.fav_sending;
    let fav_btn = bevel_button(
        text(widgets::IC_FAVORITE)
            .font(widgets::ICON_FONT)
            .size(12)
            .center()
            .width(Fill)
            .color(if favorited { FAVORITE_GOLD } else { theme::BLACK })
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .maybe_on_press(can_favorite.then_some(Msg::SongInfo(SongInfoMsg::ToggleFavorite)))
    .width(44);
    let bottom = row![fav_btn, horizontal_space(), close_btn(wid)].padding([4, 2]);

    let status_text = first_played
        .map(|fp| format!("First Played: {}", format_date(fp)))
        .unwrap_or_default();
    let status = status_bar(vec![text(status_text).size(10).width(Fill).into()]);

    column![panel, bottom, status].spacing(2).padding(2).into()
}
