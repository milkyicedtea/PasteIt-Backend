use deadpool_postgres::Pool as PgPool;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) pool: PgPool,
}