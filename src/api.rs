use crate::net::{agent, blocking, read_body};
use iced::widget::image;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Read;

const API: &str = "https://api.plaza.one";

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

#[derive(Debug, Clone, Copy, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatingsRange {
    AllTime,
    Monthly,
    Weekly,
}

impl RatingsRange {
    fn path(self) -> &'static str {
        match self {
            RatingsRange::AllTime => "overtime",
            RatingsRange::Monthly => "monthly",
            RatingsRange::Weekly => "weekly",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingEntry {
    pub song: BasicSong,
    pub likes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub meta: PaginatedMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedMeta {
    pub last_page: u32,
    pub total: u32,
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
pub struct NewsArticle {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created_at: u64,
}

#[derive(Deserialize)]
struct Data<T> {
    data: T,
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
struct ApiErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub status: Option<u16>,
    pub message: String,
}

impl Error {
    pub fn is_unauthorized(&self) -> bool {
        matches!(self.status, Some(401 | 403))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}

impl From<String> for Error {
    fn from(message: String) -> Error {
        Error {
            status: None,
            message,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

fn body_text(result: std::result::Result<ureq::Response, ureq::Error>) -> Result<String> {
    let (status, body) = read_body(result)?;
    let Some(code) = status else {
        return Ok(body);
    };
    let message = match serde_json::from_str::<ApiErrorBody>(&body) {
        Ok(err) => err.error.or(err.key),
        Err(_) => (!body.is_empty()).then_some(body),
    };
    Err(Error {
        status: Some(code),
        message: message.unwrap_or_else(|| format!("HTTP {code}")),
    })
}

fn parse_json<T: DeserializeOwned>(
    result: std::result::Result<ureq::Response, ureq::Error>,
) -> Result<T> {
    let body = body_text(result)?;
    serde_json::from_str::<T>(&body).map_err(|e| e.to_string().into())
}

fn parse_unit(result: std::result::Result<ureq::Response, ureq::Error>) -> Result<()> {
    body_text(result).map(|_| ())
}

fn auth(req: ureq::Request, token: &str) -> ureq::Request {
    req.set("Authorization", &format!("Bearer {token}"))
}

pub async fn fetch_status() -> Result<Status> {
    blocking(|| parse_json(agent().get(&format!("{API}/status")).call())).await
}

pub async fn fetch_history(page: u32) -> Result<HistoryResponse> {
    blocking(move || parse_json(agent().get(&format!("{API}/v2/history?page={page}")).call())).await
}

pub async fn fetch_ratings(range: RatingsRange, page: u32) -> Result<Paginated<RatingEntry>> {
    let url = format!("{API}/v2/ratings/{}?page={page}", range.path());
    blocking(move || parse_json(agent().get(&url).call())).await
}

pub async fn fetch_song(id: String) -> Result<SongResponse> {
    blocking(move || parse_json(agent().get(&format!("{API}/v2/songs/{id}")).call())).await
}

pub async fn fetch_news(page: u32) -> Result<Paginated<NewsArticle>> {
    blocking(move || parse_json(agent().get(&format!("{API}/v2/news?page={page}")).call())).await
}

pub async fn fetch_artwork(url: String, max_px: u32) -> Result<image::Handle> {
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

pub async fn login(username: String, password: String, remember: bool) -> Result<LoginResponse> {
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

pub async fn logout(token: String) -> Result<()> {
    blocking(move || {
        parse_unit(auth(agent().post(&format!("{API}/v2/auth/logout")), &token).call())
    })
    .await
}

pub async fn register(username: String, email: String, password: String) -> Result<User> {
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

pub async fn get_me(token: String) -> Result<User> {
    blocking(move || {
        let me: Data<User> =
            parse_json(auth(agent().get(&format!("{API}/v2/users/me")), &token).call())?;
        Ok(me.data)
    })
    .await
}

pub async fn get_stats(token: String) -> Result<UserStats> {
    blocking(move || {
        let stats: Data<UserStats> =
            parse_json(auth(agent().get(&format!("{API}/v2/users/me/stats")), &token).call())?;
        Ok(stats.data)
    })
    .await
}

pub async fn react(token: String, reaction: u8) -> Result<u32> {
    blocking(move || {
        let resp: ReactResponse = parse_json(
            auth(agent().post(&format!("{API}/v2/reactions")), &token)
                .send_json(serde_json::json!({ "reaction": reaction })),
        )?;
        Ok(resp.reactions)
    })
    .await
}

pub async fn fetch_favorites(token: String, page: u32) -> Result<Paginated<FavoriteEntry>> {
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

pub async fn add_favorite(token: String, song_id: String) -> Result<u64> {
    blocking(move || {
        let added: Data<FavoriteEntry> = parse_json(
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

pub async fn delete_favorite(token: String, id: u64) -> Result<()> {
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

pub async fn export_favorites(token: String) -> Result<String> {
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
    token: String,
    current_password: String,
    username: String,
    email: String,
) -> Result<()> {
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
    token: String,
    current_password: String,
    password: String,
) -> Result<()> {
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

pub async fn delete_profile(token: String, current_password: String) -> Result<()> {
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
