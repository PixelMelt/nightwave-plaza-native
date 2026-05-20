use serde::{Deserialize, Serialize};

// ── Status (/status) ─────────────────────────────────────────────
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

// ── History (v2/history) ─────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    pub played_at: u64,
    pub song: HistorySong,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistorySong {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub title: String,
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

// ── Ratings (v2/ratings/{range}) ─────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
pub struct RatingEntry {
    pub song: RatingSong,
    pub likes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingSong {
    #[serde(default)]
    pub id: String,
    pub artist: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingsResponse {
    pub data: Vec<RatingEntry>,
    pub meta: PaginatedMeta,
}

// ── Song Info (v2/songs/{id}) ────────────────────────────────────
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

// ── Auth / User ──────────────────────────────────────────────────
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

/// API error body: { "key": "...", "error": "..." }
#[derive(Debug, Clone, Default, Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

// ── News (v2/news) ───────────────────────────────────────────────
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
pub struct NewsResponse {
    pub data: Vec<NewsArticle>,
    pub meta: PaginatedMeta,
}

// ── Shared pagination ────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PaginatedMeta {
    pub current_page: u32,
    pub last_page: u32,
    pub per_page: u32,
    pub total: u32,
}

const API: &str = "https://api.plaza.one";

// ── Helper: parse response or extract error message ─────────────
async fn parse_response<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T, String> {
    if resp.status().is_success() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiErrorBody>(&body) {
            Err(err.error.or(err.key).unwrap_or_else(|| format!("HTTP {}", status)))
        } else if !body.is_empty() {
            Err(body)
        } else {
            Err(format!("HTTP {}", status))
        }
    }
}

// ── Public (unauthenticated) endpoints ──────────────────────────
pub async fn fetch_status() -> Result<Status, reqwest::Error> {
    reqwest::get(format!("{}/status", API)).await?.json().await
}

pub async fn fetch_history(page: u32) -> Result<HistoryResponse, reqwest::Error> {
    reqwest::get(format!("{}/v2/history?page={}", API, page)).await?.json().await
}

pub async fn fetch_ratings(range: &str, page: u32) -> Result<RatingsResponse, reqwest::Error> {
    reqwest::get(format!("{}/v2/ratings/{}?page={}", API, range, page)).await?.json().await
}

pub async fn fetch_song(id: &str) -> Result<SongResponse, reqwest::Error> {
    reqwest::get(format!("{}/v2/songs/{}", API, id)).await?.json().await
}

pub async fn fetch_news(page: u32) -> Result<NewsResponse, reqwest::Error> {
    reqwest::get(format!("{}/v2/news?page={}", API, page)).await?.json().await
}

pub async fn fetch_artwork(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    Ok(reqwest::get(url).await?.bytes().await?.to_vec())
}

// ── Authenticated endpoints (Bearer token) ──────────────────────

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v2/auth/token", API))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
            "remember": true,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn logout(token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v2/auth/logout", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Logout failed: HTTP {}", resp.status()))
    }
}

pub async fn register(username: &str, email: &str, password: &str) -> Result<User, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v2/users", API))
        .json(&serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
            "captcha_response": "",
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

/// Fetch the current user's profile using a Bearer token.
/// Used to restore a saved session on startup.
pub async fn get_me(token: &str) -> Result<User, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v2/users/me", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn get_stats(token: &str) -> Result<UserStatsResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v2/users/me/stats", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn react(token: &str, reaction: u8) -> Result<ReactResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v2/reactions", API))
        .bearer_auth(token)
        .json(&serde_json::json!({ "reaction": reaction }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}
