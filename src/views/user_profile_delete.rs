use super::widgets::{
    BOLD, action_close_row, form_input, labeled_panel, submit_button, sunken_frame,
};
use crate::message::{DeleteAccountMsg, Msg};
use crate::state::Plaza;
use crate::theme;
use iced::widget::{Space, checkbox, column, container, text};
use iced::window::Id;
use iced::{Element, Fill};

const WARNINGS: [&str; 4] = [
    "\u{2014} Immediate deletion.",
    "\u{2014} All your data will be completely deleted.",
    "\u{2014} Recovery is not possible.",
    "\u{2014} You can register again with the same username and email (if available).",
];

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let form = &state.delete_account;

    let mut memo = column![
        text("This action will completely delete your Nightwave Plaza account.")
            .size(11)
            .font(BOLD),
        Space::new().height(4),
    ];
    for warning in WARNINGS {
        memo = memo.push(text(warning).size(11));
    }
    let memo_panel = sunken_frame(
        container(memo)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(8),
    );

    let confirm = checkbox(form.confirmed)
        .label("I understand, delete my account.")
        .on_toggle(|b| Msg::DeleteAccount(DeleteAccountMsg::Confirm(b)))
        .size(13)
        .text_size(11);

    let password = form_input(&form.current_password, |s| {
        Msg::DeleteAccount(DeleteAccountMsg::Password(s))
    })
    .on_submit(Msg::DeleteAccount(DeleteAccountMsg::Submit))
    .secure(true);

    let delete_button = submit_button(
        form.loading,
        "Deleting...",
        "Delete Account",
        Msg::DeleteAccount(DeleteAccountMsg::Submit),
        Fill,
    );

    column![
        memo_panel,
        Space::new().height(8),
        confirm,
        Space::new().height(8),
        labeled_panel("Current Password:", password),
        Space::new().height(12),
        action_close_row(delete_button, wid),
    ]
    .padding(8)
    .into()
}
