use std::{env, fs};
use std::path::Path;

pub(crate) async fn load_env() {
    let secret_path = "/run/secrets/";
    if Path::new(secret_path).exists() {
        println!("🔒 Loading environment variables from Docker secrets...");

        let secrets = [
            "DB_URL",
            "PASTE_ENCRYPTION_KEY",
            "RECAPTCHA_SECRET_KEY",
        ];

        for secret in secrets.iter() {
            let secret_path = Path::new(secret_path).join(secret);
            if secret_path.exists() {
                if let Ok(value) = fs::read_to_string(&secret_path) {
                    env::set_var(secret.to_uppercase(), value.trim());
                }
            }
        }
    } else {
        println!("🛠️  Loading environment variables from .env...");
        dotenvy::dotenv().ok();
    }
}