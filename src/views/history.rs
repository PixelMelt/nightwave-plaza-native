use crate::api::HistoryEntry;
use crate::state::{HistoryMsg, Msg, Plaza};
use crate::views::{
    clickable_row, empty_panel, format_date, format_timestamp_day, format_timestamp_time,
    link_button, loading_panel, paged_footer, shaped, song_list, BOLD,
};
use iced::widget::{column, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let header: Element<Msg> = match state.history.date_range {
        Some(range) => row![
            text(format!(
                "Displaying history: {} \u{2014} {}",
                format_date(range.from_date),
                format_date(range.to_date)
            ))
            .size(10),
            Space::new().width(Fill),
            link_button(
                "Last.fm",
                10,
                Some(Msg::OpenUrl("https://plaza.one/lastfm".into()))
            ),
        ]
        .align_y(iced::Alignment::Center)
        .padding([2, 4])
        .into(),
        None => Space::new().height(0).into(),
    };

    let list = if state.history.pager.loading {
        loading_panel()
    } else if state.history.list.is_empty() {
        empty_panel("No data")
    } else {
        song_list(&state.history.list, |_, entry| entry_row(entry))
    };

    column![
        header,
        list,
        Space::new().height(4),
        paged_footer(wid, &state.history.pager, |m| Msg::History(
            HistoryMsg::Page(m)
        )),
    ]
    .padding(4)
    .height(Fill)
    .into()
}

fn entry_row(entry: &HistoryEntry) -> Element<'_, Msg> {
    let content = row![
        column![
            shaped(&entry.song.artist).size(11).font(BOLD),
            shaped(&entry.song.title).size(11),
        ]
        .spacing(1)
        .width(Fill),
        column![
            text(format_timestamp_day(entry.played_at)).size(10),
            text(format_timestamp_time(entry.played_at)).size(10),
        ]
        .width(78)
        .align_x(iced::Alignment::End),
        Space::new().width(16),
    ]
    .spacing(4)
    .padding([3, 4]);
    clickable_row(content, &entry.song.id)
}
