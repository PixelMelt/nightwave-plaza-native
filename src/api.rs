use crate::net::{self, Error, blocking};
use iced::widget::image;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};
use std::io::Read;

const API: &str = "https://api.plaza.one";
const MAX_SONG_SECS: f64 = 86_400.0;
const MAX_ARTWORK_BYTES: u64 = 16 * 1024 * 1024;

pub type Result<T> = std::result::Result<T, Error>;

fn nullable<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn song_seconds<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<f64, D::Error> {
    let seconds: f64 = nullable(deserializer)?;
    Ok(seconds.clamp(0.0, MAX_SONG_SECS))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Count {
    Number(u32),
    Text(String),
}

fn count<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<u32, D::Error> {
    Ok(match Option::<Count>::deserialize(deserializer)? {
        Some(Count::Number(count)) => count,
        Some(Count::Text(text)) => text.parse().unwrap_or_default(),
        None => 0,
    })
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatusSong {
    #[serde(default)]
    pub id: String,
    #[serde(default, deserialize_with = "nullable")]
    pub artist: String,
    #[serde(default, deserialize_with = "nullable")]
    pub album: String,
    #[serde(default, deserialize_with = "nullable")]
    pub title: String,
    #[serde(default, deserialize_with = "song_seconds")]
    pub length: f64,
    pub artwork_src: Option<String>,
    #[serde(default)]
    pub reactions: u32,
    #[serde(default, deserialize_with = "song_seconds")]
    pub position: f64,
}

impl StatusSong {
    pub fn has_metadata(&self) -> bool {
        !self.artist.is_empty() && !self.title.is_empty()
    }
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
    #[serde(default, deserialize_with = "nullable")]
    pub artist: String,
    #[serde(default, deserialize_with = "nullable")]
    pub title: String,
    #[serde(default)]
    pub artwork_src: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    pub played_at: u64,
    pub song: BasicSong,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DateRange {
    pub from_date: u64,
    pub to_date: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub data: Vec<HistoryEntry>,
    pub meta: PageMeta,
    #[serde(default)]
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RatingsRange {
    #[default]
    AllTime,
    Monthly,
    Weekly,
}

impl RatingsRange {
    fn path(self) -> &'static str {
        match self {
            Self::AllTime => "overtime",
            Self::Monthly => "monthly",
            Self::Weekly => "weekly",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingEntry {
    pub song: BasicSong,
    pub likes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageMeta {
    pub last_page: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongData {
    #[serde(default)]
    pub id: String,
    #[serde(default, deserialize_with = "nullable")]
    pub artist: String,
    #[serde(default, deserialize_with = "nullable")]
    pub album: String,
    #[serde(default, deserialize_with = "nullable")]
    pub title: String,
    #[serde(default, deserialize_with = "song_seconds")]
    pub length: f64,
    pub artwork_src: Option<String>,
}

impl SongData {
    pub fn artwork_url(&self) -> Option<&str> {
        self.artwork_src.as_deref().filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongStats {
    #[serde(default, deserialize_with = "count")]
    pub likes: u32,
    #[serde(default)]
    pub first_played_at: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongUser {
    #[serde(default)]
    pub favorite_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongResponse {
    pub data: SongData,
    pub stats: SongStats,
    #[serde(default)]
    pub current_user: Option<SongUser>,
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
    pub token: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserStats {
    #[serde(default)]
    pub reactions: u32,
    #[serde(default)]
    pub favorites: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FavoriteSong {
    #[serde(default)]
    pub id: String,
    #[serde(default, deserialize_with = "nullable")]
    pub artist: String,
    #[serde(default, deserialize_with = "nullable")]
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
pub struct NewsArticle {
    #[serde(default, deserialize_with = "nullable")]
    pub text: String,
    #[serde(default, deserialize_with = "nullable")]
    pub author: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reaction {
    #[default]
    None,
    Like,
    Love,
}

impl Reaction {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Like,
            Self::Like => Self::Love,
            Self::Love => Self::None,
        }
    }

    fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Like => 1,
            Self::Love => 2,
        }
    }
}

#[derive(Deserialize)]
struct Data<T> {
    data: T,
}

#[derive(Deserialize)]
struct Created {
    id: u64,
}

#[derive(Deserialize)]
struct ReactResponse {
    reactions: u32,
}

#[derive(Deserialize)]
struct ExportLink {
    #[serde(default)]
    link: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    key: Option<String>,
}

async fn call(
    method: &'static str,
    path: String,
    token: Option<String>,
    body: Option<Value>,
) -> Result<String> {
    blocking(move || {
        let mut request = net::request(method, &format!("{API}{path}"));
        if let Some(token) = token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        let response = match body {
            Some(body) => request.send_json(body),
            None => request.call(),
        };
        let body = net::read_body(response)?;
        if !body.is_error() {
            return Ok(body.text);
        }
        let message = serde_json::from_str::<ErrorBody>(&body.text)
            .ok()
            .and_then(|parsed| parsed.error.or(parsed.key));
        Err(Error::http(
            body.status,
            message.unwrap_or_else(|| format!("HTTP {}", body.status)),
        ))
    })
    .await
}

fn parse<T: DeserializeOwned>(body: &str) -> Result<T> {
    serde_json::from_str(body).map_err(Error::other)
}

async fn get<T: DeserializeOwned>(path: String, token: Option<String>) -> Result<T> {
    parse(&call("GET", path, token, None).await?)
}

pub async fn fetch_status() -> Result<Status> {
    get("/status".into(), None).await
}

pub async fn fetch_history(page: u32) -> Result<HistoryResponse> {
    get(format!("/v2/history?page={page}"), None).await
}

pub async fn fetch_ratings(range: RatingsRange, page: u32) -> Result<Page<RatingEntry>> {
    get(format!("/v2/ratings/{}?page={page}", range.path()), None).await
}

pub async fn fetch_song(id: String, token: Option<String>) -> Result<SongResponse> {
    get(format!("/v2/songs/{id}"), token).await
}

pub async fn fetch_news(page: u32) -> Result<Page<NewsArticle>> {
    get(format!("/v2/news?page={page}"), None).await
}

pub async fn fetch_artwork(url: String, max_px: u32) -> Result<image::Handle> {
    blocking(move || {
        let response = net::request("GET", &url).call().map_err(Error::other)?;
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(MAX_ARTWORK_BYTES)
            .read_to_end(&mut bytes)
            .map_err(Error::other)?;
        let img = ::image::load_from_memory(&bytes)
            .map_err(Error::other)?
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

pub async fn login(username: String, password: String, remember: bool) -> Result<LoginResponse> {
    let body = json!({ "username": username, "password": password, "remember": remember });
    parse(&call("POST", "/v2/auth/token".into(), None, Some(body)).await?)
}

pub async fn logout(token: String) -> Result<()> {
    call("POST", "/v2/auth/logout".into(), Some(token), None).await?;
    Ok(())
}

pub async fn register(username: String, email: String, password: String) -> Result<User> {
    let body = json!({
        "username": username,
        "email": email,
        "password": password,
        "captcha_response": "",
    });
    parse(&call("POST", "/v2/users".into(), None, Some(body)).await?)
}

pub async fn fetch_me(token: String) -> Result<User> {
    let me: Data<User> = get("/v2/users/me".into(), Some(token)).await?;
    Ok(me.data)
}

pub async fn fetch_stats(token: String) -> Result<UserStats> {
    let stats: Data<UserStats> = get("/v2/users/me/stats".into(), Some(token)).await?;
    Ok(stats.data)
}

pub async fn react(token: String, reaction: Reaction) -> Result<u32> {
    let body = json!({ "reaction": reaction.code() });
    let response: ReactResponse =
        parse(&call("POST", "/v2/reactions".into(), Some(token), Some(body)).await?)?;
    Ok(response.reactions)
}

pub async fn fetch_favorites(token: String, page: u32) -> Result<Page<FavoriteEntry>> {
    get(format!("/v2/users/me/favorites?page={page}"), Some(token)).await
}

pub async fn add_favorite(token: String, song_id: String) -> Result<u64> {
    let body = json!({ "song_id": song_id });
    let path = "/v2/users/me/favorites".into();
    let added: Data<Created> = parse(&call("POST", path, Some(token), Some(body)).await?)?;
    Ok(added.data.id)
}

pub async fn delete_favorite(token: String, id: u64) -> Result<()> {
    let path = format!("/v2/users/me/favorites/{id}");
    call("DELETE", path, Some(token), None).await?;
    Ok(())
}

pub async fn export_favorites(token: String) -> Result<String> {
    let path = "/v2/users/me/favorites/export".into();
    let export: ExportLink = parse(&call("POST", path, Some(token), None).await?)?;
    Ok(export.link)
}

pub async fn update_profile(
    token: String,
    current_password: String,
    username: String,
    email: String,
) -> Result<()> {
    let body = json!({
        "current_password": current_password,
        "username": username,
        "email": email,
    });
    call("PUT", "/v2/users/me".into(), Some(token), Some(body)).await?;
    Ok(())
}

pub async fn update_password(
    token: String,
    current_password: String,
    password: String,
) -> Result<()> {
    let body = json!({ "current_password": current_password, "password": password });
    let path = "/v2/users/me/password".into();
    call("PUT", path, Some(token), Some(body)).await?;
    Ok(())
}

pub async fn delete_account(token: String, current_password: String) -> Result<()> {
    let body = json!({ "current_password": current_password });
    call("DELETE", "/v2/users/me".into(), Some(token), Some(body)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_favorite_tolerates_numeric_song_id() {
        let body = r#"{"data":{"id":2304570,"song":{"id":2298,"artist":"x"},"created_at":1}}"#;
        let added: Data<Created> = parse(body).expect("favorite response parses");
        assert_eq!(added.data.id, 2_304_570);
    }

    #[test]
    fn null_album_does_not_reject_the_song() {
        let status: Status =
            parse(r#"{"song":{"artist":"a","album":null,"title":"t","length":180},"listeners":1}"#)
                .expect("status parses");
        assert!(status.song.album.is_empty());
        assert_eq!(status.song.title, "t");
    }

    #[test]
    fn hostile_timing_is_clamped() {
        let status: Status = parse(
            r#"{"song":{"artist":"a","album":"b","title":"t","length":1e300,"position":-5},"listeners":1}"#,
        )
        .expect("status parses");
        assert!((status.song.length - MAX_SONG_SECS).abs() < f64::EPSILON);
        assert!(status.song.position.abs() < f64::EPSILON);
    }

    #[test]
    fn null_author_keeps_the_rest_of_the_page() {
        let page: Page<NewsArticle> = parse(
            r#"{"data":[{"text":"good"},{"text":"bad","author":null}],"meta":{"last_page":1,"total":2}}"#,
        )
        .expect("page parses");
        assert_eq!(page.data.len(), 2);
    }

    #[test]
    fn likes_accept_numbers_and_numeric_strings() {
        let number: SongStats = parse(r#"{"likes":21}"#).expect("number parses");
        let text: SongStats = parse(r#"{"likes":"4"}"#).expect("string parses");
        assert_eq!(number.likes, 21);
        assert_eq!(text.likes, 4);
    }

    #[test]
    fn song_exposes_the_favorite_of_the_current_user() {
        let song: SongResponse = parse(
            r#"{"data":{"artist":"a","album":null,"title":"t","length":1},"stats":{},"current_user":{"favorite_id":7}}"#,
        )
        .expect("song parses");
        assert_eq!(song.current_user.and_then(|user| user.favorite_id), Some(7));
    }
}
