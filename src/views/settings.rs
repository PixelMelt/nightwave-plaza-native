use super::widgets::{BOLD, button, close_button, group_box, link_button};
use crate::message::{LastfmMsg, Msg};
use crate::state::Plaza;
use crate::theme::MUTED;
use crate::window::WindowKind;
use iced::widget::{Space, checkbox, column, row, text};
use iced::window::Id;
use iced::{Element, Fill, Length};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let timer_button = button("Sleep Timer...", Length::Shrink)
        .on_press(Msg::OpenWindow(WindowKind::PlayerTimer))
        .padding([4, 12]);
    let bottom = row![timer_button, Space::new().width(Fill), close_button(wid)]
        .align_y(iced::Alignment::Center);

    column![
        group_box("Last.fm Scrobbling", lastfm_body(state)),
        group_box("Discord Rich Presence", discord_body(state)),
        Space::new().height(Fill),
        bottom,
    ]
    .spacing(8)
    .padding(8)
    .width(Fill)
    .height(Fill)
    .into()
}

fn discord_body(state: &Plaza) -> Element<'_, Msg> {
    column![
        checkbox(state.config.discord.enabled)
            .label("Show \"Listening to\" status while playing")
            .on_toggle(Msg::DiscordEnabled)
            .size(13)
            .text_size(11),
        text("Requires the Discord desktop app to be running.")
            .size(11)
            .color(MUTED),
    ]
    .spacing(6)
    .width(Fill)
    .into()
}

fn lastfm_body(state: &Plaza) -> Element<'_, Msg> {
    let lastfm = &state.config.lastfm;
    let busy = state.lastfm.busy;
    let connect = (!busy).then_some(Msg::Lastfm(LastfmMsg::Connect));
    let mut col = column![].spacing(6).width(Fill);

    if let Some(username) = lastfm.connected_username() {
        col = col
            .push(
                row![
                    text("Connected as ").size(11),
                    text(username).size(11).font(BOLD),
                ]
                .align_y(iced::Alignment::Center),
            )
            .push(
                checkbox(lastfm.enabled)
                    .label("Scrobble tracks while playing")
                    .on_toggle(|b| Msg::Lastfm(LastfmMsg::SetEnabled(b)))
                    .size(13)
                    .text_size(11),
            )
            .push(
                button("Disconnect", Length::Shrink)
                    .on_press(Msg::Lastfm(LastfmMsg::Disconnect))
                    .padding([4, 12]),
            );
    } else if state.lastfm.token.is_some() {
        let finish = if busy { "Finishing..." } else { "Finish" };
        col = col
            .push(
                text("Authorize Nightwave Plaza in the browser window that opened, then click Finish.")
                    .size(11),
            )
            .push(
                row![
                    button(finish, 80).on_press_maybe((!busy).then_some(Msg::Lastfm(LastfmMsg::Finish))),
                    Space::new().width(8),
                    link_button("Open page again", 11, connect),
                ]
                .align_y(iced::Alignment::Center),
            );
    } else {
        let label = if busy { "Connecting..." } else { "Connect..." };
        col = col
            .push(
                text("Connect your Last.fm account to scrobble the tracks you listen to.").size(11),
            )
            .push(button(label, 96).on_press_maybe(connect));
    }

    if let Some(status) = &state.lastfm.status {
        col = col.push(text(status).size(11).color(MUTED));
    }
    col.into()
}
