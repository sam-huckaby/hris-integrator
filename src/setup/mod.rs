// setup.rs
use actix_web::{web, HttpResponse, Responder};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct RegistrationInfo {
    pub tenant_id: String,
    pub realm_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct IntegrationInfo {
    pub account_uuid: String,
    pub bamboo_hr_api_key: String,
}

pub async fn handle_register(info: web::Json<RegistrationInfo>) -> impl Responder {
    if !is_valid_input(&info.tenant_id) || !is_valid_input(&info.realm_id) {
        return HttpResponse::BadRequest().body("Invalid tenantId or realmId");
    }

    let conn = match Connection::open("integrator_storage.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to connect to database: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Check if tenant_id and realm_id already exist
    let mut stmt = match conn
        .prepare("SELECT tenant_id FROM accounts WHERE tenant_id = ?1 AND realm_id = ?2")
    {
        Ok(stmt) => stmt,
        Err(e) => {
            eprintln!("Failed to prepare statement: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let existing_tenant: Result<String, rusqlite::Error> =
        stmt.query_row(params![&info.tenant_id, &info.realm_id], |row| row.get(0));

    if existing_tenant.is_ok() {
        return HttpResponse::Conflict()
            .body("The provided tenant and realm are already configured");
    }

    let api_key = Uuid::new_v4().to_string();

    // Insert new account if it doesn't exist
    if let Err(e) = conn.execute(
        "INSERT INTO accounts (tenant_id, realm_id, api_key) VALUES (?1, ?2, ?3)",
        params![&info.tenant_id, &info.realm_id, &api_key],
    ) {
        eprintln!("Failed to insert data: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(api_key)
}

pub async fn handle_integration(info: web::Json<IntegrationInfo>) -> impl Responder {
    let conn = match Connection::open("integrator_storage.db") {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to connect to database: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Validate UUID
    let uuid = match Uuid::parse_str(&info.account_uuid) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid UUID"),
    };

    if let Err(e) = conn.execute(
        "INSERT INTO integrations (account_key, bamboo_hr_api_key) VALUES (?1, ?2)",
        params![uuid.to_string(), &info.bamboo_hr_api_key],
    ) {
        eprintln!("Failed to insert integration data: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Created().finish()
}

fn is_valid_input(input: &str) -> bool {
    input.len() == 10 && input.chars().all(|c| c.is_alphanumeric())
}
