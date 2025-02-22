use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, SslMode, tokio_postgres::NoTls};
use std::env;
use std::path::Path;
use tokio::fs;

pub async fn get_db_pool() -> Pool {
    dotenvy::dotenv().ok();

    let database_url = env::var("DB_URL").expect("DB_URL must be set");
    let mut cfg = Config::new();

    if Path::new("/root/.postgresql/root.crt").exists() {
        use postgres_native_tls::MakeTlsConnector;
        use native_tls::Certificate;

        let cert_bytes = fs::read("/root/.postgresql/root.crt").await.expect("Failed to read CA cert");
        let cert = Certificate::from_pem(&cert_bytes).expect("Failed to parse certificate");

        let tls_builder = native_tls::TlsConnector::builder()
            .add_root_certificate(cert)
            .build()
            .expect("Failed to create TLS connector");
        let connector = MakeTlsConnector::new(tls_builder);
        cfg.ssl_mode = Some(SslMode::Require);
        return cfg.create_pool(Some(Runtime::Tokio1), connector).unwrap();

    }

    cfg.url = Some(database_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}