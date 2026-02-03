use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

async fn root_handler() -> Json<Value>
{
    Json(json!({"message": "Welcome to the root of PUSIT server"}))
}

#[tokio::main]
async fn main() 
{
    // Modify this line in production
    // So that the only thing that can communicate with this server are:
    // Controller, and Terminals!

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any);


    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("[PUSIT-SERVER] Hosting Server On: {}", addr);
   
    let app = Router::new()
        .route("/", get(root_handler))
        .layer(cors); 

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}









