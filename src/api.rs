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

#[derive(Debug, Clone, Default, Deserialize)]
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

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct SongData {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub length: f64,
    pub artwork_src: Option<String>,
    pub preview_src: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SongStats {
    #[serde(default)]
    pub likes: u32,
    #[serde(default)]
    pub first_played_at: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SongResponse {
    pub data: SongData,
    pub stats: SongStats,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LoginResponse {
    pub data: User,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MeResponse {
    pub data: User,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserStatsData {
    #[serde(default)]
    pub reactions: u32,
    #[serde(default)]
    pub favorites: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserStatsResponse {
    pub data: UserStatsData,
}

#[derive(Debug, Clone, Default, Deserialize)]
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExportLink {
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NewsArticle {
    pub id: u64,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PaginatedMeta {
    pub current_page: u32,
    pub last_page: u32,
    pub per_page: u32,
    pub total: u32,
}

const API: &str = "https://api.plaza.one";
const STATUS_URL: &str = "https://api.plaza.one/status";

use crate::net::{agent, blocking};
use serde::de::DeserializeOwned;

fn body_text(result: Result<ureq::Response, ureq::Error>) -> Result<String, String> {
    match result {
        Ok(resp) => resp.into_string().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
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
        Err(e) => Err(e.to_string()),
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

pub async fn fetch_artwork(url: &str) -> Result<Vec<u8>, String> {
    let url = url.to_string();
    blocking(move || {
        let resp = agent().get(&url).call().map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    })
    .await
}

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let (username, password) = (username.to_string(), password.to_string());
    blocking(move || {
        parse_json(
            agent()
                .post(&format!("{API}/v2/auth/token"))
                .send_json(serde_json::json!({
                    "username": username,
                    "password": password,
                    "remember": true,
                })),
        )
    })
    .await
}

pub async fn logout(token: &str) -> Result<(), String> {
    let token = token.to_string();
    blocking(move || {
        agent()
            .post(&format!("{API}/v2/auth/logout"))
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map(|_| ())
            .map_err(|e| format!("Logout failed: {e}"))
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
        let me: MeResponse = parse_json(
            agent()
                .get(&format!("{API}/v2/users/me"))
                .set("Authorization", &format!("Bearer {token}"))
                .call(),
        )?;
        Ok(me.data)
    })
    .await
}

pub async fn get_stats(token: &str) -> Result<UserStatsResponse, String> {
    let token = token.to_string();
    blocking(move || {
        parse_json(
            agent()
                .get(&format!("{API}/v2/users/me/stats"))
                .set("Authorization", &format!("Bearer {token}"))
                .call(),
        )
    })
    .await
}

pub async fn react(token: &str, reaction: u8) -> Result<ReactResponse, String> {
    let token = token.to_string();
    blocking(move || {
        parse_json(
            agent()
                .post(&format!("{API}/v2/reactions"))
                .set("Authorization", &format!("Bearer {token}"))
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
            agent()
                .get(&format!("{API}/v2/users/me/favorites?page={page}"))
                .set("Authorization", &format!("Bearer {token}"))
                .call(),
        )
    })
    .await
}

pub async fn add_favorite(token: &str, song_id: &str) -> Result<u64, String> {
    let (token, song_id) = (token.to_string(), song_id.to_string());
    blocking(move || {
        let added: AddFavoriteResponse = parse_json(
            agent()
                .post(&format!("{API}/v2/users/me/favorites"))
                .set("Authorization", &format!("Bearer {token}"))
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
            agent()
                .delete(&format!("{API}/v2/users/me/favorites/{id}"))
                .set("Authorization", &format!("Bearer {token}"))
                .call(),
        )
    })
    .await
}

pub async fn export_favorites(token: &str) -> Result<String, String> {
    let token = token.to_string();
    blocking(move || {
        let link: ExportLink = parse_json(
            agent()
                .post(&format!("{API}/v2/users/me/favorites/export"))
                .set("Authorization", &format!("Bearer {token}"))
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
            agent()
                .put(&format!("{API}/v2/users/me"))
                .set("Authorization", &format!("Bearer {token}"))
                .send_json(serde_json::json!({
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
            agent()
                .put(&format!("{API}/v2/users/me/password"))
                .set("Authorization", &format!("Bearer {token}"))
                .send_json(serde_json::json!({
                    "current_password": current_password,
                    "password": password,
                })),
        )
    })
    .await
}

pub async fn delete_profile(token: &str, current_password: &str) -> Result<(), String> {
    let (token, current_password) = (token.to_string(), current_password.to_string());
    blocking(move || {
        parse_unit(
            agent()
                .request("DELETE", &format!("{API}/v2/users/me"))
                .set("Authorization", &format!("Bearer {token}"))
                .send_json(serde_json::json!({ "current_password": current_password })),
        )
    })
    .await
}
