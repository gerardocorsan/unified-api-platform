mod handlers;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use clap::Parser;
use env_logger::Env;
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    log::info!("Starting Mock Service on {}:{}", args.host, args.port);

    // Discover and load all services
    log::info!("Discovering services...");
    let service_registry = match utils::discover_services() {
        Ok(registry) => {
            log::info!("Successfully loaded {} services", registry.services.len());
            Arc::new(registry)
        }
        Err(e) => {
            log::error!("Failed to discover services: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Service discovery failed: {}", e)));
        }
    };

    // Clone registry for the HttpServer closure
    let registry_clone = service_registry.clone();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let mut app = App::new()
            .app_data(web::Data::new(registry_clone.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // Utility endpoints for service management
            .route("/api/services", web::get().to(handlers::list_services))
            .route("/api/services/{service}", web::post().to(handlers::create_service))
            .route("/api/services/{service}/{method}", web::put().to(handlers::upload_mock_file))
            .route("/api/services/{service}", web::delete().to(handlers::delete_service))
            .route("/api/health", web::get().to(handlers::health_check))
            // Dynamic route handler (catches all paths)
            .route("/{path:.*}", web::get().to(handlers::handle_dynamic_request))
            .route("/{path:.*}", web::post().to(handlers::handle_dynamic_request))
            .route("/{path:.*}", web::put().to(handlers::handle_dynamic_request))
            .route("/{path:.*}", web::delete().to(handlers::handle_dynamic_request));

        // Legacy static routes for backward compatibility
        app = app
            .route("/{service}", web::get().to(handlers::handle_mock_request))
            .route("/{service}", web::post().to(handlers::handle_mock_request))
            .route("/{service}", web::put().to(handlers::handle_mock_request))
            .route("/{service}", web::delete().to(handlers::handle_mock_request));

        app
    })
    .bind((args.host, args.port))?
    .run()
    .await
}