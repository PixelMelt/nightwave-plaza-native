use crate::api::RatingEntry;
use crate::state::{Msg, Plaza, RatingsMsg};
use crate::views::bevel_button;
use crate::views::{
    bold_font, clickable_row, empty_panel, icon_like, loading_panel, paged_footer, paginate,
    shaped, song_list,
};
use iced::widget::{column, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let r = &state.ratings.range;
    let range_row = row![
        range_btn("All Time", "overtime", r == "overtime"),
        Space::new().width(4),
        range_btn("Monthly", "monthly", r == "monthly"),
        Space::new().width(4),
        range_btn("Weekly", "weekly", r == "weekly"),
    ]
    .width(Fill);

    let list_area = render_list(state);
    let pages_row = render_pagination(state);

    column![
        range_row,
        Space::new().height(2),
        list_area,
        Space::new().height(4),
        paged_footer(pages_row, wid, state.ratings.pages, state.ratings.total),
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
    let page = state.ratings.page;
    song_list(&state.ratings.list, move |i, entry| {
        let rank = (page - 1) * 25 + (i as u32) + 1;
        render_row(entry, rank)
    })
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
        row![text(entry.likes.to_string()).size(11), icon_like().size(11),].spacing(2),
        Space::new().width(16),
    ]
    .spacing(4)
    .padding([3, 4]);

    clickable_row(entry_content, &entry.song.id)
}

fn render_pagination(state: &Plaza) -> Element<'_, Msg> {
    paginate(
        state.ratings.page,
        state.ratings.pages,
        state.ratings.loading,
        &state.ratings.page_input,
        |p| Msg::Ratings(RatingsMsg::Page(p)),
        |s| Msg::Ratings(RatingsMsg::PageInput(s)),
        Msg::Ratings(RatingsMsg::PageSubmit),
    )
}

fn range_btn(label: &'static str, range: &str, active: bool) -> Element<'static, Msg> {
    bevel_button(text(label).size(10).center().width(Fill))
        .on_press(Msg::Ratings(RatingsMsg::Range(range.to_string())))
        .active(active)
        .padding([3, 10])
        .width(Fill)
        .into()
}
