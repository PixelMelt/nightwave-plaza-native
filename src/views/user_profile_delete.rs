use crate::state::{DeleteMsg, Msg, Plaza};
use crate::theme;
use crate::views::{
    action_close_row, d3_sunken, form_error, form_input, labeled_panel, submit_button, BOLD,
};
use iced::widget::{checkbox, column, container, text, Space};
use iced::{Element, Fill};

const WARNINGS: [&str; 4] = [
    "\u{2014} Immediate deletion.",
    "\u{2014} All your data will be completely deleted.",
    "\u{2014} Recovery is not possible.",
    "\u{2014} You can register again with the same username and email (if available).",
];

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let del = &state.delete;

    let mut memo = column![
        text("This action will completely delete your Nightwave Plaza account.")
            .size(11)
            .font(BOLD),
        Space::new().height(4),
    ];
    for warning in WARNINGS {
        memo = memo.push(text(warning).size(11));
    }
    let memo_panel = d3_sunken(
        container(memo)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(8),
    );

    let confirm = checkbox(del.confirm)
        .label("I understand, delete my account.")
        .on_toggle(|b| Msg::DeleteAccount(DeleteMsg::Confirm(b)))
        .size(13)
        .text_size(11);

    let password = form_input(&del.current_password, |s| {
        Msg::DeleteAccount(DeleteMsg::Password(s))
    })
    .on_submit(Msg::DeleteAccount(DeleteMsg::Submit))
    .secure(true);

    let delete_btn = submit_button(
        del.loading,
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
        labeled_panel("Current Password:", password),
        Space::new().height(8),
        form_error(&del.error),
        Space::new().height(4),
        action_close_row(delete_btn, wid),
    ]
    .padding(8)
    .into()
}
