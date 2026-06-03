use crate::state::{FavoritesMsg, Msg, Plaza, WinType};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{
    bold_font, divider, empty_panel, format_date, loading_panel, pagination, scroll_panel, shaped,
    status_bar, LINK_COLOR,
};
use iced::widget::{button, column, container, image, row, text, Column, Space};
use iced::{Element, Fill, Theme};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let list_area = render_list_area(state);
    let pages_row = render_pagination(state);

    let export_btn = bevel_button(text("Export").size(11).center())
        .on_press(Msg::OpenWin(WinType::UserFavoritesExport))
        .padding([4, 12]);
    let close_btn = bevel_button(text("Close").size(11).center())
        .on_press(Msg::CloseWin(wid))
        .padding([4, 12]);

    let bottom = row![
        pages_row,
        Space::new().width(iced::Fill),
        export_btn,
        Space::new().width(6),
        close_btn,
    ]
    .align_y(iced::Alignment::Center)
    .padding([4, 0]);

    let status = status_bar(vec![
        text(format!("Pages: {}", state.favorites.pages))
            .size(10)
            .into(),
        text(format!("Songs: {}", state.favorites.total))
            .size(10)
            .into(),
    ]);

    column![
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

fn render_list_area(state: &Plaza) -> Element<'_, Msg> {
    if state.favorites.loading {
        loading_panel()
    } else if state.favorites.list.is_empty() {
        empty_panel("Your list is empty. Like a song to add it here.")
    } else {
        let bold = bold_font();
        let mut list = Column::new().spacing(0).width(Fill);
        let len = state.favorites.list.len();
        for (i, entry) in state.favorites.list.iter().enumerate() {
            let deleted = state.favorites.deleted.contains(&entry.id);
            let art = entry
                .song
                .thumb_url()
                .and_then(|url| state.favorites.artwork.get(url));
            list = list.push(render_row(entry, bold, deleted, art));
            if i < len - 1 {
                list = list.push(divider());
            }
        }
        scroll_panel(list)
    }
}

fn render_row<'a>(
    entry: &'a crate::api::FavoriteEntry,
    bold: iced::Font,
    deleted: bool,
    art: Option<&image::Handle>,
) -> Element<'a, Msg> {
    let muted = iced::Color::from_rgb(0.5, 0.5, 0.5);
    let (artist_color, title_color) = if deleted {
        (muted, muted)
    } else {
        (theme::BLACK, theme::BLACK)
    };

    let thumb: Element<Msg> = match art {
        Some(handle) => image(handle.clone()).width(54).height(54).into(),
        None => Space::new().width(54).height(54).into(),
    };
    let art_cell = container(thumb).width(62).padding([2, 4]);

    let info = column![
        shaped(&entry.song.artist)
            .size(11)
            .font(bold)
            .color(artist_color),
        shaped(&entry.song.title).size(11).color(title_color),
        text(format_date(entry.created_at)).size(10).color(muted),
    ]
    .spacing(1)
    .width(Fill);

    let song_id = entry.song.id.clone();
    let clickable: Element<Msg> = if !song_id.is_empty() && !deleted {
        button(info)
            .on_press(Msg::SongInfo(crate::state::SongInfoMsg::Open(song_id)))
            .style(theme::list_row_btn)
            .padding(0)
            .width(Fill)
            .into()
    } else {
        info.into()
    };

    let action: Element<Msg> = if deleted {
        text("Removed").size(10).color(muted).into()
    } else {
        button(text("Remove").size(10).color(LINK_COLOR))
            .on_press(Msg::Favorites(FavoritesMsg::Delete(entry.id)))
            .style(|_: &Theme, _| button::Style {
                background: None,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: LINK_COLOR,
                snap: false,
            })
            .padding(0)
            .into()
    };

    row![
        art_cell,
        clickable,
        Space::new().width(8),
        container(action).width(70).center_x(70),
    ]
    .width(Fill)
    .align_y(iced::Alignment::Center)
    .padding([3, 4])
    .into()
}

fn render_pagination(state: &Plaza) -> Element<'_, Msg> {
    if state.favorites.pages > 1 {
        let prev_msg = (state.favorites.page > 1 && !state.favorites.loading).then_some(
            Msg::Favorites(FavoritesMsg::Page(state.favorites.page.saturating_sub(1))),
        );
        let next_msg = (state.favorites.page < state.favorites.pages && !state.favorites.loading)
            .then_some(Msg::Favorites(FavoritesMsg::Page(
                state.favorites.page.saturating_add(1),
            )));

        pagination(
            state.favorites.page,
            state.favorites.pages,
            &state.favorites.page_input,
            state.favorites.loading,
            |s| Msg::Favorites(FavoritesMsg::PageInput(s)),
            Msg::Favorites(FavoritesMsg::PageSubmit),
            prev_msg,
            next_msg,
        )
    } else {
        Space::new().width(0).into()
    }
}
