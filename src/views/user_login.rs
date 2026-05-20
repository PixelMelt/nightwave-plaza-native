use crate::state::{Msg, Plaza, WinType};
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken};
use iced::widget::{button, checkbox, column, container, row, text, text_input, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    let instruction =
        text("Enter your username and password to sign in to Nightwave Plaza.").size(12);

    // Username field
    let username_label = text("Username:").size(11);
    let username_input = text_input("", &state.login_username)
        .on_input(Msg::LoginUsername)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    // Password field
    let password_label = text("Password:").size(11);
    let password_input = text_input("", &state.login_password)
        .on_input(Msg::LoginPassword)
        .on_submit(Msg::LoginSubmit)
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    // Remember me checkbox
    let remember = checkbox("Remember me", state.login_remember)
        .on_toggle(Msg::LoginRemember)
        .size(13)
        .text_size(11);

    // Form fields
    let form = column![
        row![username_label, Space::with_width(8), username_input].align_y(iced::Alignment::Center),
        Space::with_height(6),
        row![password_label, Space::with_width(10), password_input]
            .align_y(iced::Alignment::Center),
        Space::with_height(4),
        remember,
    ];

    // Error display
    let error_row: Element<Msg> = if let Some(ref err) = state.login_error {
        text(err)
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
            .into()
    } else {
        Space::with_height(0).into()
    };

    // Buttons
    let login_label = if state.login_loading {
        "Loading..."
    } else {
        "Log In"
    };
    let login_btn = button(text(login_label).size(11).font(bold).center().width(90))
        .style(theme::raised)
        .width(90);
    let login_btn = if !state.login_loading {
        login_btn.on_press(Msg::LoginSubmit)
    } else {
        login_btn
    };
    let login_btn = d3_raised(login_btn);

    let register_btn = d3_raised(
        button(text("Register").size(11).center().width(90))
            .on_press(Msg::OpenWin(WinType::UserRegister))
            .style(theme::raised)
            .width(90),
    );

    let cancel_btn = d3_raised(
        button(text("Cancel").size(11).center().width(90))
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised)
            .width(90),
    );

    let buttons = column![
        login_btn,
        Space::with_height(4),
        register_btn,
        Space::with_height(4),
        cancel_btn
    ];

    let content = row![
        column![
            instruction,
            Space::with_height(12),
            form,
            Space::with_height(6),
            error_row
        ]
        .width(Fill),
        Space::with_width(12),
        buttons,
    ]
    .padding(8);

    let panel = d3_sunken(
        container(content)
            .style(theme::panel)
            .width(Fill)
            .padding(6),
    );

    column![panel].padding(4).into()
}
