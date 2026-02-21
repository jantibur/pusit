use axum::{extract::State, http::StatusCode};
use sqlx::Row;
use crate::AppState;
use std::fmt::Write;

pub async
fn handle_get_total_products(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let total_products_query = " 
        SELECT COUNT(*) 
        FROM product
    ";

    let total_products: i64 = sqlx::query_scalar(total_products_query)
        .fetch_one(&state.database)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "CANNOT GET PRODUCT COUNT".to_string())
        })?;

    Ok((StatusCode::OK, total_products.to_string()))
}

pub async
fn handle_get_products(State(state): State<AppState>) -> Result<(StatusCode, String), (StatusCode, String)>
{
    let get_products_query  = "
        SELECT * FROM product;
    ";

    let products = sqlx::query(get_products_query)
        .fetch_all(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
        })?;

    let mut product_string: String = String::new(); 
    
    for product in products {
        let id: i64 = product.try_get("id")
            .map_err(|e| {
                println!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
            })?;
        
        let name: String = product.try_get("name")
            .map_err(|e| {
                println!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
            })?;
        
        let price: i64 = product.try_get("price")   
            .map_err(|e| {
                println!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
            })?;
       
        let inventory: i64 = product.try_get("inventory")
            .map_err(|e| {
                println!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
            })?;

        write!(product_string, "{}|{}|{}|{},", id, name, price, inventory).unwrap();    
    }
    println!("Returning Product String: {:?}", product_string); 
    Ok((StatusCode::OK, product_string))
}

pub async
fn handle_add_product_inventory(State(state): State<AppState>, body: String) -> Result<(StatusCode, String), (StatusCode, String)>
{
    if body.len() < 3 { 
        return Err((StatusCode::BAD_REQUEST, "Invalid Data".to_string()));
    }

    let splitted_body: Vec<&str> = body.split(",").collect(); 
    
    let id: i64 = splitted_body[0]
        .parse::<i64>()
        .expect("Not a valid integer");
    
    let inventory: i64 = splitted_body[1]
        .parse::<i64>()
        .expect("Not a valid integer");
    
    let add_product_inventory_query = "
        UPDATE product
        SET inventory = inventory + ?
        WHERE id LIKE ?;
    ";

    sqlx::query(add_product_inventory_query)
        .bind(inventory)
        .bind(id)
        .execute(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
        })?;

    let updated_inventory_query = "
        SELECT inventory FROM product
        WHERE id LIKE ?;
    ";

    let updated_inventory: i64 = sqlx::query_scalar(updated_inventory_query)
        .bind(id)
        .fetch_one(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Cannot retrieve updated inventory".to_string())
        })?;

    println!("Updated {:?} inventory; Added: {:?}; Updated Inventory: {:?}", id, inventory, updated_inventory);
    Ok((StatusCode::OK, format!("{}", updated_inventory)))
}

pub async
fn handle_add_product(State(state): State<AppState>, body: String) -> Result<(StatusCode, String), (StatusCode, String)>
{
    // product name, product price, inventory

    if body.len() < 5 {
        return Err((StatusCode::BAD_REQUEST, "Invalid Data".to_string()));
    }

    let splitted_body: Vec<&str> = body.split(",").collect();

    let name: &str = splitted_body[0];
    
    let price: i64 = splitted_body[1]
        .parse::<i64>()
        .expect("Not a valid integer");
    
    let inventory: i64 = splitted_body[2]
        .parse::<i64>()
        .expect("Not a valid integer");

    let query = "
        INSERT INTO product (name, price, inventory) VALUES (?, ?, ?)
    ";

    sqlx::query(query)
        .bind(name)
        .bind(price)
        .bind(inventory)
        .execute(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string())
        })?;

    let product_id_query = "
        SELECT id FROM product
        WHERE UPPER(name) LIKE ?;
    ";

    let inserted_product_id: i64 = sqlx::query_scalar(product_id_query)
        .bind(format!("{}%", name.to_uppercase())) 
        .fetch_one(&state.database)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Cannot retrieve product ID".to_string())
        })?;

    println!("Added product: {:?}; price: {:?}, inventory: {:?}", name, price, inventory);
    Ok((StatusCode::OK, format!("{}", inserted_product_id)))
}

