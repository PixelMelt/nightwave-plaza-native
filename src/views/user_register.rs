use crate::state::{Msg, Plaza, RegisterMsg};
use crate::theme;
use crate::views::{
    button, d3_sunken, form_error, form_field_row, form_input, submit_button, BOLD,
};
use iced::widget::{column, container, row, text, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let reg = &state.register;
    let field =
        |value, msg: fn(String) -> RegisterMsg| form_input(value, move |s| Msg::Register(msg(s)));

    let form = column![
        form_field_row(
            "Username:",
            120,
            field(&reg.username, RegisterMsg::Username)
        ),
        Space::new().height(4),
        form_field_row(
            "Password:",
            120,
            field(&reg.password, RegisterMsg::Password).secure(true)
        ),
        Space::new().height(4),
        form_field_row(
            "Repeat Password:",
            120,
            field(&reg.password_repeat, RegisterMsg::PasswordRepeat).secure(true)
        ),
        Space::new().height(4),
        form_field_row(
            "Email:",
            120,
            field(&reg.email, RegisterMsg::Email).on_submit(Msg::Register(RegisterMsg::Submit))
        ),
    ];

    let register_btn = submit_button(
        reg.loading,
        "Loading...",
        "Register",
        Msg::Register(RegisterMsg::Submit),
        90,
    );
    let cancel_btn = button("Cancel", 90).on_press(Msg::CloseWin(wid));
    let bottom = row![register_btn, Space::new().width(Fill), cancel_btn].padding([4, 0]);

    let content = column![
        text("User Information:").size(11).font(BOLD),
        Space::new().height(4),
        text("Please complete all fields to create your account.").size(11),
        Space::new().height(8),
        form,
        Space::new().height(6),
        form_error(&reg.error),
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
