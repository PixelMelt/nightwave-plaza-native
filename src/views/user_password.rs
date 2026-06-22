use crate::state::{Msg, PasswordMsg, Plaza};
use crate::theme;
use crate::views::{action_close_row, d3_sunken, form_error, form_input, submit_button};
use iced::widget::{column, container, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let current = form_input(&state.password.current_password, |s| {
        Msg::Password(PasswordMsg::Current(s))
    })
    .secure(true);
    let new = form_input(&state.password.password, |s| {
        Msg::Password(PasswordMsg::New(s))
    })
    .secure(true);
    let repeat = form_input(&state.password.password_repeat, |s| {
        Msg::Password(PasswordMsg::Repeat(s))
    })
    .on_submit(Msg::Password(PasswordMsg::Submit))
    .secure(true);

    let form = column![
        text("Current Password:").size(11),
        current,
        Space::new().height(6),
        text("New Password:").size(11),
        new,
        Space::new().height(6),
        text("Repeat Password:").size(11),
        repeat,
    ]
    .spacing(2);

    let panel = d3_sunken(container(form).style(theme::panel).width(Fill).padding(8));

    let error_row = form_error(&state.password.error);

    let change_btn = submit_button(
        state.password.loading,
        "Saving...",
        "Change",
        Msg::Password(PasswordMsg::Submit),
        Fill,
    );

    column![
        panel,
        Space::new().height(8),
        error_row,
        Space::new().height(4),
        action_close_row(change_btn, wid),
    ]
    .padding(8)
    .into()
}
