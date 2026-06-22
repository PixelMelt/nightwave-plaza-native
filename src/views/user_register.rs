use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::bevel_button;
use crate::views::{bold_font, d3_sunken, form_error, form_field_row, form_input, submit_button};
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let instruction = column![
        text("User Information:").size(11).font(bold_font()),
        Space::new().height(4),
        text("Please complete all fields to create your account.").size(11),
    ];

    let username_input = form_input(&state.register.username, |s| {
        Msg::Register(crate::state::RegisterMsg::Username(s))
    });
    let password_input = form_input(&state.register.password, |s| {
        Msg::Register(crate::state::RegisterMsg::Password(s))
    })
    .secure(true);
    let repeat_input = form_input(&state.register.password_repeat, |s| {
        Msg::Register(crate::state::RegisterMsg::PasswordRepeat(s))
    })
    .secure(true);
    let email_input = form_input(&state.register.email, |s| {
        Msg::Register(crate::state::RegisterMsg::Email(s))
    })
    .on_submit(Msg::Register(crate::state::RegisterMsg::Submit));

    let form = column![
        form_field_row("Username:", 120, username_input),
        Space::new().height(4),
        form_field_row("Password:", 120, password_input),
        Space::new().height(4),
        form_field_row("Repeat Password:", 120, repeat_input),
        Space::new().height(4),
        form_field_row("Email:", 120, email_input),
    ];

    let error_row = form_error(&state.register.error);

    let register_btn = submit_button(
        state.register.loading,
        "Loading...",
        "Register",
        Msg::Register(crate::state::RegisterMsg::Submit),
        90,
    );

    let cancel_btn = bevel_button(text("Cancel").size(11).center().width(90))
        .on_press(Msg::CloseWin(wid))
        .width(90);

    let bottom = row![register_btn, Space::new().width(iced::Fill), cancel_btn].padding([4, 0]);

    let content = column![
        instruction,
        Space::new().height(8),
        form,
        Space::new().height(6),
        error_row,
        Space::new().height(4),
        bottom,
    ]
    .padding(8);

    let panel = d3_sunken(
        container(content)
            .style(theme::panel)
            .width(Fill)
            .padding(4),
    );

    column![panel].padding(4).into()
}
