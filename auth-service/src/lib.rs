pub mod config;
pub mod db;
pub mod models;
pub mod services;
pub mod handlers;
pub mod middleware;

pub use config::Config;
pub use db::create_pool;
pub use models::{User, Role, Permission};
pub use services::{generate_token, validate_token, Claims, hash_password, verify_password};

