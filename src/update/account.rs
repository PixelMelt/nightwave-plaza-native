use super::{close_windows, show_error, show_info, song_info};
use crate::api;
use crate::config::{self, Session};
use crate::message::{
    AccountMsg, DeleteAccountMsg, LoginMsg, Msg, PasswordMsg, ProfileEditMsg, ProfileUpdate,
    RegisterMsg,
};
use crate::state::{
    DeleteAccountForm, ExportState, FavoritesState, LoginForm, PasswordForm, Plaza,
    ProfileEditForm, RegisterForm, SongReaction,
};
use crate::window::WindowKind;
use iced::Task;
use std::future::Future;
use std::time::Instant;

const ACCOUNT_WINDOWS: [WindowKind; 6] = [
    WindowKind::UserFavorites,
    WindowKind::UserFavoritesExport,
    WindowKind::UserPassword,
    WindowKind::UserProfile,
    WindowKind::UserProfileDelete,
    WindowKind::UserProfileEdit,
];

#[derive(Clone, Copy)]
enum Persistence {
    Remember,
    Forget,
}

fn store_session(state: &mut Plaza, session: Session, persistence: Persistence) {
    if let Persistence::Remember = persistence {
        state.config.session = Some(session.clone());
        config::save(&state.config);
    }
    state.session = Some(session);
}

fn sign_out(state: &mut Plaza) -> Task<Msg> {
    state.session = None;
    state.config.session = None;
    config::save(&state.config);
    state.user_stats = None;
    state.stats_loading = false;
    state.stats_at = None;
    state.cache.songs.clear();
    state.reaction = SongReaction::default();
    state.reaction_pending = false;
    state.favorites = FavoritesState::default();
    state.export = ExportState::default();
    state.profile_edit = ProfileEditForm::default();
    state.password = PasswordForm::default();
    state.delete_account = DeleteAccountForm::default();
    state.song_info.favorite_id = None;
    state.song_info.favorite_pending = false;
    close_windows(state, &ACCOUNT_WINDOWS)
}

fn expire_session(state: &mut Plaza) -> Task<Msg> {
    let close = sign_out(state);
    Task::batch([
        close,
        show_error(state, "Your session has expired. Please log in again."),
    ])
}

pub fn authed<T, F>(
    token: String,
    request: impl FnOnce(String) -> F,
    done: impl FnOnce(api::Result<T>) -> Msg + Send + 'static,
) -> Task<Msg>
where
    T: Send + 'static,
    F: Future<Output = api::Result<T>> + Send + 'static,
{
    let response = request(token.clone());
    Task::future(async move {
        match response.await {
            Err(e) if e.is_unauthorized() => Msg::Account(AccountMsg::Expired(token)),
            result => done(result),
        }
    })
}

pub fn refresh_stats(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    if state.stats_fresh() {
        return Task::none();
    }
    state.stats_loading = true;
    authed(token.clone(), api::fetch_stats, move |r| {
        Msg::Account(AccountMsg::Stats(token, r))
    })
}

pub fn update(state: &mut Plaza, msg: AccountMsg) -> Task<Msg> {
    match msg {
        AccountMsg::Checked(token, _)
        | AccountMsg::Expired(token)
        | AccountMsg::Stats(token, _)
            if !state.has_token(&token) =>
        {
            Task::none()
        }
        AccountMsg::Checked(_, Ok(user)) => {
            if let Some(mut session) = state.session.clone() {
                session.user = user;
                store_session(state, session, Persistence::Remember);
            }
            Task::none()
        }
        AccountMsg::Checked(_, Err(e)) => {
            if e.is_unauthorized() {
                expire_session(state)
            } else {
                show_error(state, format!("Could not verify your session: {e}"))
            }
        }
        AccountMsg::Expired(_) => expire_session(state),
        AccountMsg::Logout => match state.token() {
            Some(token) => Task::perform(api::logout(token), |r| {
                Msg::Account(AccountMsg::LoggedOut(r))
            }),
            None => update(state, AccountMsg::LoggedOut(Ok(()))),
        },
        AccountMsg::LoggedOut(result) => {
            let close = sign_out(state);
            match result {
                Ok(()) => close,
                Err(e) => Task::batch([close, show_error(state, e.to_string())]),
            }
        }
        AccountMsg::Stats(_, result) => {
            state.stats_loading = false;
            match result {
                Ok(stats) => {
                    state.user_stats = Some(stats);
                    state.stats_at = Some(Instant::now());
                    Task::none()
                }
                Err(e) => show_error(state, e.to_string()),
            }
        }
    }
}

pub fn login(state: &mut Plaza, msg: LoginMsg) -> Task<Msg> {
    let form = &mut state.login;
    match msg {
        LoginMsg::Username(s) => form.username = s,
        LoginMsg::Password(s) => form.password = s,
        LoginMsg::Remember(remember) => form.remember = remember,
        LoginMsg::Submit => {
            if form.loading {
                return Task::none();
            }
            if let Err(problem) = form.validate() {
                return show_error(state, problem);
            }
            form.loading = true;
            let request = api::login(form.username.clone(), form.password.clone(), form.remember);
            return Task::perform(request, |r| Msg::Login(LoginMsg::Done(r)));
        }
        LoginMsg::Done(Ok(response)) => {
            let persistence = if form.remember {
                Persistence::Remember
            } else {
                Persistence::Forget
            };
            state.login = LoginForm::default();
            let session = Session {
                token: response.token,
                user: response.data,
            };
            store_session(state, session, persistence);
            state.cache.songs.clear();
            return Task::batch([
                close_windows(state, &[WindowKind::UserLogin]),
                song_info::reload(state),
            ]);
        }
        LoginMsg::Done(Err(e)) => {
            form.loading = false;
            return show_error(state, e.to_string());
        }
    }
    Task::none()
}

pub fn register(state: &mut Plaza, msg: RegisterMsg) -> Task<Msg> {
    let form = &mut state.register;
    match msg {
        RegisterMsg::Username(s) => form.username = s,
        RegisterMsg::Email(s) => form.email = s,
        RegisterMsg::Password(s) => form.password = s,
        RegisterMsg::PasswordRepeat(s) => form.password_repeat = s,
        RegisterMsg::Submit => {
            if form.loading {
                return Task::none();
            }
            if let Err(problem) = form.validate() {
                return show_error(state, problem);
            }
            form.loading = true;
            let request = api::register(
                form.username.clone(),
                form.email.clone(),
                form.password.clone(),
            );
            return Task::perform(request, |r| Msg::Register(RegisterMsg::Done(r)));
        }
        RegisterMsg::Done(Ok(_)) => {
            state.register = RegisterForm::default();
            let close = close_windows(state, &[WindowKind::UserRegister]);
            return Task::batch([
                close,
                show_info(state, "Registration successful! You can now log in."),
            ]);
        }
        RegisterMsg::Done(Err(e)) => {
            form.loading = false;
            return show_error(state, e.to_string());
        }
    }
    Task::none()
}

pub fn profile_edit(state: &mut Plaza, msg: ProfileEditMsg) -> Task<Msg> {
    let form = &mut state.profile_edit;
    match msg {
        ProfileEditMsg::Username(s) => form.username = s,
        ProfileEditMsg::Email(s) => form.email = s,
        ProfileEditMsg::CurrentPassword(s) => form.current_password = s,
        ProfileEditMsg::Submit => {
            if form.loading {
                return Task::none();
            }
            if let Err(problem) = form.validate() {
                return show_error(state, problem);
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.profile_edit;
            form.loading = true;
            let submitted = ProfileUpdate {
                token: token.clone(),
                username: form.username.clone(),
                email: form.email.clone(),
            };
            let request = api::update_profile(
                token,
                form.current_password.clone(),
                form.username.clone(),
                form.email.clone(),
            );
            return Task::perform(request, move |r| {
                Msg::ProfileEdit(ProfileEditMsg::Done(submitted, r))
            });
        }
        ProfileEditMsg::Done(submitted, Ok(())) => {
            let submitted_here = form.loading;
            form.loading = false;
            if submitted_here {
                form.current_password.clear();
            }
            let session = state.session.clone();
            if let Some(mut session) = session.filter(|s| s.token == submitted.token) {
                session.user.username = submitted.username;
                session.user.email = submitted.email;
                let persistence = if state.config.session.is_some() {
                    Persistence::Remember
                } else {
                    Persistence::Forget
                };
                store_session(state, session, persistence);
            }
            let close = if submitted_here {
                close_windows(state, &[WindowKind::UserProfileEdit])
            } else {
                Task::none()
            };
            return Task::batch([close, show_info(state, "Profile has been updated.")]);
        }
        ProfileEditMsg::Done(_, Err(e)) => {
            form.loading = false;
            return show_error(state, e.to_string());
        }
    }
    Task::none()
}

pub fn password(state: &mut Plaza, msg: PasswordMsg) -> Task<Msg> {
    let form = &mut state.password;
    match msg {
        PasswordMsg::Current(s) => form.current_password = s,
        PasswordMsg::New(s) => form.password = s,
        PasswordMsg::Repeat(s) => form.password_repeat = s,
        PasswordMsg::Submit => {
            if form.loading {
                return Task::none();
            }
            if let Err(problem) = form.validate() {
                return show_error(state, problem);
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.password;
            form.loading = true;
            let request =
                api::update_password(token, form.current_password.clone(), form.password.clone());
            return Task::perform(request, |r| Msg::Password(PasswordMsg::Done(r)));
        }
        PasswordMsg::Done(Ok(())) => {
            let close = sign_out(state);
            return Task::batch([
                close,
                show_info(state, "Password updated. Please log in again."),
            ]);
        }
        PasswordMsg::Done(Err(e)) => {
            form.loading = false;
            return show_error(state, e.to_string());
        }
    }
    Task::none()
}

pub fn delete(state: &mut Plaza, msg: DeleteAccountMsg) -> Task<Msg> {
    let form = &mut state.delete_account;
    match msg {
        DeleteAccountMsg::Password(s) => form.current_password = s,
        DeleteAccountMsg::Confirm(confirmed) => form.confirmed = confirmed,
        DeleteAccountMsg::Submit => {
            if form.loading {
                return Task::none();
            }
            if let Err(problem) = form.validate() {
                return show_error(state, problem);
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.delete_account;
            form.loading = true;
            let request = api::delete_account(token, form.current_password.clone());
            return Task::perform(request, |r| Msg::DeleteAccount(DeleteAccountMsg::Done(r)));
        }
        DeleteAccountMsg::Done(Ok(())) => {
            let close = sign_out(state);
            return Task::batch([close, show_info(state, "Your account has been deleted.")]);
        }
        DeleteAccountMsg::Done(Err(e)) => {
            form.loading = false;
            return show_error(state, e.to_string());
        }
    }
    Task::none()
}
