use super::{open_url, playback, task};
use crate::config;
use crate::lastfm;
use crate::state::{self, LastfmMsg, Msg, Plaza, TimerMsg};
use iced::Task;
use std::time::{Duration, Instant};

pub fn lastfm(state: &mut Plaza, msg: LastfmMsg) -> Task<Msg> {
    match msg {
        LastfmMsg::ToggleEnabled(enabled) => {
            state.config.lastfm.enabled = enabled;
            config::save(&state.config);
            let playing = state.player.is_playing();
            playback::sync_playback(state, enabled && playing)
        }
        LastfmMsg::Connect => {
            state.lastfm.busy = true;
            state.lastfm.status = None;
            task(lastfm::get_token(), |r| Msg::Lastfm(LastfmMsg::Token(r)))
        }
        LastfmMsg::Token(Ok(token)) => {
            state.lastfm.busy = false;
            open_url(&lastfm::auth_url(&token));
            state.lastfm.token = Some(token);
            Task::none()
        }
        LastfmMsg::Finish => {
            let Some(token) = state.lastfm.token.clone() else {
                return Task::none();
            };
            state.lastfm.busy = true;
            state.lastfm.status = None;
            task(lastfm::get_session(token), |r| {
                Msg::Lastfm(LastfmMsg::Session(r))
            })
        }
        LastfmMsg::Session(Ok((username, key))) => {
            state.lastfm = state::LastfmState {
                status: Some("Connected. Scrobbling is on.".into()),
                ..Default::default()
            };
            state.config.lastfm.username = Some(username);
            state.config.lastfm.session_key = Some(key);
            state.config.lastfm.enabled = true;
            config::save(&state.config);
            let playing = state.player.is_playing();
            playback::sync_playback(state, playing)
        }
        LastfmMsg::Disconnect => {
            state.config.lastfm = Default::default();
            state.lastfm = Default::default();
            config::save(&state.config);
            Task::none()
        }
        LastfmMsg::Token(Err(e)) | LastfmMsg::Session(Err(e)) => {
            state.lastfm.busy = false;
            state.lastfm.status = Some(e);
            Task::none()
        }
    }
}

pub fn discord(state: &mut Plaza, enabled: bool) -> Task<Msg> {
    state.config.discord.enabled = enabled;
    config::save(&state.config);
    playback::discord_update(state);
    Task::none()
}

pub fn timer(state: &mut Plaza, msg: TimerMsg) -> Task<Msg> {
    let timer = &mut state.timer;
    match msg {
        TimerMsg::Input(s) => state::digits_input(&mut timer.minutes_input, s),
        TimerMsg::Add(delta) => {
            let current = timer.minutes_input.parse::<i32>().unwrap_or(0);
            timer.minutes_input = (current + delta).max(1).to_string();
        }
        TimerMsg::Start => {
            if let Ok(mins @ 1..) = timer.minutes_input.parse::<u64>() {
                timer.until = Some(Instant::now() + Duration::from_secs(mins * 60));
            }
        }
        TimerMsg::Stop => timer.until = None,
    }
    Task::none()
}
