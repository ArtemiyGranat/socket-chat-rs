use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult},
    Pool, Postgres,
};

pub async fn connect() -> Option<Pool<Postgres>> {
    dotenv::dotenv().ok()?;
    let db_url = std::env::var("DATABASE_URL").ok()?;

    PgPoolOptions::new().connect(&db_url).await.ok()
}

pub async fn _add_user(pool: &Pool<Postgres>, name: &str) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("insert into users(username) values ($1)", name)
        .execute(pool)
        .await
}
