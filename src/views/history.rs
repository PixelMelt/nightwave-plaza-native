use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{
    bold_font, close_btn, divider, empty_panel, format_timestamp_day, format_timestamp_time,
    loading_panel, pagination, scroll_panel, shaped, status_bar, LINK_COLOR,
};
use iced::widget::{button, column, row, text, Column, Space};
use iced::{Element, Fill, Theme};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let header = render_header(state);
    let list_area = render_list_area(state);
    let pages_row = render_pagination(state);

    let bottom = row![pages_row, Space::new().width(iced::Fill), close_btn(wid)]
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);

    let status = status_bar(vec![
        text(format!("Pages: {}", state.history.pages))
            .size(10)
            .into(),
        text(format!("Songs: {}", state.history.total))
            .size(10)
            .into(),
    ]);

    column![
        header,
        list_area,
        Space::new().height(4),
        bottom,
        Space::new().height(2),
        status,
    ]
    .padding(4)
    .height(Fill)
    .into()
}

fn render_header(state: &Plaza) -> Element<'_, Msg> {
    if !state.history.date_from.is_empty() {
        row![
            text(format!(
                "Displaying history: {} \u{2014} {}",
                state.history.date_from, state.history.date_to
            ))
            .size(10),
            Space::new().width(iced::Fill),
            button(text("Last.fm").size(10).color(LINK_COLOR))
                .on_press(Msg::OpenUrl("https://plaza.one/lastfm".into()))
                .style(|_: &Theme, _| button::Style {
                    background: None,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    text_color: LINK_COLOR,
                    snap: false,
                })
                .padding(0),
        ]
        .align_y(iced::Alignment::Center)
        .padding([2, 4])
        .into()
    } else {
        Space::new().height(0).into()
    }
}

fn render_list_area(state: &Plaza) -> Element<'_, Msg> {
    if state.history.loading {
        loading_panel()
    } else if state.history.list.is_empty() {
        empty_panel("No data")
    } else {
        let bold = bold_font();
        let mut list = Column::new().spacing(0).width(Fill);
        let len = state.history.list.len();
        for (i, entry) in state.history.list.iter().enumerate() {
            list = list.push(render_row(entry, bold));
            if i < len - 1 {
                list = list.push(divider());
            }
        }
        scroll_panel(list)
    }
}

fn render_row<'a>(entry: &'a crate::api::HistoryEntry, bold: iced::Font) -> Element<'a, Msg> {
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

    if !entry.song.id.is_empty() {
        button(entry_content)
            .on_press(Msg::SongInfo(crate::state::SongInfoMsg::Open(
                entry.song.id.clone(),
            )))
            .style(theme::list_row_btn)
            .padding(0)
            .width(Fill)
            .into()
    } else {
        entry_content.into()
    }
}

fn render_pagination(state: &Plaza) -> Element<'_, Msg> {
    if state.history.pages > 1 {
        let prev_msg = (state.history.page > 1 && !state.history.loading).then_some(Msg::History(
            crate::state::HistoryMsg::Page(state.history.page.saturating_sub(1)),
        ));
        let next_msg = (state.history.page < state.history.pages && !state.history.loading)
            .then_some(Msg::History(crate::state::HistoryMsg::Page(
                state.history.page.saturating_add(1),
            )));

        pagination(
            state.history.page,
            state.history.pages,
            &state.history.page_input,
            state.history.loading,
            |s| Msg::History(crate::state::HistoryMsg::PageInput(s)),
            Msg::History(crate::state::HistoryMsg::PageSubmit),
            prev_msg,
            next_msg,
        )
    } else {
        Space::new().width(0).into()
    }
}
