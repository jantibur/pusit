mod card;
mod balance;
mod product;


use axum::{serve, Router, routing::post, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use tokio::net::TcpListener;
use card::{handle_make_card_lost, handle_generate_card};
use balance::{handle_add_balance, handle_total_balance};
use product::{handle_add_product, handle_add_product_inventory, handle_get_products};


#[derive(Clone)]
struct AppState {
    database: SqlitePool,
}

#[tokio::main]
async fn
main() -> Result<(), Box<dyn std::error::Error>>
{
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any);

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
   
    let options = SqliteConnectOptions::from_str("sqlite:server.db")?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
   
    let state = AppState { database: pool };

    let query = "
        CREATE TABLE IF NOT EXISTS card (
            is_lost INTEGER,
            uuid TEXT PRIMARY KEY,
            uid BLOB UNIQUE,
            server_uid BLOB,
            balance BOOLEAN 
        );
        CREATE TABLE IF NOT EXISTS product (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            price INTEGER,
            inventory INTEGER
        );
        CREATE TABLE IF NOT EXISTS transact (
            uuid TEXT PRIMARY KEY,
            product TEXT,
            price INTEGER
        );
    ";

    sqlx::query(query).execute(&state.database).await?;

    let app = Router::new()
        .route("/make-card-lost", post(handle_make_card_lost))
        .route("/generate-card", post(handle_generate_card))
        .route("/add_balance", post(handle_add_balance))
        .route("/total_balance", get(handle_total_balance))
        .route("/add_product", post(handle_add_product))
        .route("/add_product_inventory", post(handle_add_product_inventory))
        .route("/get_products", get(handle_get_products))
        .with_state(state)
        .layer(cors);

    println!("[SERVER] Hosting functions at: {}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}
