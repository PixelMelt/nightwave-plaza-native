use crate::state::{HistoryMsg, Msg, Plaza};
use crate::views::{
    bold_font, clickable_row, empty_panel, format_timestamp_day, format_timestamp_time,
    link_button, loading_panel, paged_footer, paginate, shaped, song_list,
};
use iced::widget::{column, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let header = render_header(state);
    let list_area = render_list_area(state);
    let pages_row = render_pagination(state);

    column![
        header,
        list_area,
        Space::new().height(4),
        paged_footer(pages_row, wid, &state.history.pager),
    ]
    .padding(4)
    .height(Fill)
    .into()
}

fn render_header(state: &Plaza) -> Element<'_, Msg> {
    if state.history.date_from.is_empty() {
        return Space::new().height(0).into();
    }
    row![
        text(format!(
            "Displaying history: {} \u{2014} {}",
            state.history.date_from, state.history.date_to
        ))
        .size(10),
        Space::new().width(iced::Fill),
        link_button(
            "Last.fm",
            10,
            Some(Msg::OpenUrl("https://plaza.one/lastfm".into()))
        ),
    ]
    .align_y(iced::Alignment::Center)
    .padding([2, 4])
    .into()
}

fn render_list_area(state: &Plaza) -> Element<'_, Msg> {
    if state.history.pager.loading {
        return loading_panel();
    }
    if state.history.list.is_empty() {
        return empty_panel("No data");
    }
    let bold = bold_font();
    song_list(&state.history.list, move |_, entry| render_row(entry, bold))
}

fn render_row(entry: &crate::api::HistoryEntry, bold: iced::Font) -> Element<'_, Msg> {
    let day = format_timestamp_day(entry.played_at);
    let time = format_timestamp_time(entry.played_at);

    let entry_content = row![
        column![
            shaped(&entry.song.artist).size(11).font(bold),
            shaped(&entry.song.title).size(11),
        ]
        .spacing(1)
        .width(Fill),
        column![text(day).size(10), text(time).size(10)]
            .spacing(0)
            .width(78)
            .align_x(iced::Alignment::End),
        Space::new().width(16),
    ]
    .spacing(4)
    .padding([3, 4]);

    clickable_row(entry_content, &entry.song.id)
}

fn render_pagination(state: &Plaza) -> Element<'_, Msg> {
    paginate(&state.history.pager, |m| Msg::History(HistoryMsg::Page(m)))
}
