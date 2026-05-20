use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{self, d3_raised, d3_sunken, divider, pagination, shaped, status_bar};
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
    let r = &state.ratings_range;

    let range_row = row![
        range_btn("All Time", "overtime", r == "overtime"),
        Space::with_width(4),
        range_btn("Monthly", "monthly", r == "monthly"),
        Space::with_width(4),
        range_btn("Weekly", "weekly", r == "weekly"),
    ];

    let list_area: Element<Msg> = if state.ratings_loading {
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
    } else {
        let mut list = Column::new().spacing(0).width(Fill);

        if state.ratings.is_empty() {
            list = list.push(
                container(text("No data").size(11))
                    .padding(8)
                    .center_x(Fill)
                    .width(Fill),
            );
        } else {
            for (i, entry) in state.ratings.iter().enumerate() {
                let rank = (state.ratings_page - 1) * 25 + (i as u32) + 1;

                let entry_content = row![
                    text(format!("{:03}", rank)).size(11).width(28),
                    column![
                        shaped(entry.song.artist.clone()).size(11).font(bold),
                        shaped(entry.song.title.clone()).size(11),
                    ]
                    .spacing(1)
                    .width(Fill),
                    row![
                        text(format!("{}", entry.likes)).size(11),
                        widgets::icon_like().size(11),
                    ]
                    .spacing(2),
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
                if i < state.ratings.len() - 1 {
                    list = list.push(divider());
                }
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

    let pages_row = if state.ratings_pages > 1 {
        let prev_msg = if state.ratings_page > 1 && !state.ratings_loading {
            Some(Msg::RatingsPage(state.ratings_page - 1))
        } else {
            None
        };
        let next_msg = if state.ratings_page < state.ratings_pages && !state.ratings_loading {
            Some(Msg::RatingsPage(state.ratings_page + 1))
        } else {
            None
        };
        pagination(
            state.ratings_page,
            state.ratings_pages,
            &state.ratings_page_input,
            state.ratings_loading,
            Msg::RatingsPageInput,
            Msg::RatingsPageSubmit,
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
        text(format!("Pages: {}", state.ratings_pages))
            .size(10)
            .into(),
        text(format!("Songs: {}", state.ratings_total))
            .size(10)
            .into(),
    ]);

    column![
        range_row,
        Space::with_height(2),
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

fn range_btn(label: &str, range: &str, active: bool) -> Element<'static, Msg> {
    d3_raised(
        button(text(label.to_string()).size(10).center().width(Fill))
            .on_press(Msg::RatingsRange(range.to_string()))
            .style(if active {
                theme::active_tab
            } else {
                theme::raised
            })
            .padding([3, 10]),
    )
    .into()
}
