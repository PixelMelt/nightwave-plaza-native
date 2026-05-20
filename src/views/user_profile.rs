use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken, format_date, menu_btn_underline, status_bar};
use iced::widget::{button, column, container, horizontal_space, image, row, text, Space};
use iced::{Element, Fill};

const USER_CARD_IMG: &[u8] = include_bytes!("../../assets/icons/user_card.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    let menu = row![
        menu_btn_underline("Edit Profile", Msg::Refresh),
        menu_btn_underline("Change Password", Msg::Refresh),
        menu_btn_underline("Log Out", Msg::Logout),
    ]
    .padding([1, 1]);

    let (username, email, created_at) = if let Some(ref u) = state.user {
        (u.username.as_str(), u.email.as_str(), u.created_at)
    } else {
        ("...", "...", 0)
    };

    let user_card_img = image(image::Handle::from_bytes(USER_CARD_IMG))
        .width(32)
        .height(32);

    let user_info = column![
        text(username).size(14).font(bold),
        Space::with_height(4),
        text(email)
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
    ];

    let user_card = row![
        container(user_card_img).center_y(Fill),
        Space::with_width(8),
        user_info,
    ]
    .align_y(iced::Alignment::Center);

    let card_panel = d3_sunken(
        container(user_card)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    );

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

    let stats_label = container(text(" Statistics ").size(11).font(bold))
        .style(theme::group_label)
        .padding([0, 4]);

    let stats_content = column![
        row![
            text("Likes:").size(11).font(bold).width(75),
            text(likes_str).size(11),
        ],
        row![
            text("Favorites:").size(11).font(bold).width(75),
            text(favs_str).size(11),
        ],
    ]
    .spacing(2);

    let stats_box = column![
        stats_label,
        d3_sunken(
            container(stats_content)
                .style(theme::panel)
                .width(Fill)
                .padding(8),
        ),
    ]
    .spacing(0);

    let registered = if created_at > 0 {
        format_date(created_at)
    } else {
        "...".into()
    };

    let account_label = container(text(" Account ").size(11).font(bold))
        .style(theme::group_label)
        .padding([0, 4]);

    let account_content = column![
        text("Registered:").size(11).font(bold),
        text(registered).size(11),
    ]
    .spacing(2);

    let account_box = column![
        account_label,
        d3_sunken(
            container(account_content)
                .style(theme::panel)
                .width(Fill)
                .padding(8),
        ),
    ]
    .spacing(0);

    let info_row = row![
        container(stats_box).width(Fill),
        Space::with_width(8),
        container(account_box).width(Fill),
    ];

    let favorites_btn = d3_raised(
        button(text("Favorites").size(11).center())
            .on_press(Msg::Refresh)
            .style(theme::raised)
            .padding([4, 12]),
    );

    let close_btn = d3_raised(
        button(text("Close").size(11).center().width(Fill))
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised),
    );

    let bottom = row![
        favorites_btn,
        horizontal_space(),
        container(close_btn).width(iced::Length::FillPortion(4)),
    ];

    let status = status_bar(vec![text(format!("Logged in as: {}", username))
        .size(10)
        .width(Fill)
        .into()]);

    column![
        menu,
        container(column![
            card_panel,
            Space::with_height(8),
            info_row,
            Space::with_height(16),
            bottom,
        ])
        .padding(8),
        status,
    ]
    .spacing(0)
    .into()
}
