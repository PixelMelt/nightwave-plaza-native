use crate::state::{FavoritesMsg, Msg, Plaza, WinType};
use crate::theme;
use crate::views::bevel_button;
use crate::views::{
    bold_font, clickable_row, empty_panel, format_date, link_button, loading_panel, paginate,
    shaped, song_list, status_bar,
};
use iced::widget::{column, container, image, row, text, Space};
use iced::{Element, Fill};

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
        song_list(&state.favorites.list, move |_, entry| {
            let deleted = state.favorites.deleted.contains(&entry.id);
            let art = entry
                .song
                .thumb_url()
                .and_then(|url| state.favorites.artwork.get(url));
            render_row(entry, bold, deleted, art)
        })
    }
}

fn render_row<'a>(
    entry: &'a crate::api::FavoriteEntry,
    bold: iced::Font,
    deleted: bool,
    art: Option<&image::Handle>,
) -> Element<'a, Msg> {
    let muted = iced::Color::from_rgb(0.5, 0.5, 0.5);
    let row_color = if deleted { muted } else { theme::BLACK };

    let thumb: Element<Msg> = match art {
        Some(handle) => image(handle.clone()).width(54).height(54).into(),
        None => Space::new().width(54).height(54).into(),
    };
    let art_cell = container(thumb).width(62).padding([2, 4]);

    let info = column![
        shaped(&entry.song.artist)
            .size(11)
            .font(bold)
            .color(row_color),
        shaped(&entry.song.title).size(11).color(row_color),
        text(format_date(entry.created_at)).size(10).color(muted),
    ]
    .spacing(1)
    .width(Fill);

    let clickable: Element<Msg> = if deleted {
        info.into()
    } else {
        clickable_row(info, &entry.song.id)
    };

    let action: Element<Msg> = if deleted {
        text("Removed").size(10).color(muted).into()
    } else {
        link_button("Remove", 10, Some(Msg::Favorites(FavoritesMsg::Delete(entry.id))))
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
    paginate(
        state.favorites.page,
        state.favorites.pages,
        state.favorites.loading,
        &state.favorites.page_input,
        |p| Msg::Favorites(FavoritesMsg::Page(p)),
        |s| Msg::Favorites(FavoritesMsg::PageInput(s)),
        Msg::Favorites(FavoritesMsg::PageSubmit),
    )
}
