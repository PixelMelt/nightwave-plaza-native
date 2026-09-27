use super::widgets::{
    action_close_row, form_error, form_input, group_box, labeled_panel, menu_bar, submit_button,
};
use crate::message::{Msg, ProfileEditMsg};
use crate::state::Plaza;
use crate::window::WindowKind;
use iced::widget::{Space, column, container, row, text};
use iced::window::Id;
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let edit = &state.profile_edit;
    let menu = row![
        Space::new().width(Fill),
        menu_bar([(
            "Delete Account",
            Msg::OpenWindow(WindowKind::UserProfileDelete)
        )]),
    ];

    let details = group_box(
        "User Details",
        column![
            text("Username:").size(11),
            form_input(&edit.username, |s| Msg::ProfileEdit(
                ProfileEditMsg::Username(s)
            )),
            Space::new().height(6),
            text("Email:").size(11),
            form_input(&edit.email, |s| Msg::ProfileEdit(ProfileEditMsg::Email(s))),
        ]
        .spacing(2),
    );

    let password = form_input(&edit.current_password, |s| {
        Msg::ProfileEdit(ProfileEditMsg::CurrentPassword(s))
    })
    .on_submit(Msg::ProfileEdit(ProfileEditMsg::Submit))
    .secure(true);

    let save_button = submit_button(
        edit.loading,
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
            labeled_panel("Current Password:", password),
            Space::new().height(8),
            form_error(edit.error.as_deref()),
            Space::new().height(4),
            action_close_row(save_button, wid),
        ])
        .padding(8),
    ]
    .into()
}
