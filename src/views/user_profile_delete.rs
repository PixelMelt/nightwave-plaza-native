use crate::state::{DeleteMsg, Msg, Plaza};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, d3_sunken};
use iced::widget::{checkbox, column, container, row, text, text_input, Space};
use iced::{Element, Fill};

const WARNINGS: &[&str] = &[
    "\u{2014} Immediate deletion.",
    "\u{2014} All your data will be completely deleted.",
    "\u{2014} Recovery is not possible.",
    "\u{2014} You can register again with the same username and email (if available).",
];

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let mut memo = column![
        text("This action will completely delete your Nightwave Plaza account.")
            .size(11)
            .font(bold),
        Space::with_height(4),
    ];
    for w in WARNINGS {
        memo = memo.push(text(*w).size(11));
    }

    let memo_panel = d3_sunken(
        container(memo).style(theme::sunken_inner).width(Fill).padding(8),
    );

    let confirm = checkbox("I understand, delete my account.", state.delete.confirm)
        .on_toggle(|b| Msg::DeleteAccount(DeleteMsg::Confirm(b)))
        .size(13)
        .text_size(11);

    let password_input = text_input("", &state.delete.current_password)
        .on_input(|s| Msg::DeleteAccount(DeleteMsg::Password(s)))
        .on_submit(Msg::DeleteAccount(DeleteMsg::Submit))
        .secure(true)
        .size(11)
        .padding([3, 4])
        .style(theme::page_input);

    let password_panel = d3_sunken(
        container(column![text("Current Password:").size(11), password_input].spacing(2))
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    );

    let error_row: Element<Msg> = if let Some(ref err) = state.delete.error {
        text(err).size(11).color(iced::Color::from_rgb(0.8, 0.0, 0.0)).into()
    } else {
        Space::with_height(0).into()
    };

    let delete_label = if state.delete.loading { "Deleting..." } else { "Delete Account" };
    let delete_press = (!state.delete.loading).then_some(Msg::DeleteAccount(DeleteMsg::Submit));
    let delete_btn = bevel_button(text(delete_label).size(11).font(bold).center().width(Fill))
        .maybe_on_press(delete_press)
        .width(Fill);
    let close_btn = bevel_button(text("Close").size(11).center().width(Fill))
        .on_press(Msg::CloseWin(wid))
        .width(Fill);

    let buttons = row![
        container(delete_btn).width(iced::Length::FillPortion(3)),
        Space::with_width(8),
        container(close_btn).width(iced::Length::FillPortion(2)),
    ];

    column![
        memo_panel,
        Space::with_height(8),
        confirm,
        Space::with_height(8),
        password_panel,
        Space::with_height(8),
        error_row,
        Space::with_height(4),
        buttons,
    ]
    .padding(8)
    .into()
}
