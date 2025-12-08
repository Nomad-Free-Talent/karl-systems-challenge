use actix_web::{web, App, HttpServer, Responder};
use auth_service::handlers;
use auth_service::{create_pool, Config};
use log::info;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    config: Config,
}

async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    let port = config.port;
    let database_url = config.database_url.clone();

    info!("Starting auth-service on port {}", port);

    // Initialize database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    let app_state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login)),
            )
            .service(
                web::scope("/admin")
                    .wrap(auth_service::middleware::JwtAuth::new(
                        config.jwt_secret.clone(),
                    ))
                    .wrap(auth_service::middleware::AdminAuth)
                    .service(
                        web::scope("/users")
                            .route("", web::get().to(handlers::admin::list_users))
                            .route("", web::post().to(handlers::admin::create_user))
                            .route("/{id}", web::get().to(handlers::admin::get_user))
                            .route("/{id}", web::put().to(handlers::admin::update_user))
                            .route("/{id}", web::delete().to(handlers::admin::delete_user)),
                    )
                    .service(
                        web::scope("/roles")
                            .route("", web::get().to(handlers::admin::list_roles))
                            .route("", web::post().to(handlers::admin::create_role)),
                    )
                    .service(
                        web::scope("/permissions")
                            .route("", web::get().to(handlers::admin::list_permissions))
                            .route("", web::post().to(handlers::admin::create_permission)),
                    )
                    .service(
                        web::scope("/users/{user_id}/roles")
                            .route("", web::post().to(handlers::admin::assign_role_to_user))
                            .route(
                                "/{role_id}",
                                web::delete().to(handlers::admin::remove_role_from_user),
                            ),
                    )
                    .service(web::scope("/roles/{role_id}/permissions").route(
                        "",
                        web::post().to(handlers::admin::assign_permission_to_role),
                    )),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
