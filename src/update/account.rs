use super::{close_windows_of, task};
use crate::api;
use crate::config::{self, Session};
use crate::state::{
    AccountMsg, DeleteMsg, LoginMsg, Msg, PasswordMsg, Plaza, ProfileEditMsg, RegisterMsg, WinType,
};
use iced::Task;

fn store_session(state: &mut Plaza, session: Option<Session>, remember: bool) {
    if remember || session.is_none() {
        state.config.session = session.clone();
        config::save(&state.config);
    }
    state.session = session;
}

fn sign_out(state: &mut Plaza) {
    store_session(state, None, false);
    state.user_stats = None;
    state.reaction = Default::default();
}

pub fn load_stats(state: &mut Plaza) -> Task<Msg> {
    let Some(token) = state.token() else {
        return Task::none();
    };
    state.stats_loading = true;
    task(api::get_stats(token), |r| {
        Msg::Account(AccountMsg::Stats(r))
    })
}

pub fn update(state: &mut Plaza, msg: AccountMsg) -> Task<Msg> {
    match msg {
        AccountMsg::Checked(Ok(user)) => {
            if let Some(mut session) = state.session.clone() {
                session.user = user;
                store_session(state, Some(session), true);
            }
            Task::none()
        }
        AccountMsg::Checked(Err(e)) => {
            if e.is_unauthorized() {
                sign_out(state);
                state.error_msg = Some("Your session has expired. Please log in again.".into());
            } else {
                state.error_msg = Some(format!("Could not verify your session: {e}"));
            }
            Task::none()
        }
        AccountMsg::Logout => match state.token() {
            Some(token) => task(api::logout(token), |r| {
                Msg::Account(AccountMsg::LoggedOut(r))
            }),
            None => update(state, AccountMsg::LoggedOut(Ok(()))),
        },
        AccountMsg::LoggedOut(result) => {
            sign_out(state);
            if let Err(e) = result {
                state.error_msg = Some(e);
            }
            close_windows_of(state, WinType::UserProfile)
        }
        AccountMsg::Stats(result) => {
            state.stats_loading = false;
            match result {
                Ok(stats) => state.user_stats = Some(stats),
                Err(e) => state.error_msg = Some(e),
            }
            Task::none()
        }
    }
}

pub fn login(state: &mut Plaza, msg: LoginMsg) -> Task<Msg> {
    let login = &mut state.login;
    match msg {
        LoginMsg::Username(s) => login.username = s,
        LoginMsg::Password(s) => login.password = s,
        LoginMsg::Remember(b) => login.remember = b,
        LoginMsg::Submit => {
            if login.username.is_empty() || login.password.is_empty() {
                login.error = Some("Please enter a username and password.".into());
                return Task::none();
            }
            login.loading = true;
            login.error = None;
            return task(
                api::login(
                    login.username.clone(),
                    login.password.clone(),
                    login.remember,
                ),
                |r| Msg::Login(LoginMsg::Done(r)),
            );
        }
        LoginMsg::Done(Ok(resp)) => {
            let remember = login.remember;
            state.login = Default::default();
            store_session(
                state,
                Some(Session {
                    token: resp.token,
                    user: resp.data,
                }),
                remember,
            );
            return close_windows_of(state, WinType::UserLogin);
        }
        LoginMsg::Done(Err(e)) => {
            login.loading = false;
            login.error = Some(e);
        }
    }
    Task::none()
}

pub fn register(state: &mut Plaza, msg: RegisterMsg) -> Task<Msg> {
    let reg = &mut state.register;
    match msg {
        RegisterMsg::Username(s) => reg.username = s,
        RegisterMsg::Email(s) => reg.email = s,
        RegisterMsg::Password(s) => reg.password = s,
        RegisterMsg::PasswordRepeat(s) => reg.password_repeat = s,
        RegisterMsg::Submit => {
            let problem = if !reg
                .username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                Some("Username may only contain letters, numbers, and underscores.")
            } else if reg.username.len() < 4 {
                Some("Username is too short.")
            } else if reg.username.len() > 32 {
                Some("Username is too long.")
            } else if reg.password.len() < 3 {
                Some("Password is too short.")
            } else if reg.password != reg.password_repeat {
                Some("Passwords do not match.")
            } else if reg.email.is_empty() {
                Some("Email is required.")
            } else {
                None
            };
            if let Some(problem) = problem {
                reg.error = Some(problem.into());
                return Task::none();
            }
            reg.loading = true;
            reg.error = None;
            return task(
                api::register(
                    reg.username.clone(),
                    reg.email.clone(),
                    reg.password.clone(),
                ),
                |r| Msg::Register(RegisterMsg::Done(r)),
            );
        }
        RegisterMsg::Done(Ok(_)) => {
            state.register = Default::default();
            state.error_msg = Some("Registration successful! You can now log in.".into());
            return close_windows_of(state, WinType::UserRegister);
        }
        RegisterMsg::Done(Err(e)) => {
            reg.loading = false;
            reg.error = Some(e);
        }
    }
    Task::none()
}

pub fn profile_edit(state: &mut Plaza, msg: ProfileEditMsg) -> Task<Msg> {
    let edit = &mut state.profile_edit;
    match msg {
        ProfileEditMsg::Username(s) => edit.username = s,
        ProfileEditMsg::Email(s) => edit.email = s,
        ProfileEditMsg::CurrentPassword(s) => edit.current_password = s,
        ProfileEditMsg::Submit => {
            if edit.current_password.is_empty() {
                edit.error = Some("Current password is required.".into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let edit = &mut state.profile_edit;
            edit.loading = true;
            edit.error = None;
            return task(
                api::update_profile(
                    token,
                    edit.current_password.clone(),
                    edit.username.clone(),
                    edit.email.clone(),
                ),
                |r| Msg::ProfileEdit(ProfileEditMsg::Done(r)),
            );
        }
        ProfileEditMsg::Done(Ok(())) => {
            edit.loading = false;
            edit.current_password.clear();
            let (username, email) = (edit.username.clone(), edit.email.clone());
            if let Some(mut session) = state.session.clone() {
                session.user.username = username;
                session.user.email = email;
                let remember = state.config.session.is_some();
                store_session(state, Some(session), remember);
            }
            state.error_msg = Some("Profile has been updated.".into());
            return close_windows_of(state, WinType::UserProfileEdit);
        }
        ProfileEditMsg::Done(Err(e)) => {
            edit.loading = false;
            edit.error = Some(e);
        }
    }
    Task::none()
}

pub fn password(state: &mut Plaza, msg: PasswordMsg) -> Task<Msg> {
    let pw = &mut state.password;
    match msg {
        PasswordMsg::Current(s) => pw.current_password = s,
        PasswordMsg::New(s) => pw.password = s,
        PasswordMsg::Repeat(s) => pw.password_repeat = s,
        PasswordMsg::Submit => {
            let problem = if pw.current_password.is_empty() {
                Some("Current password is required.")
            } else if pw.password.len() < 3 {
                Some("Password is too short.")
            } else if pw.password != pw.password_repeat {
                Some("Passwords do not match.")
            } else {
                None
            };
            if let Some(problem) = problem {
                pw.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let pw = &mut state.password;
            pw.loading = true;
            pw.error = None;
            return task(
                api::update_password(token, pw.current_password.clone(), pw.password.clone()),
                |r| Msg::Password(PasswordMsg::Done(r)),
            );
        }
        PasswordMsg::Done(Ok(())) => {
            state.password = Default::default();
            sign_out(state);
            state.error_msg = Some("Password updated. Please log in again.".into());
            return Task::batch([
                close_windows_of(state, WinType::UserPassword),
                close_windows_of(state, WinType::UserProfile),
            ]);
        }
        PasswordMsg::Done(Err(e)) => {
            pw.loading = false;
            pw.error = Some(e);
        }
    }
    Task::none()
}

pub fn delete(state: &mut Plaza, msg: DeleteMsg) -> Task<Msg> {
    let del = &mut state.delete;
    match msg {
        DeleteMsg::Password(s) => del.current_password = s,
        DeleteMsg::Confirm(b) => del.confirm = b,
        DeleteMsg::Submit => {
            let problem = if !del.confirm {
                Some("You must confirm account deletion.")
            } else if del.current_password.is_empty() {
                Some("Current password is required.")
            } else {
                None
            };
            if let Some(problem) = problem {
                del.error = Some(problem.into());
                return Task::none();
            }
            let Some(token) = state.token() else {
                return Task::none();
            };
            let del = &mut state.delete;
            del.loading = true;
            del.error = None;
            return task(
                api::delete_profile(token, del.current_password.clone()),
                |r| Msg::DeleteAccount(DeleteMsg::Done(r)),
            );
        }
        DeleteMsg::Done(Ok(())) => {
            state.delete = Default::default();
            sign_out(state);
            state.error_msg = Some("Your account has been deleted.".into());
            return Task::batch([
                close_windows_of(state, WinType::UserProfileDelete),
                close_windows_of(state, WinType::UserProfileEdit),
                close_windows_of(state, WinType::UserProfile),
            ]);
        }
        DeleteMsg::Done(Err(e)) => {
            del.loading = false;
            del.error = Some(e);
        }
    }
    Task::none()
}
