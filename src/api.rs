use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatusSong {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub length: f64,
    pub artwork_src: Option<String>,
    #[serde(default)]
    pub reactions: u32,
    #[serde(default)]
    pub position: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Status {
    pub song: StatusSong,
    pub listeners: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BasicSong {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    pub played_at: u64,
    pub song: BasicSong,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateRange {
    pub from_date: u64,
    pub to_date: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub data: Vec<HistoryEntry>,
    pub meta: PaginatedMeta,
    #[serde(default)]
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingEntry {
    pub song: BasicSong,
    pub likes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginatedMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongData {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub length: f64,
    pub artwork_src: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongStats {
    #[serde(default)]
    pub likes: u32,
    #[serde(default)]
    pub first_played_at: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongResponse {
    pub data: SongData,
    pub stats: SongStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub data: User,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MeResponse {
    pub data: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserStatsData {
    #[serde(default)]
    pub reactions: u32,
    #[serde(default)]
    pub favorites: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserStatsResponse {
    pub data: UserStatsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactResponse {
    pub reactions: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FavoriteSong {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artwork_src: Option<String>,
    #[serde(default)]
    pub artwork_sm_src: Option<String>,
}

impl FavoriteSong {
    pub fn thumb_url(&self) -> Option<&str> {
        self.artwork_sm_src
            .as_deref()
            .or(self.artwork_src.as_deref())
            .filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FavoriteEntry {
    pub id: u64,
    pub song: FavoriteSong,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct AddFavoriteResponse {
    pub data: FavoriteEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct ExportLink {
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsArticle {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedMeta {
    pub last_page: u32,
    pub total: u32,
}

const API: &str = "https://api.plaza.one";
const STATUS_URL: &str = "https://api.plaza.one/status";

use crate::net::{agent, blocking, read_body};
use iced::widget::image;
use serde::de::DeserializeOwned;
use std::io::Read;

fn body_text(result: Result<ureq::Response, ureq::Error>) -> Result<String, String> {
    let (status, body) = read_body(result)?;
    let Some(code) = status else {
        return Ok(body);
    };
    if let Ok(err) = serde_json::from_str::<ApiErrorBody>(&body) {
        Err(err
            .error
            .or(err.key)
            .unwrap_or_else(|| format!("HTTP {code}")))
    } else if !body.is_empty() {
        Err(body)
    } else {
        Err(format!("HTTP {code}"))
    }
}

fn parse_json<T: DeserializeOwned>(
    result: Result<ureq::Response, ureq::Error>,
) -> Result<T, String> {
    let body = body_text(result)?;
    serde_json::from_str::<T>(&body).map_err(|e| e.to_string())
}

fn parse_unit(result: Result<ureq::Response, ureq::Error>) -> Result<(), String> {
    body_text(result).map(|_| ())
}

fn auth(req: ureq::Request, token: &str) -> ureq::Request {
    req.set("Authorization", &format!("Bearer {token}"))
}

pub async fn fetch_status() -> Result<Status, String> {
    blocking(|| parse_json(agent().get(STATUS_URL).call())).await
}

pub async fn fetch_history(page: u32) -> Result<HistoryResponse, String> {
    let url = format!("{API}/v2/history?page={page}");
    blocking(move || parse_json(agent().get(&url).call())).await
}

pub async fn fetch_ratings(
    range: &str,
    page: u32,
) -> Result<PaginatedResponse<RatingEntry>, String> {
    let url = format!("{API}/v2/ratings/{range}?page={page}");
    blocking(move || parse_json(agent().get(&url).call())).await
}

pub async fn fetch_song(id: &str) -> Result<SongResponse, String> {
    let url = format!("{API}/v2/songs/{id}");
    blocking(move || parse_json(agent().get(&url).call())).await
}

pub async fn fetch_news(page: u32) -> Result<PaginatedResponse<NewsArticle>, String> {
    let url = format!("{API}/v2/news?page={page}");
    blocking(move || parse_json(agent().get(&url).call())).await
}

/// Downloads an image and decodes it off the UI thread, shrinking it to at
/// most `max_px` on either side so the renderer never touches the original.
pub async fn fetch_artwork(url: &str, max_px: u32) -> Result<image::Handle, String> {
    let url = url.to_string();
    blocking(move || {
        let resp = agent().get(&url).call().map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        let img = ::image::load_from_memory(&buf)
            .map_err(|e| e.to_string())?
            .thumbnail(max_px, max_px)
            .into_rgba8();
        Ok(image::Handle::from_rgba(
            img.width(),
            img.height(),
            img.into_raw(),
        ))
    })
    .await
}

pub async fn login(
    username: &str,
    password: &str,
    remember: bool,
) -> Result<LoginResponse, String> {
    let (username, password) = (username.to_string(), password.to_string());
    blocking(move || {
        parse_json(
            agent()
                .post(&format!("{API}/v2/auth/token"))
                .send_json(serde_json::json!({
                    "username": username,
                    "password": password,
                    "remember": remember,
                })),
        )
    })
    .await
}

pub async fn logout(token: &str) -> Result<(), String> {
    let token = token.to_string();
    blocking(move || {
        parse_unit(auth(agent().post(&format!("{API}/v2/auth/logout")), &token).call())
    })
    .await
}

pub async fn register(username: &str, email: &str, password: &str) -> Result<User, String> {
    let (username, email, password) = (
        username.to_string(),
        email.to_string(),
        password.to_string(),
    );
    blocking(move || {
        parse_json(
            agent()
                .post(&format!("{API}/v2/users"))
                .send_json(serde_json::json!({
                    "username": username,
                    "email": email,
                    "password": password,
                    "captcha_response": "",
                })),
        )
    })
    .await
}

pub async fn get_me(token: &str) -> Result<User, String> {
    let token = token.to_string();
    blocking(move || {
        let me: MeResponse =
            parse_json(auth(agent().get(&format!("{API}/v2/users/me")), &token).call())?;
        Ok(me.data)
    })
    .await
}

pub async fn get_stats(token: &str) -> Result<UserStatsResponse, String> {
    let token = token.to_string();
    blocking(move || {
        parse_json(auth(agent().get(&format!("{API}/v2/users/me/stats")), &token).call())
    })
    .await
}

pub async fn react(token: &str, reaction: u8) -> Result<ReactResponse, String> {
    let token = token.to_string();
    blocking(move || {
        parse_json(
            auth(agent().post(&format!("{API}/v2/reactions")), &token)
                .send_json(serde_json::json!({ "reaction": reaction })),
        )
    })
    .await
}

pub async fn fetch_favorites(
    token: &str,
    page: u32,
) -> Result<PaginatedResponse<FavoriteEntry>, String> {
    let token = token.to_string();
    blocking(move || {
        parse_json(
            auth(
                agent().get(&format!("{API}/v2/users/me/favorites?page={page}")),
                &token,
            )
            .call(),
        )
    })
    .await
}

pub async fn add_favorite(token: &str, song_id: &str) -> Result<u64, String> {
    let (token, song_id) = (token.to_string(), song_id.to_string());
    blocking(move || {
        let added: AddFavoriteResponse = parse_json(
            auth(
                agent().post(&format!("{API}/v2/users/me/favorites")),
                &token,
            )
            .send_json(serde_json::json!({ "song_id": song_id })),
        )?;
        Ok(added.data.id)
    })
    .await
}

pub async fn delete_favorite(token: &str, id: u64) -> Result<(), String> {
    let token = token.to_string();
    blocking(move || {
        parse_unit(
            auth(
                agent().delete(&format!("{API}/v2/users/me/favorites/{id}")),
                &token,
            )
            .call(),
        )
    })
    .await
}

pub async fn export_favorites(token: &str) -> Result<String, String> {
    let token = token.to_string();
    blocking(move || {
        let link: ExportLink = parse_json(
            auth(
                agent().post(&format!("{API}/v2/users/me/favorites/export")),
                &token,
            )
            .call(),
        )?;
        Ok(link.link)
    })
    .await
}

pub async fn update_profile(
    token: &str,
    current_password: &str,
    username: &str,
    email: &str,
) -> Result<(), String> {
    let (token, current_password, username, email) = (
        token.to_string(),
        current_password.to_string(),
        username.to_string(),
        email.to_string(),
    );
    blocking(move || {
        parse_unit(
            auth(agent().put(&format!("{API}/v2/users/me")), &token).send_json(serde_json::json!({
                "current_password": current_password,
                "username": username,
                "email": email,
            })),
        )
    })
    .await
}

pub async fn update_password(
    token: &str,
    current_password: &str,
    password: &str,
) -> Result<(), String> {
    let (token, current_password, password) = (
        token.to_string(),
        current_password.to_string(),
        password.to_string(),
    );
    blocking(move || {
        parse_unit(
            auth(agent().put(&format!("{API}/v2/users/me/password")), &token).send_json(
                serde_json::json!({
                    "current_password": current_password,
                    "password": password,
                }),
            ),
        )
    })
    .await
}

pub async fn delete_profile(token: &str, current_password: &str) -> Result<(), String> {
    let (token, current_password) = (token.to_string(), current_password.to_string());
    blocking(move || {
        parse_unit(
            auth(
                agent().request("DELETE", &format!("{API}/v2/users/me")),
                &token,
            )
            .send_json(serde_json::json!({ "current_password": current_password })),
        )
    })
    .await
}
