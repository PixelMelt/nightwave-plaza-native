use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, d3_sunken};
use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let instruction = column![
        text("User Information:").size(11).font(bold),
        Space::new().height(4),
        text("Please complete all fields to create your account.").size(11),
    ];

    let username_input = text_input("", &state.register.username)
        .on_input(|s| Msg::Register(crate::state::RegisterMsg::Username(s)))
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let password_input = text_input("", &state.register.password)
        .on_input(|s| Msg::Register(crate::state::RegisterMsg::Password(s)))
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    let repeat_input = text_input("", &state.register.password_repeat)
        .on_input(|s| Msg::Register(crate::state::RegisterMsg::PasswordRepeat(s)))
        .size(11)
        .padding([3, 4])
        .secure(true)
        .style(theme::page_input);

    let email_input = text_input("", &state.register.email)
        .on_input(|s| Msg::Register(crate::state::RegisterMsg::Email(s)))
        .on_submit(Msg::Register(crate::state::RegisterMsg::Submit))
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let form = column![
        form_row("Username:", username_input),
        Space::new().height(4),
        form_row("Password:", password_input),
        Space::new().height(4),
        form_row("Repeat Password:", repeat_input),
        Space::new().height(4),
        form_row("Email:", email_input),
    ];

    let error_row: Element<Msg> = if let Some(ref err) = state.register.error {
        text(err)
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
            .into()
    } else {
        Space::new().height(0).into()
    };

    let register_label = if state.register.loading {
        "Loading..."
    } else {
        "Register"
    };
    let register_press =
        (!state.register.loading).then_some(Msg::Register(crate::state::RegisterMsg::Submit));
    let register_btn = bevel_button(text(register_label).size(11).font(bold).center().width(90))
        .maybe_on_press(register_press)
        .width(90);

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

fn form_row<'a>(label_text: &'a str, input: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    row![
        container(text(label_text).size(11)).width(120),
        input.into(),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}
