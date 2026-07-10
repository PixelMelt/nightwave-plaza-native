use crate::state::{Msg, Plaza, WinType};
use crate::theme;
use crate::views::bevel_button;
use crate::views::{bold_font, d3_sunken, format_date, group_box, menu_bar, static_image};
use iced::widget::{column, container, image, row, text, Space};
use iced::{Element, Fill};

const USER_CARD_IMG: &[u8] = include_bytes!("../assets/img/user_card.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let menu = menu_bar([
        ("Edit Profile", Msg::OpenWin(WinType::UserProfileEdit)),
        ("Change Password", Msg::OpenWin(WinType::UserPassword)),
        ("Log Out", Msg::Logout),
    ]);

    let card_panel = render_user_card(state);

    let info_row = row![
        container(render_stats_box(state)).width(Fill),
        Space::new().width(8),
        container(render_account_box(state)).width(Fill),
    ];

    let bottom = render_bottom_row(wid);

    column![
        menu,
        container(column![
            card_panel,
            Space::new().height(8),
            info_row,
            Space::new().height(12),
            bottom,
        ])
        .padding(8),
    ]
    .spacing(0)
    .into()
}

fn render_user_card(state: &Plaza) -> Element<'_, Msg> {
    let bold = bold_font();

    let (username, email) = if let Some(ref u) = state.user {
        (u.username.as_str(), u.email.as_str())
    } else {
        ("...", "...")
    };

    let user_card_img = image(static_image(USER_CARD_IMG)).width(32).height(32);

    let user_info = column![
        text(username).size(14).font(bold),
        Space::new().height(4),
        text(email).size(11).color(theme::DISABLED),
    ];

    let user_card = row![
        container(user_card_img).center_y(Fill),
        Space::new().width(8),
        user_info,
    ]
    .align_y(iced::Alignment::Center);

    d3_sunken(
        container(user_card)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    )
    .into()
}

fn render_stats_box(state: &Plaza) -> Element<'_, Msg> {
    let bold = bold_font();

    let (likes_str, favs_str) = if let Some(ref stats) = state.user_stats {
        (
            format!("{}", stats.reactions),
            format!("{}", stats.favorites),
        )
    } else if state.stats_loading {
        ("...".into(), "...".into())
    } else {
        ("0".into(), "0".into())
    };

    let body = column![
        row![
            text("Likes:").size(11).font(bold).width(75),
            text(likes_str).size(11)
        ],
        row![
            text("Favorites:").size(11).font(bold).width(75),
            text(favs_str).size(11)
        ],
    ]
    .spacing(2);

    group_box("Statistics", body)
}

fn render_account_box(state: &Plaza) -> Element<'_, Msg> {
    let bold = bold_font();

    let created_at = state.user.as_ref().map_or(0, |u| u.created_at);
    let registered = if created_at > 0 {
        format_date(created_at)
    } else {
        "...".into()
    };

    let body = column![
        text("Registered:").size(11).font(bold),
        text(registered).size(11),
    ]
    .spacing(2);

    group_box("Account", body)
}

fn render_bottom_row(wid: iced::window::Id) -> Element<'static, Msg> {
    let favorites_btn = bevel_button(text("My Favorites").size(11).center())
        .on_press(Msg::OpenWin(WinType::UserFavorites))
        .padding([4, 12]);

    let close_btn = bevel_button(text("Close").size(11).center().width(Fill))
        .on_press(Msg::CloseWin(wid))
        .width(Fill);

    row![
        favorites_btn,
        Space::new().width(iced::Fill),
        container(close_btn).width(88),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}
