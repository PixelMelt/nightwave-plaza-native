use super::widgets::{BOLD, Png, button, format_date, group_box, menu_bar, sunken_panel};
use crate::message::{AccountMsg, Msg};
use crate::state::Plaza;
use crate::theme;
use crate::window::WindowKind;
use iced::widget::{Space, column, container, row, text};
use iced::window::Id;
use iced::{Element, Fill, Length};

static USER_CARD: Png = Png::new(include_bytes!("../assets/img/user_card.png"));

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let menu = menu_bar([
        ("Edit Profile", Msg::OpenWindow(WindowKind::UserProfileEdit)),
        ("Change Password", Msg::OpenWindow(WindowKind::UserPassword)),
        ("Log Out", Msg::Account(AccountMsg::Logout)),
    ]);

    let info_row = row![
        container(stats_box(state)).width(Fill),
        Space::new().width(8),
        container(account_box(state)).width(Fill),
    ];

    let favorites_button = button("My Favorites", Length::Shrink)
        .on_press(Msg::OpenWindow(WindowKind::UserFavorites))
        .padding([4, 12]);
    let bottom = row![
        favorites_button,
        Space::new().width(Fill),
        button("Close", 88).on_press(Msg::CloseWindow(wid)),
    ]
    .align_y(iced::Alignment::Center);

    column![
        menu,
        container(column![
            user_card(state),
            Space::new().height(8),
            info_row,
            Space::new().height(12),
            bottom,
        ])
        .padding(8),
    ]
    .into()
}

fn user_card(state: &Plaza) -> Element<'_, Msg> {
    let (username, email) = state
        .user()
        .map_or(("...", "..."), |u| (u.username.as_str(), u.email.as_str()));
    let card = row![
        container(USER_CARD.image().width(32).height(32)).center_y(Fill),
        Space::new().width(8),
        column![
            text(username).size(14).font(BOLD),
            Space::new().height(4),
            text(email).size(11).color(theme::DISABLED),
        ],
    ]
    .align_y(iced::Alignment::Center);
    sunken_panel(card)
}

fn stats_box(state: &Plaza) -> Element<'_, Msg> {
    let (likes, favorites) = match &state.user_stats {
        Some(stats) => (stats.reactions.to_string(), stats.favorites.to_string()),
        None if state.stats_loading => ("...".into(), "...".into()),
        None => ("0".into(), "0".into()),
    };
    let stat = |label, value: String| {
        row![
            text(label).size(11).font(BOLD).width(75),
            text(value).size(11)
        ]
    };
    group_box(
        "Statistics",
        column![stat("Likes:", likes), stat("Favorites:", favorites)].spacing(2),
    )
}

fn account_box(state: &Plaza) -> Element<'_, Msg> {
    let registered = match state.user().map(|u| u.created_at) {
        Some(ts) if ts > 0 => format_date(ts),
        _ => "...".into(),
    };
    group_box(
        "Account",
        column![
            text("Registered:").size(11).font(BOLD),
            text(registered).size(11),
        ]
        .spacing(2),
    )
}
