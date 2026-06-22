use crate::state::{DeleteMsg, Msg, Plaza};
use crate::theme;
use crate::views::{
    action_close_row, bold_font, d3_sunken, form_error, form_input, labeled_panel, submit_button,
};
use iced::widget::{checkbox, column, container, text, Space};
use iced::{Element, Fill};

const WARNINGS: &[&str] = &[
    "\u{2014} Immediate deletion.",
    "\u{2014} All your data will be completely deleted.",
    "\u{2014} Recovery is not possible.",
    "\u{2014} You can register again with the same username and email (if available).",
];

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let mut memo = column![
        text("This action will completely delete your Nightwave Plaza account.")
            .size(11)
            .font(bold_font()),
        Space::new().height(4),
    ];
    for w in WARNINGS {
        memo = memo.push(text(*w).size(11));
    }

    let memo_panel = d3_sunken(
        container(memo)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(8),
    );

    let confirm = checkbox(state.delete.confirm)
        .label("I understand, delete my account.")
        .on_toggle(|b| Msg::DeleteAccount(DeleteMsg::Confirm(b)))
        .size(13)
        .text_size(11);

    let password_input = form_input(&state.delete.current_password, |s| {
        Msg::DeleteAccount(DeleteMsg::Password(s))
    })
    .on_submit(Msg::DeleteAccount(DeleteMsg::Submit))
    .secure(true);

    let password_panel = labeled_panel("Current Password:", password_input);

    let error_row = form_error(&state.delete.error);

    let delete_btn = submit_button(
        state.delete.loading,
        "Deleting...",
        "Delete Account",
        Msg::DeleteAccount(DeleteMsg::Submit),
        Fill,
    );

    column![
        memo_panel,
        Space::new().height(8),
        confirm,
        Space::new().height(8),
        password_panel,
        Space::new().height(8),
        error_row,
        Space::new().height(4),
        action_close_row(delete_btn, wid),
    ]
    .padding(8)
    .into()
}
