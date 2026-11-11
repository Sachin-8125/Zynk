use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthUser, error::AppError, AppState};

#[derive(Serialize, sqlx::FromRow)]
pub struct PostSummary {
    id: Uuid,
    author: String,
    title: String,
    slug: String,
    excerpt: String,
    published: bool,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Post {
    id: Uuid,
    author_id: Uuid,
    author: String,
    title: String,
    slug: String,
    excerpt: String,
    body: String,
    published: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

const POST_COLS: &str = "p.id, p.author_id, u.username AS author, p.title, p.slug, p.excerpt, p.body, p.published, p.created_at, p.updated_at";
const SUMMARY_COLS: &str =
    "p.id, u.username AS author, p.title, p.slug, p.excerpt, p.published, p.created_at";

#[derive(Deserialize)]
pub struct ListParams {
    page: Option<i64>,
    per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct PostList {
    posts: Vec<PostSummary>,
    page: i64,
    per_page: i64,
    total: i64,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<ListParams>,
) -> Result<Json<PostList>, AppError> {
    let per_page = q.per_page.unwrap_or(10).clamp(1, 50);
    let page = q.page.unwrap_or(1).max(1);

    let posts = sqlx::query_as::<_, PostSummary>(&format!(
        "SELECT {SUMMARY_COLS} FROM posts p JOIN users u ON u.id = p.author_id \
         WHERE p.published ORDER BY p.created_at DESC LIMIT $1 OFFSET $2"
    ))
    .bind(per_page)
    .bind((page - 1) * per_page)
    .fetch_all(&s.db)
    .await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts WHERE published")
        .fetch_one(&s.db)
        .await?;

    Ok(Json(PostList { posts, page, per_page, total }))
}

pub async fn get_by_slug(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = sqlx::query_as::<_, Post>(&format!(
        "SELECT {POST_COLS} FROM posts p JOIN users u ON u.id = p.author_id \
         WHERE p.slug = $1 AND p.published"
    ))
    .bind(slug)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(post))
}

pub async fn mine(
    State(s): State<AppState>,
    AuthUser(uid): AuthUser,
) -> Result<Json<Vec<PostSummary>>, AppError> {
    let posts = sqlx::query_as::<_, PostSummary>(&format!(
        "SELECT {SUMMARY_COLS} FROM posts p JOIN users u ON u.id = p.author_id \
         WHERE p.author_id = $1 ORDER BY p.updated_at DESC"
    ))
    .bind(uid)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(posts))
}

async fn fetch_own(s: &AppState, id: Uuid, uid: Uuid) -> Result<Post, AppError> {
    Ok(sqlx::query_as::<_, Post>(&format!(
        "SELECT {POST_COLS} FROM posts p JOIN users u ON u.id = p.author_id \
         WHERE p.id = $1 AND p.author_id = $2"
    ))
    .bind(id)
    .bind(uid)
    .fetch_one(&s.db)
    .await?)
}

pub async fn get_own(
    State(s): State<AppState>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Post>, AppError> {
    Ok(Json(fetch_own(&s, id, uid).await?))
}

#[derive(Deserialize)]
pub struct PostInput {
    title: String,
    body: String,
    excerpt: Option<String>,
    #[serde(default)]
    published: bool,
}

impl PostInput {
    fn validate(&self) -> Result<(), AppError> {
        let t = self.title.trim();
        if t.is_empty() || t.chars().count() > 200 {
            return Err(AppError::BadRequest("Title must be 1 to 200 characters".into()));
        }
        if self.body.trim().is_empty() {
            return Err(AppError::BadRequest("Write something in the body first".into()));
        }
        Ok(())
    }

    fn excerpt(&self) -> String {
        match self.excerpt.as_deref().map(str::trim) {
            Some(e) if !e.is_empty() => e.to_string(),
            _ => self
                .body
                .chars()
                .filter(|c| !"#*`>_".contains(*c))
                .take(160)
                .collect::<String>()
                .trim()
                .to_string(),
        }
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.trim_end_matches('-').chars().take(60).collect();
    if out.is_empty() { "post".into() } else { out }
}

pub async fn create(
    State(s): State<AppState>,
    AuthUser(uid): AuthUser,
    Json(input): Json<PostInput>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    input.validate()?;
    let id = Uuid::new_v4();
    let slug = format!("{}-{}", slugify(&input.title), &id.simple().to_string()[..6]);

    sqlx::query(
        "INSERT INTO posts (id, author_id, title, slug, excerpt, body, published) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(uid)
    .bind(input.title.trim())
    .bind(&slug)
    .bind(input.excerpt())
    .bind(&input.body)
    .bind(input.published)
    .execute(&s.db)
    .await?;

    Ok((StatusCode::CREATED, Json(fetch_own(&s, id, uid).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<PostInput>,
) -> Result<Json<Post>, AppError> {
    input.validate()?;
    let res = sqlx::query(
        "UPDATE posts SET title = $1, excerpt = $2, body = $3, published = $4, updated_at = now() \
         WHERE id = $5 AND author_id = $6",
    )
    .bind(input.title.trim())
    .bind(input.excerpt())
    .bind(&input.body)
    .bind(input.published)
    .bind(id)
    .bind(uid)
    .execute(&s.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(fetch_own(&s, id, uid).await?))
}

pub async fn remove(
    State(s): State<AppState>,
    AuthUser(uid): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM posts WHERE id = $1 AND author_id = $2")
        .bind(id)
        .bind(uid)
        .execute(&s.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
