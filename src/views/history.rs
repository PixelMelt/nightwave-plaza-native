use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{
    d3_raised, d3_sunken, divider, format_timestamp_day, format_timestamp_time, pagination, shaped,
    status_bar,
};
use iced::widget::{
    button, column, container, horizontal_space, image, row, scrollable, text, Column, Space,
};
use iced::{Element, Fill};

const LOADING_IMG: &[u8] = include_bytes!("../../assets/icons/loading.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    let header: Element<Msg> = if !state.history_date_from.is_empty() {
        row![
            text(format!(
                "Displaying history: {} \u{2014} {}",
                state.history_date_from, state.history_date_to
            ))
            .size(10),
            horizontal_space(),
            text("Last.fm")
                .size(10)
                .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
        ]
        .padding([2, 4])
        .into()
    } else {
        Space::with_height(0).into()
    };

    let list_area: Element<Msg> = if state.history_loading {
        d3_sunken(
            container(
                image(image::Handle::from_bytes(LOADING_IMG))
                    .width(36)
                    .height(36),
            )
            .style(theme::sunken_inner)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    } else if state.history.is_empty() {
        d3_sunken(
            container(text("No data").size(11))
                .style(theme::sunken_inner)
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    } else {
        let mut list = Column::new().spacing(0).width(Fill);

        for (i, entry) in state.history.iter().enumerate() {
            let day = format_timestamp_day(entry.played_at);
            let time = format_timestamp_time(entry.played_at);

            let entry_content = row![
                column![
                    shaped(entry.song.artist.clone()).size(11).font(bold),
                    shaped(entry.song.title.clone()).size(11),
                ]
                .spacing(1)
                .width(Fill),
                column![text(day).size(10), text(time).size(10),]
                    .spacing(0)
                    .width(78)
                    .align_x(iced::Alignment::End),
                Space::with_width(16),
            ]
            .spacing(4)
            .padding([3, 4]);

            let entry_row: Element<Msg> = if !entry.song.id.is_empty() {
                button(entry_content)
                    .on_press(Msg::OpenSongInfo(entry.song.id.clone()))
                    .style(theme::list_row_btn)
                    .padding(0)
                    .width(Fill)
                    .into()
            } else {
                entry_content.into()
            };

            list = list.push(entry_row);
            if i < state.history.len() - 1 {
                list = list.push(divider());
            }
        }

        d3_sunken(
            container(scrollable(list).height(Fill).style(theme::scrollbar))
                .style(theme::sunken_inner)
                .width(Fill)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    };

    let pages_row = if state.history_pages > 1 {
        let prev_msg = if state.history_page > 1 && !state.history_loading {
            Some(Msg::HistoryPage(state.history_page - 1))
        } else {
            None
        };
        let next_msg = if state.history_page < state.history_pages && !state.history_loading {
            Some(Msg::HistoryPage(state.history_page + 1))
        } else {
            None
        };
        pagination(
            state.history_page,
            state.history_pages,
            &state.history_page_input,
            state.history_loading,
            Msg::HistoryPageInput,
            Msg::HistoryPageSubmit,
            prev_msg,
            next_msg,
        )
    } else {
        Space::with_width(0).into()
    };

    let close_btn = d3_raised(
        button(text("Close").size(11).center().width(80))
            .on_press(Msg::CloseWin(wid))
            .width(80)
            .style(theme::raised),
    );

    let bottom = row![pages_row, horizontal_space(), close_btn]
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);

    let status = status_bar(vec![
        text(format!("Pages: {}", state.history_pages))
            .size(10)
            .into(),
        text(format!("Songs: {}", state.history_total))
            .size(10)
            .into(),
    ]);

    column![
        header,
        list_area,
        Space::with_height(4),
        bottom,
        Space::with_height(2),
        status,
    ]
    .padding(4)
    .height(Fill)
    .into()
}
