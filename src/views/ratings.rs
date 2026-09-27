use crate::api::{RatingEntry, RatingsRange};
use crate::state::{Msg, Plaza, RatingsMsg};
use crate::views::{
    button, clickable_row, empty_panel, icon_like, loading_panel, paged_footer, shaped, song_list,
    BOLD,
};
use iced::widget::{column, row, text, Space};
use iced::{Element, Fill};

const PAGE_SIZE: u32 = 25;

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let range_btn = |label, range| {
        button(label, Fill)
            .on_press(Msg::Ratings(RatingsMsg::Range(range)))
            .active(state.ratings.range == range)
            .padding([3, 10])
    };
    let ranges = row![
        range_btn("All Time", RatingsRange::AllTime),
        Space::new().width(4),
        range_btn("Monthly", RatingsRange::Monthly),
        Space::new().width(4),
        range_btn("Weekly", RatingsRange::Weekly),
    ]
    .width(Fill);

    let list = if state.ratings.pager.loading {
        loading_panel()
    } else if state.ratings.list.is_empty() {
        empty_panel("No data")
    } else {
        let first_rank = (state.ratings.pager.page - 1) * PAGE_SIZE + 1;
        song_list(&state.ratings.list, move |i, entry| {
            entry_row(entry, first_rank + i as u32)
        })
    };

    column![
        ranges,
        Space::new().height(2),
        list,
        Space::new().height(4),
        paged_footer(wid, &state.ratings.pager, |m| Msg::Ratings(
            RatingsMsg::Page(m)
        )),
    ]
    .padding(4)
    .height(Fill)
    .into()
}

fn entry_row(entry: &RatingEntry, rank: u32) -> Element<'_, Msg> {
    let content = row![
        text(format!("{rank:03}")).size(11).width(28),
        column![
            shaped(&entry.song.artist).size(11).font(BOLD),
            shaped(&entry.song.title).size(11),
        ]
        .spacing(1)
        .width(Fill),
        row![text(entry.likes.to_string()).size(11), icon_like().size(11)].spacing(2),
        Space::new().width(16),
    ]
    .spacing(4)
    .padding([3, 4]);
    clickable_row(content, &entry.song.id)
}
