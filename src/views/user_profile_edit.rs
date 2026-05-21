use crate::state::{Msg, Plaza, ProfileEditMsg, WinType};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, d3_sunken, group_box, menu_bar};
use iced::widget::{column, container, horizontal_space, row, text, text_input, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let menu = row![
        horizontal_space(),
        menu_bar([("Delete Account", Msg::OpenWin(WinType::UserProfileDelete))]),
    ];

    let username_input = text_input("", &state.profile_edit.username)
        .on_input(|s| Msg::ProfileEdit(ProfileEditMsg::Username(s)))
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let email_input = text_input("", &state.profile_edit.email)
        .on_input(|s| Msg::ProfileEdit(ProfileEditMsg::Email(s)))
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let details = group_box(
        "User Details",
        column![
            text("Username:").size(11),
            username_input,
            Space::with_height(6),
            text("Email:").size(11),
            email_input,
        ]
        .spacing(2),
    );

    let password_input = text_input("", &state.profile_edit.current_password)
        .on_input(|s| Msg::ProfileEdit(ProfileEditMsg::CurrentPassword(s)))
        .on_submit(Msg::ProfileEdit(ProfileEditMsg::Submit))
        .secure(true)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let password_panel = d3_sunken(
        container(
            column![text("Current Password:").size(11), password_input].spacing(2),
        )
        .style(theme::panel)
        .width(Fill)
        .padding(8),
    );

    let error_row: Element<Msg> = if let Some(ref err) = state.profile_edit.error {
        text(err).size(11).color(iced::Color::from_rgb(0.8, 0.0, 0.0)).into()
    } else {
        Space::with_height(0).into()
    };

    let save_label = if state.profile_edit.loading { "Saving..." } else { "Save" };
    let save_press = (!state.profile_edit.loading).then_some(Msg::ProfileEdit(ProfileEditMsg::Submit));
    let save_btn = bevel_button(text(save_label).size(11).font(bold).center().width(Fill))
        .maybe_on_press(save_press)
        .width(Fill);
    let close_btn = bevel_button(text("Close").size(11).center().width(Fill))
        .on_press(Msg::CloseWin(wid))
        .width(Fill);

    let buttons = row![
        container(save_btn).width(iced::Length::FillPortion(3)),
        Space::with_width(8),
        container(close_btn).width(iced::Length::FillPortion(2)),
    ];

    column![
        menu,
        container(column![
            details,
            Space::with_height(8),
            password_panel,
            Space::with_height(8),
            error_row,
            Space::with_height(4),
            buttons,
        ])
        .padding(8),
    ]
    .into()
}
