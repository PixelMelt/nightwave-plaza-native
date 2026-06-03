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

fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    if resp.status().is_success() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiErrorBody>(&body) {
            Err(err
                .error
                .or(err.key)
                .unwrap_or_else(|| format!("HTTP {}", status)))
        } else if !body.is_empty() {
            Err(body)
        } else {
            Err(format!("HTTP {}", status))
        }
    }
}

async fn parse_result(resp: reqwest::Response) -> Result<(), String> {
    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiErrorBody>(&body) {
            Err(err
                .error
                .or(err.key)
                .unwrap_or_else(|| format!("HTTP {}", status)))
        } else if !body.is_empty() {
            Err(body)
        } else {
            Err(format!("HTTP {}", status))
        }
    }
}

pub async fn fetch_status() -> Result<Status, String> {
    let resp = client()
        .get(STATUS_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_history(page: u32) -> Result<HistoryResponse, String> {
    let resp = client()
        .get(format!("{}/v2/history?page={}", API, page))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_ratings(
    range: &str,
    page: u32,
) -> Result<PaginatedResponse<RatingEntry>, String> {
    let resp = client()
        .get(format!("{}/v2/ratings/{}?page={}", API, range, page))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_song(id: &str) -> Result<SongResponse, String> {
    let resp = client()
        .get(format!("{}/v2/songs/{}", API, id))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_news(page: u32) -> Result<PaginatedResponse<NewsArticle>, String> {
    let resp = client()
        .get(format!("{}/v2/news?page={}", API, page))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_artwork(url: &str) -> Result<Vec<u8>, String> {
    client()
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let resp = client()
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
    let resp = client()
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
    let resp = client()
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

pub async fn get_me(token: &str) -> Result<User, String> {
    let resp = client()
        .get(format!("{}/v2/users/me", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let me: MeResponse = parse_response(resp).await?;
    Ok(me.data)
}

pub async fn get_stats(token: &str) -> Result<UserStatsResponse, String> {
    let resp = client()
        .get(format!("{}/v2/users/me/stats", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn react(token: &str, reaction: u8) -> Result<ReactResponse, String> {
    let resp = client()
        .post(format!("{}/v2/reactions", API))
        .bearer_auth(token)
        .json(&serde_json::json!({ "reaction": reaction }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_favorites(
    token: &str,
    page: u32,
) -> Result<PaginatedResponse<FavoriteEntry>, String> {
    let resp = client()
        .get(format!("{}/v2/users/me/favorites?page={}", API, page))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn add_favorite(token: &str, song_id: &str) -> Result<u64, String> {
    let resp = client()
        .post(format!("{}/v2/users/me/favorites", API))
        .bearer_auth(token)
        .json(&serde_json::json!({ "song_id": song_id }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let added: FavoriteEntry = parse_response::<AddFavoriteResponse>(resp).await?.data;
    Ok(added.id)
}

pub async fn delete_favorite(token: &str, id: u64) -> Result<(), String> {
    let resp = client()
        .delete(format!("{}/v2/users/me/favorites/{}", API, id))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_result(resp).await
}

pub async fn export_favorites(token: &str) -> Result<String, String> {
    let resp = client()
        .post(format!("{}/v2/users/me/favorites/export", API))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let link: ExportLink = parse_response(resp).await?;
    Ok(link.link)
}

pub async fn update_profile(
    token: &str,
    current_password: &str,
    username: &str,
    email: &str,
) -> Result<(), String> {
    let resp = client()
        .put(format!("{}/v2/users/me", API))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "current_password": current_password,
            "username": username,
            "email": email,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_result(resp).await
}

pub async fn update_password(
    token: &str,
    current_password: &str,
    password: &str,
) -> Result<(), String> {
    let resp = client()
        .put(format!("{}/v2/users/me/password", API))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "current_password": current_password,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_result(resp).await
}

pub async fn delete_profile(token: &str, current_password: &str) -> Result<(), String> {
    let resp = client()
        .delete(format!("{}/v2/users/me", API))
        .bearer_auth(token)
        .json(&serde_json::json!({ "current_password": current_password }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_result(resp).await
}
