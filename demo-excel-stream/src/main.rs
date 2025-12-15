mod config;
mod db;
mod error;
mod export;
mod insert_data;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use dotenv::dotenv;

async fn insert_data_handler(
    pool: web::Data<Arc<db::DbPool>>,
) -> Result<impl Responder, error::AppError> {
    println!("Starting data insertion...");
    insert_data::insert_test_data(pool.get_ref().clone(), 1_596_496).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully inserted 1,000,000 rows"
    })))
}

async fn export_handler(
    pool: web::Data<Arc<db::DbPool>>,
    config: web::Data<config::Config>,
) -> Result<impl Responder, error::AppError> {
    println!("Starting export...");
    let file_path = export::export_to_excel(pool.get_ref().clone(), &config, None).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Export completed",
        "file_path": file_path.to_string_lossy()
    })))
}

async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = config::Config::from_env().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Config error: {}", e))
    })?;

    println!("Connecting to database: {}", config.database_url);
    let pool = Arc::new(
        db::DbPool::new(&config)
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database connection error: {}", e),
                )
            })?,
    );

    let server_address = config.server_address();
    println!("Starting server at http://{}", server_address);
    println!("Available endpoints:");
    println!("  POST /insert-data - Insert 1 million test records");
    println!("  GET  /export      - Export orders to Excel file");
    println!("  GET  /health       - Health check");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/insert-data", web::post().to(insert_data_handler))
            .route("/export", web::get().to(export_handler))
            .route("/health", web::get().to(health_handler))
    })
    .bind(&server_address)?
    .run()
    .await
}
