use crate::state::{Msg, Plaza, WinType};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, LINK_COLOR};
use iced::widget::{button, checkbox, column, container, image, row, text, text_input, Space};
use iced::{Element, Fill, Theme};

const KEY_IMG: &[u8] = include_bytes!("../assets/img/key.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let key = container(
        image(image::Handle::from_bytes(KEY_IMG))
            .width(45)
            .height(48),
    )
    .padding([2, 0]);

    let instruction =
        text("Enter your username and password to sign in to Nightwave Plaza.").size(11);

    let username_input = text_input("", &state.login.username)
        .on_input(|s| Msg::Login(crate::state::LoginMsg::Username(s)))
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let password_input = text_input("", &state.login.password)
        .on_input(|s| Msg::Login(crate::state::LoginMsg::Password(s)))
        .on_submit(Msg::Login(crate::state::LoginMsg::Submit))
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    let reset_link = button(text("Reset").size(11).color(LINK_COLOR))
        .on_press(Msg::OpenUrl("https://plaza.one".into()))
        .style(|_: &Theme, _| button::Style {
            background: None,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            text_color: LINK_COLOR,
        })
        .padding(0);

    let username_row = row![
        container(text("Username:").size(11)).width(72),
        username_input,
    ]
    .align_y(iced::Alignment::Center);

    let password_row = row![
        container(text("Password:").size(11)).width(72),
        password_input,
        Space::with_width(8),
        reset_link,
    ]
    .align_y(iced::Alignment::Center);

    let remember = checkbox("Remember Me", state.login.remember)
        .on_toggle(|b| Msg::Login(crate::state::LoginMsg::Remember(b)))
        .size(13)
        .text_size(11);

    let error_row: Element<Msg> = if let Some(ref err) = state.login.error {
        text(err)
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
            .into()
    } else {
        Space::with_height(0).into()
    };

    let center = column![
        instruction,
        Space::with_height(10),
        username_row,
        Space::with_height(5),
        password_row,
        Space::with_height(5),
        row![Space::with_width(72), remember],
        Space::with_height(4),
        error_row,
    ]
    .width(Fill);

    let login_label = if state.login.loading {
        "Loading..."
    } else {
        "Log In"
    };
    let login_press = (!state.login.loading).then_some(Msg::Login(crate::state::LoginMsg::Submit));
    let login_btn = bevel_button(text(login_label).size(11).font(bold).center().width(76))
        .maybe_on_press(login_press)
        .width(76);
    let register_btn = bevel_button(text("Register").size(11).center().width(76))
        .on_press(Msg::OpenWin(WinType::UserRegister))
        .width(76);
    let cancel_btn = bevel_button(text("Cancel").size(11).center().width(76))
        .on_press(Msg::CloseWin(wid))
        .width(76);

    let buttons = column![
        login_btn,
        Space::with_height(6),
        register_btn,
        Space::with_height(6),
        cancel_btn,
    ];

    row![
        key,
        Space::with_width(12),
        center,
        Space::with_width(12),
        buttons,
    ]
    .padding(8)
    .into()
}
