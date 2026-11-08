use super::playback;
use crate::config;
use crate::lastfm;
use crate::message::{LastfmMsg, Msg, TimerMsg};
use crate::platform;
use crate::state::{self, LastfmState, Plaza};
use iced::Task;
use std::time::{Duration, Instant};

pub fn lastfm(state: &mut Plaza, msg: LastfmMsg) -> Task<Msg> {
    match msg {
        LastfmMsg::SetEnabled(enabled) => {
            state.config.lastfm.enabled = enabled;
            config::save(&state.config);
            playback::sync_integrations(state);
            if enabled {
                playback::announce_now_playing(state)
            } else {
                Task::none()
            }
        }
        LastfmMsg::Connect => {
            state.lastfm.busy = true;
            state.lastfm.status = None;
            Task::perform(lastfm::fetch_token(), |r| Msg::Lastfm(LastfmMsg::Token(r)))
        }
        LastfmMsg::Token(Ok(token)) => {
            state.lastfm.busy = false;
            platform::open_url(&lastfm::auth_url(&token));
            state.lastfm.token = Some(token);
            Task::none()
        }
        LastfmMsg::Finish => {
            let Some(token) = state.lastfm.token.clone() else {
                return Task::none();
            };
            state.lastfm.busy = true;
            state.lastfm.status = None;
            Task::perform(lastfm::fetch_session(token), |r| {
                Msg::Lastfm(LastfmMsg::Session(r))
            })
        }
        LastfmMsg::Session(Ok(session)) => {
            state.lastfm = LastfmState {
                status: Some("Connected. Scrobbling is on.".into()),
                ..LastfmState::default()
            };
            state.config.lastfm.username = Some(session.username);
            state.config.lastfm.session_key = Some(session.key);
            state.config.lastfm.enabled = true;
            config::save(&state.config);
            playback::sync_integrations(state);
            playback::announce_now_playing(state)
        }
        LastfmMsg::Submitted(key, Err(e))
            if e.is_unauthorized() && state.config.lastfm.session_key.as_ref() == Some(&key) =>
        {
            state.config.lastfm = lastfm::LastfmConfig::default();
            state.lastfm = LastfmState {
                status: Some("Last.fm session expired. Please connect again.".into()),
                ..LastfmState::default()
            };
            config::save(&state.config);
            Task::none()
        }
        LastfmMsg::Submitted(_, result) => {
            if let Err(e) = result {
                eprintln!("Last.fm submission failed: {e}");
            }
            Task::none()
        }
        LastfmMsg::Disconnect => {
            state.config.lastfm = lastfm::LastfmConfig::default();
            state.lastfm = LastfmState::default();
            config::save(&state.config);
            Task::none()
        }
        LastfmMsg::Token(Err(e)) | LastfmMsg::Session(Err(e)) => {
            state.lastfm.busy = false;
            state.lastfm.status = Some(e.to_string());
            Task::none()
        }
    }
}

pub fn set_discord_enabled(state: &mut Plaza, enabled: bool) {
    state.config.discord.enabled = enabled;
    config::save(&state.config);
    playback::sync_discord(state);
}

pub fn timer(state: &mut Plaza, msg: TimerMsg) {
    let timer = &mut state.timer;
    match msg {
        TimerMsg::Input(input) => state::accept_digits(&mut timer.minutes_input, input),
        TimerMsg::Add(delta) => {
            let current = timer.minutes_input.parse::<u32>().unwrap_or(0);
            let minutes = current
                .saturating_add_signed(delta)
                .clamp(1, state::MAX_TIMER_MINUTES);
            timer.minutes_input = minutes.to_string();
        }
        TimerMsg::Start => {
            if let Some(minutes) = timer.minutes() {
                timer.minutes_input = minutes.to_string();
                timer.until = Some(Instant::now() + Duration::from_secs(u64::from(minutes) * 60));
            }
        }
        TimerMsg::Stop => timer.until = None,
    }
}
