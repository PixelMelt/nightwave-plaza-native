use super::widgets::{Png, button, form_field_row, form_input, link_button, submit_button};
use crate::message::{LoginMsg, Msg};
use crate::state::Plaza;
use crate::window::WindowKind;
use iced::widget::{Space, checkbox, column, container, row, text};
use iced::window::Id;
use iced::{Element, Fill};

static KEY: Png = Png::new(include_bytes!("../assets/img/key.png"));

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let login = &state.login;
    let key = container(KEY.image().width(45).height(48)).padding([2, 0]);

    let username = form_input(&login.username, |s| Msg::Login(LoginMsg::Username(s)));
    let password = form_input(&login.password, |s| Msg::Login(LoginMsg::Password(s)))
        .on_submit(Msg::Login(LoginMsg::Submit))
        .secure(true);
    let reset = link_button("Reset", 11, Some(Msg::OpenUrl("https://plaza.one".into())));
    let remember = checkbox(login.remember)
        .label("Remember Me")
        .on_toggle(|b| Msg::Login(LoginMsg::Remember(b)))
        .size(13)
        .text_size(11);

    let center = column![
        text("Enter your username and password to sign in to Nightwave Plaza.").size(11),
        Space::new().height(10),
        form_field_row("Username:", 72, username),
        Space::new().height(5),
        form_field_row(
            "Password:",
            72,
            row![password, Space::new().width(8), reset].align_y(iced::Alignment::Center),
        ),
        Space::new().height(5),
        row![Space::new().width(72), remember],
    ]
    .width(Fill);

    let buttons = column![
        submit_button(
            login.loading,
            "Loading...",
            "Log In",
            Msg::Login(LoginMsg::Submit),
            76,
        ),
        Space::new().height(6),
        button("Register", 76).on_press(Msg::OpenWindow(WindowKind::UserRegister)),
        Space::new().height(6),
        button("Cancel", 76).on_press(Msg::CloseWindow(wid)),
    ];

    row![
        key,
        Space::new().width(12),
        center,
        Space::new().width(12),
        buttons,
    ]
    .padding(8)
    .into()
}
