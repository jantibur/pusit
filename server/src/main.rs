mod card;
mod balance;
mod product;
mod order;

use axum::{serve, Router, routing::post, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use tokio::net::TcpListener;
use card::{handle_make_card_lost, handle_generate_card, handle_get_total_active_card, handle_get_total_lost_card};
use balance::{handle_add_balance, handle_total_balance};
use product::{handle_add_product, handle_add_product_inventory, handle_get_products, handle_get_total_products};
use order::{handle_create_order, handle_deliver_order, handle_get_not_delivered, handle_get_delivered};

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
            balance INTEGER 
        );
        CREATE TABLE IF NOT EXISTS product (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            price INTEGER,
            inventory INTEGER
        );
        CREATE TABLE IF NOT EXISTS `order` (
            product TEXT, 
            reference TEXT,
            is_delivered INTEGER
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
        .route("/create_order", post(handle_create_order))
        .route("/deliver_order", post(handle_deliver_order))
        .route("/get_not_delivered", get(handle_get_not_delivered))
        .route("/get_delivered", get(handle_get_delivered))
        .route("/get_total_products", get(handle_get_total_products))
        .route("/get_total_active_card", get(handle_get_total_active_card))
        .route("/get_total_lost_card", get(handle_get_total_lost_card))
        .with_state(state)
        .layer(cors);

    println!("[SERVER] Hosting functions at: {}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
}
