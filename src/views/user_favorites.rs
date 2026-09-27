use crate::api::FavoriteEntry;
use crate::state::{FavoritesMsg, Msg, Plaza, WinType};
use crate::theme;
use crate::views::{
    button, clickable_row, empty_panel, format_date, link_button, loading_panel, pager_status,
    paginate, shaped, song_list, BOLD,
};
use iced::widget::{column, container, image, row, text, Space};
use iced::{Element, Fill, Length};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let favs = &state.favorites;
    let list = if favs.pager.loading {
        loading_panel()
    } else if favs.list.is_empty() {
        empty_panel("Your list is empty. Like a song to add it here.")
    } else {
        song_list(&favs.list, |_, entry| {
            let deleted = favs.deleted.contains(&entry.id);
            let art = entry.song.thumb_url().and_then(|url| favs.artwork.get(url));
            entry_row(entry, deleted, art)
        })
    };

    let export_btn = button("Export", Length::Shrink)
        .on_press(Msg::OpenWin(WinType::UserFavoritesExport))
        .padding([4, 12]);
    let close_btn = button("Close", Length::Shrink)
        .on_press(Msg::CloseWin(wid))
        .padding([4, 12]);
    let bottom = row![
        paginate(&favs.pager, |m| Msg::Favorites(FavoritesMsg::Page(m))),
        Space::new().width(Fill),
        export_btn,
        Space::new().width(6),
        close_btn,
    ]
    .align_y(iced::Alignment::Center)
    .padding([4, 0]);

    column![
        list,
        Space::new().height(4),
        bottom,
        Space::new().height(2),
        pager_status(&favs.pager),
    ]
    .padding(4)
    .height(Fill)
    .into()
}

fn entry_row<'a>(
    entry: &'a FavoriteEntry,
    deleted: bool,
    art: Option<&image::Handle>,
) -> Element<'a, Msg> {
    let color = if deleted {
        theme::DISABLED
    } else {
        theme::BLACK
    };

    let thumb: Element<Msg> = match art {
        Some(handle) => image(handle.clone()).width(54).height(54).into(),
        None => Space::new().width(54).height(54).into(),
    };

    let info = column![
        shaped(&entry.song.artist).size(11).font(BOLD).color(color),
        shaped(&entry.song.title).size(11).color(color),
        text(format_date(entry.created_at))
            .size(10)
            .color(theme::DISABLED),
    ]
    .spacing(1)
    .width(Fill);

    let (info, action): (Element<Msg>, Element<Msg>) = if deleted {
        (
            info.into(),
            text("Removed").size(10).color(theme::DISABLED).into(),
        )
    } else {
        (
            clickable_row(info, &entry.song.id),
            link_button(
                "Remove",
                10,
                Some(Msg::Favorites(FavoritesMsg::Delete(entry.id))),
            ),
        )
    };

    row![
        container(thumb).width(62).padding([2, 4]),
        info,
        Space::new().width(8),
        container(action).width(70).center_x(70),
    ]
    .width(Fill)
    .align_y(iced::Alignment::Center)
    .padding([3, 4])
    .into()
}
