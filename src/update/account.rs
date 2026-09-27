use super::close_windows;
use crate::api;
use crate::config::{self, Session};
use crate::message::{
    AccountMsg, DeleteAccountMsg, LoginMsg, Msg, PasswordMsg, ProfileEditMsg, RegisterMsg,
};
use crate::state::{DeleteAccountForm, LoginForm, PasswordForm, Plaza, RegisterForm, SongReaction};
use crate::window::WindowKind;
use iced::Task;

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

fn sign_out(state: &mut Plaza) {
    state.session = None;
    state.config.session = None;
    config::save(&state.config);
    state.user_stats = None;
    state.reaction = SongReaction::default();
}

pub fn load_stats(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    state.stats_loading = true;
    Task::perform(api::fetch_stats(token), |r| {
        Msg::Account(AccountMsg::Stats(r))
    })
}

pub fn update(state: &mut Plaza, msg: AccountMsg) -> Task<Msg> {
    match msg {
        AccountMsg::Checked(Ok(user)) => {
            if let Some(mut session) = state.session.clone() {
                session.user = user;
                store_session(state, session, Persistence::Remember);
            }
            Task::none()
        }
        AccountMsg::Checked(Err(e)) => {
            if e.is_unauthorized() {
                sign_out(state);
                state.alert = Some("Your session has expired. Please log in again.".into());
            } else {
                state.alert = Some(format!("Could not verify your session: {e}"));
            }
            Task::none()
        }
        AccountMsg::Logout => match state.token() {
            Some(token) => Task::perform(api::logout(token), |r| {
                Msg::Account(AccountMsg::LoggedOut(r))
            }),
            None => update(state, AccountMsg::LoggedOut(Ok(()))),
        },
        AccountMsg::LoggedOut(result) => {
            sign_out(state);
            if let Err(e) = result {
                state.alert = Some(e.to_string());
            }
            close_windows(state, &[WindowKind::UserProfile])
        }
        AccountMsg::Stats(result) => {
            state.stats_loading = false;
            match result {
                Ok(stats) => state.user_stats = Some(stats),
                Err(e) => state.alert = Some(e.to_string()),
            }
            Task::none()
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
            if let Err(problem) = form.validate() {
                form.error = Some(problem.into());
                return Task::none();
            }
            form.loading = true;
            form.error = None;
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
            return close_windows(state, &[WindowKind::UserLogin]);
        }
        LoginMsg::Done(Err(e)) => {
            form.loading = false;
            form.error = Some(e.to_string());
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
            if let Err(problem) = form.validate() {
                form.error = Some(problem.into());
                return Task::none();
            }
            form.loading = true;
            form.error = None;
            let request = api::register(
                form.username.clone(),
                form.email.clone(),
                form.password.clone(),
            );
            return Task::perform(request, |r| Msg::Register(RegisterMsg::Done(r)));
        }
        RegisterMsg::Done(Ok(_)) => {
            state.register = RegisterForm::default();
            state.alert = Some("Registration successful! You can now log in.".into());
            return close_windows(state, &[WindowKind::UserRegister]);
        }
        RegisterMsg::Done(Err(e)) => {
            form.loading = false;
            form.error = Some(e.to_string());
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
            if let Err(problem) = form.validate() {
                form.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.profile_edit;
            form.loading = true;
            form.error = None;
            let request = api::update_profile(
                token,
                form.current_password.clone(),
                form.username.clone(),
                form.email.clone(),
            );
            return Task::perform(request, |r| Msg::ProfileEdit(ProfileEditMsg::Done(r)));
        }
        ProfileEditMsg::Done(Ok(())) => {
            form.loading = false;
            form.current_password.clear();
            let (username, email) = (form.username.clone(), form.email.clone());
            if let Some(mut session) = state.session.clone() {
                session.user.username = username;
                session.user.email = email;
                let persistence = if state.config.session.is_some() {
                    Persistence::Remember
                } else {
                    Persistence::Forget
                };
                store_session(state, session, persistence);
            }
            state.alert = Some("Profile has been updated.".into());
            return close_windows(state, &[WindowKind::UserProfileEdit]);
        }
        ProfileEditMsg::Done(Err(e)) => {
            form.loading = false;
            form.error = Some(e.to_string());
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
            if let Err(problem) = form.validate() {
                form.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.password;
            form.loading = true;
            form.error = None;
            let request =
                api::update_password(token, form.current_password.clone(), form.password.clone());
            return Task::perform(request, |r| Msg::Password(PasswordMsg::Done(r)));
        }
        PasswordMsg::Done(Ok(())) => {
            state.password = PasswordForm::default();
            sign_out(state);
            state.alert = Some("Password updated. Please log in again.".into());
            return close_windows(state, &[WindowKind::UserPassword, WindowKind::UserProfile]);
        }
        PasswordMsg::Done(Err(e)) => {
            form.loading = false;
            form.error = Some(e.to_string());
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
            if let Err(problem) = form.validate() {
                form.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let form = &mut state.delete_account;
            form.loading = true;
            form.error = None;
            let request = api::delete_account(token, form.current_password.clone());
            return Task::perform(request, |r| Msg::DeleteAccount(DeleteAccountMsg::Done(r)));
        }
        DeleteAccountMsg::Done(Ok(())) => {
            state.delete_account = DeleteAccountForm::default();
            sign_out(state);
            state.alert = Some("Your account has been deleted.".into());
            return close_windows(
                state,
                &[
                    WindowKind::UserProfileDelete,
                    WindowKind::UserProfileEdit,
                    WindowKind::UserProfile,
                ],
            );
        }
        DeleteAccountMsg::Done(Err(e)) => {
            form.loading = false;
            form.error = Some(e.to_string());
        }
    }
    Task::none()
}
