mod auth;
mod error;
mod posts;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "longhand_api=debug,tower_http=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let origin = std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("could not connect to Postgres");
    sqlx::migrate!().run(&db).await.expect("migrations failed");

    let state = AppState { db, jwt_secret };

    let cors = CorsLayer::new()
        .allow_origin(origin.parse::<HeaderValue>().expect("invalid CORS_ORIGIN"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        // public reads
        .route("/api/posts", get(posts::list))
        .route("/api/posts/:slug", get(posts::get_by_slug))
        // author-only
        .route("/api/manage/posts", get(posts::mine).post(posts::create))
        .route(
            "/api/manage/posts/:id",
            get(posts::get_own).put(posts::update).delete(posts::remove),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("could not bind port");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
