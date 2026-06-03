use crate::api::RatingEntry;
use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{
    self, bold_font, close_btn, divider, empty_panel, loading_panel, pagination, scroll_panel,
    shaped, status_bar,
};
use iced::widget::{button, column, horizontal_space, row, text, Column, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let r = &state.ratings.range;
    let range_row = row![
        range_btn("All Time", "overtime", r == "overtime"),
        Space::with_width(4),
        range_btn("Monthly", "monthly", r == "monthly"),
        Space::with_width(4),
        range_btn("Weekly", "weekly", r == "weekly"),
    ]
    .width(Fill);

    let list_area = render_list(state);
    let pages_row = render_pagination(state);

    let bottom = row![pages_row, horizontal_space(), close_btn(wid)]
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);

    let status = status_bar(vec![
        text(format!("Pages: {}", state.ratings.pages))
            .size(10)
            .into(),
        text(format!("Songs: {}", state.ratings.total))
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

fn render_list(state: &Plaza) -> Element<'_, Msg> {
    if state.ratings.loading {
        return loading_panel();
    }

    if state.ratings.list.is_empty() {
        return empty_panel("No data");
    }

    let mut list = Column::new().spacing(0).width(Fill);
    for (i, entry) in state.ratings.list.iter().enumerate() {
        let rank = (state.ratings.page - 1) * 25 + (i as u32) + 1;
        list = list.push(render_row(entry, rank));
        if i < state.ratings.list.len() - 1 {
            list = list.push(divider());
        }
    }
    scroll_panel(list)
}

fn render_row(entry: &RatingEntry, rank: u32) -> Element<'_, Msg> {
    let bold = bold_font();

    let entry_content = row![
        text(format!("{:03}", rank)).size(11).width(28),
        column![
            shaped(&entry.song.artist).size(11).font(bold),
            shaped(&entry.song.title).size(11),
        ]
        .spacing(1)
        .width(Fill),
        row![
            text(entry.likes.to_string()).size(11),
            widgets::icon_like().size(11),
        ]
        .spacing(2),
        Space::with_width(16),
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
    if state.ratings.pages <= 1 {
        return Space::with_width(0).into();
    }

    let prev_msg = (state.ratings.page > 1 && !state.ratings.loading).then_some(Msg::Ratings(
        crate::state::RatingsMsg::Page(state.ratings.page - 1),
    ));
    let next_msg = (state.ratings.page < state.ratings.pages && !state.ratings.loading).then_some(
        Msg::Ratings(crate::state::RatingsMsg::Page(state.ratings.page + 1)),
    );

    pagination(
        state.ratings.page,
        state.ratings.pages,
        &state.ratings.page_input,
        state.ratings.loading,
        |s| Msg::Ratings(crate::state::RatingsMsg::PageInput(s)),
        Msg::Ratings(crate::state::RatingsMsg::PageSubmit),
        prev_msg,
        next_msg,
    )
}

fn range_btn(label: &'static str, range: &str, active: bool) -> Element<'static, Msg> {
    bevel_button(text(label).size(10).center().width(Fill))
        .on_press(Msg::Ratings(crate::state::RatingsMsg::Range(
            range.to_string(),
        )))
        .active(active)
        .padding([3, 10])
        .width(Fill)
        .into()
}
