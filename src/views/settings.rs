use crate::state::{DiscordMsg, LastfmMsg, Msg, Plaza, WinType};
use crate::views::bevel::bevel_button;
use crate::views::widgets::{bold_font, close_btn, group_box, LINK_COLOR, MUTED};
use crate::{discord, lastfm};
use iced::widget::{button, checkbox, column, row, text, Space};
use iced::{Element, Fill, Theme};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let lastfm = group_box("Last.fm Scrobbling", lastfm_body(state));
    let discord = group_box("Discord Rich Presence", discord_body(state));

    let timer_btn = bevel_button(text("Sleep Timer...").size(11).center())
        .on_press(Msg::OpenWin(WinType::PlayerTimer))
        .padding([4, 12]);

    let bottom =
        row![timer_btn, Space::with_width(Fill), close_btn(wid)].align_y(iced::Alignment::Center);

    column![lastfm, discord, Space::with_height(Fill), bottom]
        .spacing(8)
        .padding(8)
        .width(Fill)
        .height(Fill)
        .into()
}

fn discord_body(state: &Plaza) -> Element<'_, Msg> {
    if !discord::is_configured() {
        return text(
            "Discord Rich Presence is not configured in this build. \
             An application client ID must be compiled in to enable it.",
        )
        .size(11)
        .color(MUTED)
        .into();
    }

    column![
        checkbox(
            "Show \"Listening to\" status while playing",
            state.discord.enabled
        )
        .on_toggle(|b| Msg::Discord(DiscordMsg::ToggleEnabled(b)))
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
    if !lastfm::is_configured() {
        return text(
            "Last.fm support is not configured in this build. \
             An API key must be compiled in to enable scrobbling.",
        )
        .size(11)
        .color(MUTED)
        .into();
    }

    let mut col = column![].spacing(6).width(Fill);

    let connected_as = state
        .lastfm
        .session_key
        .as_ref()
        .and(state.lastfm.username.as_deref());

    if let Some(username) = connected_as {
        col = col.push(
            row![
                text("Connected as ").size(11),
                text(username).size(11).font(bold_font()),
            ]
            .align_y(iced::Alignment::Center),
        );

        col = col.push(
            checkbox("Scrobble tracks while playing", state.lastfm.enabled)
                .on_toggle(|b| Msg::Lastfm(LastfmMsg::ToggleEnabled(b)))
                .size(13)
                .text_size(11),
        );

        col = col.push(
            bevel_button(text("Disconnect").size(11).center())
                .on_press(Msg::Lastfm(LastfmMsg::Disconnect))
                .padding([4, 12]),
        );
    } else if state.lastfm_token.is_some() {
        col = col.push(
            text("Authorize Nightwave Plaza in the browser window that opened, then click Finish.")
                .size(11),
        );

        let finish_label = if state.lastfm_busy {
            "Finishing..."
        } else {
            "Finish"
        };
        let finish_press = (!state.lastfm_busy).then_some(Msg::Lastfm(LastfmMsg::Finish));
        col = col.push(
            row![
                bevel_button(text(finish_label).size(11).center().width(80))
                    .maybe_on_press(finish_press)
                    .width(80),
                Space::with_width(8),
                reauth_link("Open page again", state),
            ]
            .align_y(iced::Alignment::Center),
        );
    } else {
        col = col.push(
            text("Connect your Last.fm account to scrobble the tracks you listen to.").size(11),
        );

        let connect_label = if state.lastfm_busy {
            "Connecting..."
        } else {
            "Connect..."
        };
        let connect_press = (!state.lastfm_busy).then_some(Msg::Lastfm(LastfmMsg::Connect));
        col = col.push(
            bevel_button(text(connect_label).size(11).center().width(96))
                .maybe_on_press(connect_press)
                .width(96),
        );
    }

    if let Some(ref status) = state.lastfm_status {
        col = col.push(text(status.as_str()).size(11).color(MUTED));
    }

    col.into()
}

fn reauth_link<'a>(label: &'a str, state: &Plaza) -> Element<'a, Msg> {
    let press = (!state.lastfm_busy).then_some(Msg::Lastfm(LastfmMsg::Connect));
    button(text(label).size(11).color(LINK_COLOR))
        .on_press_maybe(press)
        .style(|_: &Theme, _| button::Style {
            background: None,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            text_color: LINK_COLOR,
        })
        .padding(0)
        .into()
}
