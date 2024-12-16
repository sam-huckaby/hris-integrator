// main.rs
mod setup;

use actix_web::{web, App, HttpServer};

fn initialize_database() -> Result<(), Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open("integrator_storage.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            tenant_id TEXT NOT NULL,
            realm_id TEXT NOT NULL,
            api_key TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS integrations (
            account_key TEXT NOT NULL,
            bamboo_hr_api_key TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = initialize_database() {
        eprintln!("Failed to initialize database: {:?}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Database initialization failed",
        ));
    }

    HttpServer::new(|| {
        App::new()
            .route("/register", web::post().to(setup::handle_register))
            .route("/integrate", web::post().to(setup::handle_integration))
    })
    .bind("127.0.0.1:8899")?
    .run()
    .await
}
