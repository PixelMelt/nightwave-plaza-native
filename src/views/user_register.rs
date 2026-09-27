use super::widgets::{BOLD, button, form_field_row, form_input, submit_button, sunken_frame};
use crate::message::{Msg, RegisterMsg};
use crate::state::Plaza;
use crate::theme;
use iced::widget::{Space, column, container, row, text};
use iced::window::Id;
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let form = &state.register;
    let field =
        |value, msg: fn(String) -> RegisterMsg| form_input(value, move |s| Msg::Register(msg(s)));

    let fields = column![
        form_field_row(
            "Username:",
            120,
            field(&form.username, RegisterMsg::Username)
        ),
        Space::new().height(4),
        form_field_row(
            "Password:",
            120,
            field(&form.password, RegisterMsg::Password).secure(true)
        ),
        Space::new().height(4),
        form_field_row(
            "Repeat Password:",
            120,
            field(&form.password_repeat, RegisterMsg::PasswordRepeat).secure(true)
        ),
        Space::new().height(4),
        form_field_row(
            "Email:",
            120,
            field(&form.email, RegisterMsg::Email).on_submit(Msg::Register(RegisterMsg::Submit))
        ),
    ];

    let register_button = submit_button(
        form.loading,
        "Loading...",
        "Register",
        Msg::Register(RegisterMsg::Submit),
        90,
    );
    let cancel_button = button("Cancel", 90).on_press(Msg::CloseWindow(wid));
    let bottom = row![register_button, Space::new().width(Fill), cancel_button].padding([4, 0]);

    let content = column![
        text("User Information:").size(11).font(BOLD),
        Space::new().height(4),
        text("Please complete all fields to create your account.").size(11),
        Space::new().height(8),
        fields,
        Space::new().height(10),
        bottom,
    ]
    .padding(8);

    let panel = sunken_frame(
        container(content)
            .style(theme::panel)
            .width(Fill)
            .padding(4),
    );
    column![panel].padding(4).into()
}
