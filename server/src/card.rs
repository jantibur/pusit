use axum::{extract::State, http::StatusCode, body::Bytes};
use uuid::Uuid;
use rand::rand_core::{TryRngCore, OsRng};
use sqlx;
use crate::AppState;

pub async
fn handle_get_total_lost_card(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let total_lost_card_query = " 
        SELECT COUNT(*) 
        FROM card
        WHERE is_lost = 1
    ";

    let total_lost_card: i64 = sqlx::query_scalar(total_lost_card_query)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT GET LOST CARD COUNT".to_string())
        })?;

    Ok((StatusCode::OK, total_lost_card.to_string()))
}

pub async
fn handle_get_total_active_card(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let total_active_card_query = " 
        SELECT COUNT(*) 
        FROM card
        WHERE is_lost = 0
    ";

    let total_active_card: i64 = sqlx::query_scalar(total_active_card_query)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT GET ACTIVE CARD COUNT".to_string())
        })?;

    Ok((StatusCode::OK, total_active_card.to_string()))
}

pub fn
generate_server_uid() -> [u8; 16]
{
    let mut keys = [0u8; 16];
    OsRng.try_fill_bytes(&mut keys).unwrap();
    keys
}

pub async fn
handle_make_card_lost(State(state): State<AppState>, body: String) -> Result<(StatusCode, String), (StatusCode, String)>
{
    if body.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Invalid UUID".to_string()));
    }
    
    let is_lost_query = "
        SELECT is_lost 
        FROM card 
        WHERE hex(uuid) LIKE ?;
    ";

    let is_lost = sqlx::query_scalar(is_lost_query)
        .bind(format!("{}%", body.to_uppercase()))
        .fetch_one(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Not Found".to_string())
        })?;

    let query = "
        UPDATE card
        SET is_lost = ? 
        WHERE hex(uuid) LIKE ?;
    ";
    
    let new_is_lost_status = if is_lost { 0 } else { 1 };

    let result = sqlx::query(query)
        .bind(new_is_lost_status)
        .bind(format!("{}%", body.to_uppercase()))
        .execute(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, "Not Found".to_string()));
    }

    println!("{:?} MARKED AS {:?} SUCCESS", body, if new_is_lost_status == 1 { "LOST" } else { "UNLOST" });
    Ok((StatusCode::OK, format!("Success: {}", if new_is_lost_status == 1 { "LOST" } else { "UNLOST" })))
}

pub async fn
handle_generate_card(State(state): State<AppState>, body: Bytes) -> Result<(StatusCode, [u8; 32]), (StatusCode, [u8; 32])>
{
    if body.len() < 4 {
        return Err((StatusCode::BAD_REQUEST, [0u8; 32]));
    }

    let query = "
        INSERT INTO card (is_lost, uuid, uid, server_uid, balance) VALUES (?, ?, ?, ?, ?)
    ";

    let uuid = Uuid::new_v4();

    let uuid_bytes = uuid.as_bytes();
    let uid = &body[0..4];
    let server_uid: [u8; 16] = generate_server_uid();

    let mut response: [u8; 32] = [0; 32];
    response[0..16].copy_from_slice(&server_uid);
    response[16..32].copy_from_slice(uuid_bytes);

    let result = sqlx::query(query)
        .bind(0)
        .bind(uuid)
        .bind(uid)
        .bind(&server_uid[0..16])
        .bind(0i64)
        .execute(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, [0u8; 32])
        })?;

    if result.rows_affected() == 0 {
        return Ok((StatusCode::BAD_REQUEST, [0u8; 32]))
    }

    println!(
        "UID: {}\nUUID: {:?}\nSERVER_UID: {}",
        hex::encode(uid), uuid, hex::encode(server_uid)
    );

    Ok((StatusCode::OK, response))

}
