use crate::state::{Msg, Plaza, ProfileEditMsg, WinType};
use crate::views::{
    action_close_row, form_error, form_input, group_box, labeled_panel, menu_bar, submit_button,
};
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let menu = row![
        Space::new().width(iced::Fill),
        menu_bar([("Delete Account", Msg::OpenWin(WinType::UserProfileDelete))]),
    ];

    let username_input = form_input(&state.profile_edit.username, |s| {
        Msg::ProfileEdit(ProfileEditMsg::Username(s))
    });
    let email_input = form_input(&state.profile_edit.email, |s| {
        Msg::ProfileEdit(ProfileEditMsg::Email(s))
    });

    let details = group_box(
        "User Details",
        column![
            text("Username:").size(11),
            username_input,
            Space::new().height(6),
            text("Email:").size(11),
            email_input,
        ]
        .spacing(2),
    );

    let password_input = form_input(&state.profile_edit.current_password, |s| {
        Msg::ProfileEdit(ProfileEditMsg::CurrentPassword(s))
    })
    .on_submit(Msg::ProfileEdit(ProfileEditMsg::Submit))
    .secure(true);

    let password_panel = labeled_panel("Current Password:", password_input);

    let error_row = form_error(&state.profile_edit.error);

    let save_btn = submit_button(
        state.profile_edit.loading,
        "Saving...",
        "Save",
        Msg::ProfileEdit(ProfileEditMsg::Submit),
        Fill,
    );

    column![
        menu,
        container(column![
            details,
            Space::new().height(8),
            password_panel,
            Space::new().height(8),
            error_row,
            Space::new().height(4),
            action_close_row(save_btn, wid),
        ])
        .padding(8),
    ]
    .into()
}
