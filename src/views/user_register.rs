use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken};
use iced::widget::{button, column, container, horizontal_space, row, text, text_input, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    let instruction = column![
        text("User Information:").size(11).font(bold),
        Space::with_height(4),
        text("Please complete all fields to create your account.").size(11),
    ];

    let username_input = text_input("", &state.register_username)
        .on_input(Msg::RegisterUsername)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let password_input = text_input("", &state.register_password)
        .on_input(Msg::RegisterPassword)
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    let repeat_input = text_input("", &state.register_password_repeat)
        .on_input(Msg::RegisterPasswordRepeat)
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    let email_input = text_input("", &state.register_email)
        .on_input(Msg::RegisterEmail)
        .on_submit(Msg::RegisterSubmit)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let form = column![
        row![
            container(text("Username:").size(11)).width(120),
            username_input,
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(4),
        row![
            container(text("Password:").size(11)).width(120),
            password_input,
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(4),
        row![
            container(text("Repeat Password:").size(11)).width(120),
            repeat_input,
        ]
        .align_y(iced::Alignment::Center),
        Space::with_height(4),
        row![container(text("Email:").size(11)).width(120), email_input,]
            .align_y(iced::Alignment::Center),
    ];

    let error_row: Element<Msg> = if let Some(ref err) = state.register_error {
        text(err)
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
            .into()
    } else {
        Space::with_height(0).into()
    };

    let register_label = if state.register_loading {
        "Loading..."
    } else {
        "Register"
    };
    let register_btn = button(text(register_label).size(11).font(bold).center().width(90))
        .style(theme::raised)
        .width(90);
    let register_btn = if !state.register_loading {
        register_btn.on_press(Msg::RegisterSubmit)
    } else {
        register_btn
    };
    let register_btn = d3_raised(register_btn);

    let cancel_btn = d3_raised(
        button(text("Cancel").size(11).center().width(90))
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised)
            .width(90),
    );

    let bottom = row![register_btn, horizontal_space(), cancel_btn].padding([4, 0]);

    let content = column![
        instruction,
        Space::with_height(8),
        form,
        Space::with_height(6),
        error_row,
        Space::with_height(4),
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
