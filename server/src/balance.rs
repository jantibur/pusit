use axum::{extract::State, http::StatusCode}; use sqlx;
use crate::AppState;

pub async fn
handle_total_balance(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let total_balance_query = "SELECT COALESCE(SUM(balance), 0) FROM card;";

    let total_balance: i64 = sqlx::query_scalar(total_balance_query)
        .fetch_one(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "-".to_string())
        })?;
    
    Ok((StatusCode::OK, format!("₱{}", total_balance)))
}


pub async fn
handle_add_balance(State(state): State<AppState>, body: String) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let split_body: Vec<&str> = body.split(",").collect();
    let uuid: &str = split_body[0];
    let amount: &str = split_body[1];
    
    if uuid.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Invalid CARD UUID".to_string()));
    }

    let parsed_amount: i64 = amount
        .parse::<i64>()
        .expect("Not an integer");

    let update_balance = "
        UPDATE card
        SET balance = balance + ?
        WHERE hex(uuid) LIKE ?;
    ";

    let updated_balance_exec = sqlx::query(update_balance)
        .bind(parsed_amount)
        .bind(format!("{}%", uuid.to_uppercase()))
        .execute(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::BAD_REQUEST, "Card UUID Not Found".to_string())
        })?;

    if updated_balance_exec.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, "Card UUID Not Found".to_string()));
    }

    let updated_balance_query = "
        SELECT balance FROM card
        WHERE hex(uuid) LIKE ?;
    ";

    let updated_balance: i64 = sqlx::query_scalar(updated_balance_query)
        .bind(format!("{}%", uuid.to_uppercase()))
        .fetch_one(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::BAD_REQUEST, "Failed".to_string())
        })?;
   
    println!("Updated Card Balance: {:?}; Added Amount: ₱{:?}; Balance: ₱{:?};", uuid, parsed_amount, updated_balance);

    Ok((StatusCode::OK, format!("{}", updated_balance)))
}
