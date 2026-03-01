use axum::{extract::State, http::StatusCode, body::Bytes};
use crate::AppState;
use chrono::Utc;
use sqlx::Row;

pub async fn
handle_get_delivered(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let not_delivered_query = "
        SELECT COUNT(*) 
        FROM `order` 
        WHERE is_delivered = 1;
    ";

    let not_delivered_count: i64 = sqlx::query_scalar(not_delivered_query)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT GET DELIVERED COUNT".to_string())
        })?;

    Ok((StatusCode::OK, not_delivered_count.to_string()))
}

pub async fn
handle_get_not_delivered(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let not_delivered_query = "
        SELECT COUNT(*) 
        FROM `order` 
        WHERE is_delivered = 0;
    ";

    let not_delivered_count: i64 = sqlx::query_scalar(not_delivered_query)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT GET NOT DELIVERED COUNT".to_string())
        })?;

    Ok((StatusCode::OK, not_delivered_count.to_string()))
}


pub async fn
handle_deliver_order(State(state): State<AppState>, body: String) -> Result<(StatusCode, String), (StatusCode, String)>
{
    if body.len() < 19 {
        return Err((StatusCode::BAD_REQUEST, "INVALID ORDER REFERENCE NUMBER".to_string()))
    }
    
    let get_order_query = "
        SELECT product, is_delivered FROM `order`
        WHERE reference = ?
    ";

    println!("Getting order for: {:?}", &body);

    let order = sqlx::query(get_order_query)
        .bind(&body)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "REFERENCE DOES NOT EXISTS".to_string())
        })?;


    let is_delivered: i64 = order.get("is_delivered");

    if is_delivered == 1 {
        return Err((StatusCode::UNAUTHORIZED, "ALREADY DELIVERED".to_string()))
    }

    let product_names_str: String = order.get("product");
 
    let mut tx = state.database.begin().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database transaction failed".to_string()))?;

    let set_delivered_query = "
        UPDATE `order`
        SET is_delivered = ?
        WHERE reference = ?;
    ";

    sqlx::query(set_delivered_query)
        .bind(1 as i64)
        .bind(&body)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT SET TO DELIVERED".to_string())
        })?;

    tx.commit().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO COMMIT TRANSACTION".to_string()))?;

    Ok((StatusCode::OK, product_names_str))
}


pub async fn
handle_create_order(State(state): State<AppState>, body: Bytes) -> Result<(StatusCode, String), (StatusCode, String)>
{
    if body.len() < 22 {
        return Err((StatusCode::BAD_REQUEST, "Invalid Data".to_string()));
    }

    let uid = &body[0..4];
    let server_uid_provided = &body[4..20]; 
    let ordered_products_raw = &body[22..];
    
    let ordered_products_str = String::from_utf8(ordered_products_raw.to_vec())
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UTF-8 in product list".to_string()))?;

    let ordered_products: Vec<&str> = ordered_products_str.split(',').collect();

    let mut tx = state.database.begin().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database transaction failed".to_string()))?;

    let mut total_price: i64 = 0;

    for product_id in &ordered_products {
        let price: i64 = sqlx::query_scalar("SELECT price FROM product WHERE id = ?")
            .bind(product_id)
            .fetch_one(&mut *tx) 
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, format!("Product {} not found", product_id)))?;
        total_price += price;
    }

    let card_hex = hex::encode(uid).to_uppercase();

    let card = sqlx::query("SELECT server_uid, is_lost, balance FROM card WHERE hex(uid) = ?")
        .bind(&card_hex)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| (StatusCode::FORBIDDEN, "CARD IS NOT A PUSIT CARD".to_string()))?;

    let server_uid_db: Vec<u8>  = card.get("server_uid");
    let is_lost: bool = card.get::<i64, _>("is_lost") != 0; 
    let balance: i64 = card.get("balance");

    if server_uid_db != server_uid_provided {
        return Err((StatusCode::UNAUTHORIZED, "PUSIT CARD INVALID".to_string()));
    }

    if is_lost {
        return Err((StatusCode::FORBIDDEN, "PUSIT CARD REPORTED AS LOST".to_string()));
    }

    if total_price > balance {
        return Err((StatusCode::PAYMENT_REQUIRED, format!("YOUR CARD BALANCE IS JUST ₱{}", balance)));
    }

    sqlx::query("UPDATE card SET balance = balance - ? WHERE hex(uid) = ?")
        .bind(total_price)
        .bind(&card_hex)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO UPDATED BALANCE".to_string()))?;


    for product_id in &ordered_products {
        sqlx::query("UPDATE product SET inventory = inventory - 1 WHERE id = ? AND inventory > 0")
            .bind(product_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO UPDATE INVENTORY".to_string()))?;
    }

    let reference = format!("{}{:05}", Utc::now().format("%Y%m%d%H%M%S"), rand::random::<u16>());
   
    let mut ordered_products_name: Vec<String> = Vec::new();
    
    for product_id in &ordered_products {
        let product_names = sqlx::query("SELECT name FROM product WHERE id = ?")             
            .bind(product_id)
            .fetch_one(&mut *tx) 
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, format!("Product {} not found", product_id)))?;
        
        let product_name: String = product_names.try_get("name")
            .map_err(|e| {
                println!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
            })?;

        ordered_products_name.push(product_name);
    }

    let ordered_products_name_str = ordered_products_name.join(",");

    sqlx::query("INSERT INTO `order` (product, reference, is_delivered) VALUES (?, ?, ?)")
        // Change this to ordered_products_name_str
        .bind(&ordered_products_name_str)
        .bind(&reference)
        .bind(0 as i64)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "ORDER FAILED".to_string()))?;

    tx.commit().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO COMMIT TRANSACTION".to_string()))?;

    println!("{:?} ({:?}) Ordered: {:?}; {:?}", card_hex, (balance - total_price), total_price, reference);

    Ok((StatusCode::OK, reference))
}
