use crate::state::{Msg, PasswordMsg, Plaza};
use crate::views::{action_close_row, form_error, form_input, submit_button, sunken_panel};
use iced::widget::{column, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let pw = &state.password;
    let form = column![
        text("Current Password:").size(11),
        form_input(&pw.current_password, |s| Msg::Password(
            PasswordMsg::Current(s)
        ))
        .secure(true),
        Space::new().height(6),
        text("New Password:").size(11),
        form_input(&pw.password, |s| Msg::Password(PasswordMsg::New(s))).secure(true),
        Space::new().height(6),
        text("Repeat Password:").size(11),
        form_input(&pw.password_repeat, |s| Msg::Password(PasswordMsg::Repeat(
            s
        )))
        .on_submit(Msg::Password(PasswordMsg::Submit))
        .secure(true),
    ]
    .spacing(2);

    let change_btn = submit_button(
        pw.loading,
        "Saving...",
        "Change",
        Msg::Password(PasswordMsg::Submit),
        Fill,
    );

    column![
        sunken_panel(form),
        Space::new().height(8),
        form_error(&pw.error),
        Space::new().height(4),
        action_close_row(change_btn, wid),
    ]
    .padding(8)
    .into()
}
