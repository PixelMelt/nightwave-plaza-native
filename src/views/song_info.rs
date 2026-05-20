use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{
    self, d3_raised, d3_sunken, format_date, format_time, shaped, status_bar,
};
use iced::widget::{
    button, column, container, horizontal_space, image, row, text, vertical_space, Space,
};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    // ── Loading state ───────────────────────────────────────────
    if state.song_info_loading {
        let loading = container(text("Loading...").size(11).center().width(Fill))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill);
        let close = d3_raised(
            button(text("Close").size(11).center().width(80))
                .on_press(Msg::CloseWin(wid))
                .width(80)
                .style(theme::raised),
        );
        let bottom = row![horizontal_space(), close].padding([4, 2]);
        return column![loading, bottom].spacing(2).padding(2).into();
    }

    // ── Song data: use fetched SongResponse if available, otherwise current song ──
    let (artist, album, title_str, length, likes, first_played, art_handle) =
        if let Some(ref info) = state.song_info {
            (
                info.data.artist.clone(),
                info.data.album.clone(),
                info.data.title.clone(),
                info.data.length,
                info.stats.likes,
                info.stats.first_played_at,
                state
                    .song_info_artwork
                    .as_ref()
                    .or(state.artwork_handle.as_ref()),
            )
        } else {
            let song = &state.status.song;
            (
                song.artist.clone(),
                song.album.clone(),
                song.title.clone(),
                song.length,
                song.reactions,
                None,
                state.artwork_handle.as_ref(),
            )
        };

    // ── Artwork (right side, ~42% width) (#40) ──────────────────
    let art: Element<Msg> = if let Some(h) = art_handle {
        d3_sunken(container(image(h.clone()).width(100).height(100)).style(theme::sunken_inner))
            .into()
    } else {
        d3_sunken(container(Space::new(100, 100)).style(theme::sunken_inner)).into()
    };

    // ── Song info fields ────────────────────────────────────────
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
        // clock icon + song length, heart icon + likes (matching webapp WindowSong.vue)
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
        container(info_row)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(4),
    );

    // ── Button row: Play Preview (disabled), Close (#38) ────────
    let preview_btn = d3_raised(
        button(text("Play Preview").size(11).center().width(100))
            .style(theme::raised)
            .width(100),
    );
    let close = d3_raised(
        button(text("Close").size(11).center().width(80))
            .on_press(Msg::CloseWin(wid))
            .width(80)
            .style(theme::raised),
    );
    let bottom = row![preview_btn, horizontal_space(), close].padding([4, 2]);

    // ── Status bar: First Played date (#41) ─────────────────────
    let status = if let Some(fp) = first_played {
        status_bar(vec![text(format!("First Played: {}", format_date(fp)))
            .size(10)
            .width(Fill)
            .into()])
    } else {
        status_bar(vec![text("").size(10).width(Fill).into()])
    };

    column![panel, bottom, status].spacing(2).padding(2).into()
}
