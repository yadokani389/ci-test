use std::net::{Ipv4Addr, SocketAddr};

use axum::{extract::State, http::StatusCode, routing::get};
use sqlx::{postgres::PgConnectOptions, PgPool};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "app".into(),
        password: "passwd".into(),
        database: "app".into(),
    };
    let conn_pool = connect_database_with(database_config);

    let app = axum::Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(conn_pool);
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8081);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {addr}");

    Ok(axum::serve(listener, app).await?)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    let connection_result = sqlx::query("SELECT 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from(config: DatabaseConfig) -> Self {
        Self::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
    }
}

fn connect_database_with(cfg: DatabaseConfig) -> PgPool {
    PgPool::connect_lazy_with(cfg.into())
}

#[tokio::test]
async fn test_health_check() {
    let response = health_check().await;
    assert_eq!(response, StatusCode::OK);
}

#[sqlx::test]
async fn test_health_check_db(pool: sqlx::PgPool) {
    let response = health_check_db(State(pool)).await;
    assert_eq!(response, StatusCode::OK);
}
