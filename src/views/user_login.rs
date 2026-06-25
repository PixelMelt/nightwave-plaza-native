use crate::state::{Msg, Plaza, WinType};
use crate::views::bevel_button;
use crate::views::{
    form_error, form_field_row, form_input, link_button, static_image, submit_button,
};
use iced::widget::{checkbox, column, container, image, row, text, Space};
use iced::{Element, Fill};

const KEY_IMG: &[u8] = include_bytes!("../assets/img/key.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let key = container(image(static_image(KEY_IMG)).width(45).height(48)).padding([2, 0]);

    let instruction =
        text("Enter your username and password to sign in to Nightwave Plaza.").size(11);

    let username_input = form_input(&state.login.username, |s| {
        Msg::Login(crate::state::LoginMsg::Username(s))
    });

    let password_input = form_input(&state.login.password, |s| {
        Msg::Login(crate::state::LoginMsg::Password(s))
    })
    .on_submit(Msg::Login(crate::state::LoginMsg::Submit))
    .secure(true);

    let reset_link = link_button("Reset", 11, Some(Msg::OpenUrl("https://plaza.one".into())));

    let username_row = form_field_row("Username:", 72, username_input);

    let password_row = form_field_row(
        "Password:",
        72,
        row![password_input, Space::new().width(8), reset_link].align_y(iced::Alignment::Center),
    );

    let remember = checkbox(state.login.remember)
        .label("Remember Me")
        .on_toggle(|b| Msg::Login(crate::state::LoginMsg::Remember(b)))
        .size(13)
        .text_size(11);

    let error_row = form_error(&state.login.error);

    let center = column![
        instruction,
        Space::new().height(10),
        username_row,
        Space::new().height(5),
        password_row,
        Space::new().height(5),
        row![Space::new().width(72), remember],
        Space::new().height(4),
        error_row,
    ]
    .width(Fill);

    let login_btn = submit_button(
        state.login.loading,
        "Loading...",
        "Log In",
        Msg::Login(crate::state::LoginMsg::Submit),
        76,
    );
    let register_btn = bevel_button(text("Register").size(11).center().width(76))
        .on_press(Msg::OpenWin(WinType::UserRegister))
        .width(76);
    let cancel_btn = bevel_button(text("Cancel").size(11).center().width(76))
        .on_press(Msg::CloseWin(wid))
        .width(76);

    let buttons = column![
        login_btn,
        Space::new().height(6),
        register_btn,
        Space::new().height(6),
        cancel_btn,
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
