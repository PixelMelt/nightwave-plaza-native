use super::widgets::{action_close_row, form_error, form_input, submit_button, sunken_panel};
use crate::message::{Msg, PasswordMsg};
use crate::state::Plaza;
use iced::widget::{Space, column, text};
use iced::window::Id;
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let form = &state.password;
    let fields = column![
        text("Current Password:").size(11),
        form_input(&form.current_password, |s| Msg::Password(
            PasswordMsg::Current(s)
        ))
        .secure(true),
        Space::new().height(6),
        text("New Password:").size(11),
        form_input(&form.password, |s| Msg::Password(PasswordMsg::New(s))).secure(true),
        Space::new().height(6),
        text("Repeat Password:").size(11),
        form_input(&form.password_repeat, |s| Msg::Password(
            PasswordMsg::Repeat(s)
        ))
        .on_submit(Msg::Password(PasswordMsg::Submit))
        .secure(true),
    ]
    .spacing(2);

    let change_button = submit_button(
        form.loading,
        "Saving...",
        "Change",
        Msg::Password(PasswordMsg::Submit),
        Fill,
    );

    column![
        sunken_panel(fields),
        Space::new().height(8),
        form_error(form.error.as_deref()),
        Space::new().height(4),
        action_close_row(change_button, wid),
    ]
    .padding(8)
    .into()
}
