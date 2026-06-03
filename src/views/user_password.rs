use crate::state::{Msg, PasswordMsg, Plaza};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, d3_sunken};
use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let field = |value: &str, on_input: fn(String) -> Msg| {
        text_input("", value)
            .on_input(on_input)
            .secure(true)
            .size(11)
            .padding([3, 4])
            .style(theme::page_input)
    };

    let current = field(&state.password.current_password, |s| {
        Msg::Password(PasswordMsg::Current(s))
    });
    let new = field(&state.password.password, |s| {
        Msg::Password(PasswordMsg::New(s))
    });
    let repeat = text_input("", &state.password.password_repeat)
        .on_input(|s| Msg::Password(PasswordMsg::Repeat(s)))
        .on_submit(Msg::Password(PasswordMsg::Submit))
        .secure(true)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let form = column![
        text("Current Password:").size(11),
        current,
        Space::with_height(6),
        text("New Password:").size(11),
        new,
        Space::with_height(6),
        text("Repeat Password:").size(11),
        repeat,
    ]
    .spacing(2);

    let panel = d3_sunken(container(form).style(theme::panel).width(Fill).padding(8));

    let error_row: Element<Msg> = if let Some(ref err) = state.password.error {
        text(err)
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
            .into()
    } else {
        Space::with_height(0).into()
    };

    let change_label = if state.password.loading {
        "Saving..."
    } else {
        "Change"
    };
    let change_press = (!state.password.loading).then_some(Msg::Password(PasswordMsg::Submit));
    let change_btn = bevel_button(text(change_label).size(11).font(bold).center().width(Fill))
        .maybe_on_press(change_press)
        .width(Fill);
    let close_btn = bevel_button(text("Close").size(11).center().width(Fill))
        .on_press(Msg::CloseWin(wid))
        .width(Fill);

    let buttons = row![
        container(change_btn).width(iced::Length::FillPortion(3)),
        Space::with_width(8),
        container(close_btn).width(iced::Length::FillPortion(2)),
    ];

    column![
        panel,
        Space::with_height(8),
        error_row,
        Space::with_height(4),
        buttons,
    ]
    .padding(8)
    .into()
}
