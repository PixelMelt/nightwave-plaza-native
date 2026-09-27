use crate::state::{Msg, Plaza, ProfileEditMsg, WinType};
use crate::views::{
    action_close_row, form_error, form_input, group_box, labeled_panel, menu_bar, submit_button,
};
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let edit = &state.profile_edit;
    let menu = row![
        Space::new().width(Fill),
        menu_bar([("Delete Account", Msg::OpenWin(WinType::UserProfileDelete))]),
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

    let save_btn = submit_button(
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
            form_error(&edit.error),
            Space::new().height(4),
            action_close_row(save_btn, wid),
        ])
        .padding(8),
    ]
    .into()
}
