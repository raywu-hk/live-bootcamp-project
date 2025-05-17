use dotenvy::dotenv;
use secrecy::SecretString;
use std::env as std_env;
use std::sync::LazyLock;

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

// Define lazily evaluated static. Lazy_static is needed because std_env::var is not a const function.
pub static JWT_SECRET: LazyLock<SecretString> = LazyLock::new(set_token);
pub static DATABASE_URL: LazyLock<SecretString> = LazyLock::new(get_db_url);
pub static REDIS_HOST_NAME: LazyLock<String> = LazyLock::new(set_redis_host);
fn get_db_url() -> SecretString {
    dotenv().ok();
    let url = std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set");
    if url.is_empty() {
        panic!("DATABASE_URL must be set");
    }
    SecretString::from(url)
}

fn set_token() -> SecretString {
    dotenv().ok(); // Load environment variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    SecretString::from(secret)
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

pub mod env {
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
