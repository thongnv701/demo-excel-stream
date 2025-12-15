use crate::db::DbPool;
use crate::error::AppError;
use chrono::{NaiveDate, Utc};
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;

pub async fn insert_test_data(pool: Arc<DbPool>, total_rows: usize) -> Result<(), AppError> {
    let batch_size = 1000;
    let mut inserted = 0;

    println!("Starting to insert {} rows...", total_rows);

    for batch_start in (1500000..total_rows).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(total_rows);
        
        // Prepare statement per batch to ensure it's on the same connection
        let client = pool.get_client().await;
        let stmt = client
            .prepare(
                "INSERT INTO orders (
                    order_number, customer_id, customer_name, customer_email, order_date,
                    status, total_amount, shipping_address, city, state, country,
                    postal_code, payment_method, payment_status, shipping_method,
                    tracking_number, notes
                ) VALUES (
                    $1::varchar, $2::int, $3::varchar, $4::varchar, $5::date,
                    $6::varchar, $7::numeric, $8::varchar, $9::varchar, $10::varchar, $11::varchar,
                    $12::varchar, $13::varchar, $14::varchar, $15::varchar, $16::varchar, $17::text
                )
                ON CONFLICT (order_number) DO NOTHING",
            )
            .await?;
        
        for i in batch_start..batch_end {
            let order_number = format!("ORD-{:08}", i + 1);
            let customer_id = rand::thread_rng().gen_range(1..=100000);
            let customer_name = generate_name();
            let customer_email = format!("customer{}@example.com", customer_id);
            let order_date = generate_random_date();
            let status = generate_status();
            let total_amount = Decimal::from_f64(rand::thread_rng().gen_range(10.0..=5000.0))
                .unwrap_or_else(|| Decimal::new(0, 0));
            let shipping_address = generate_address();
            let city = generate_city();
            let state = generate_state();
            let country = generate_country();
            let postal_code = generate_postal_code();
            let payment_method = generate_payment_method();
            let payment_status = generate_payment_status();
            let shipping_method = generate_shipping_method();
            let tracking_number = if rand::thread_rng().gen_bool(0.8) {
                Some(format!("TRACK{:012}", rand::thread_rng().gen_range(100000000000i64..999999999999i64)))
            } else {
                None
            };
            let notes = if rand::thread_rng().gen_bool(0.3) {
                Some(generate_notes())
            } else {
                None
            };

            let exec_result = client
                .execute(
                    &stmt,
                    &[
                        &order_number,
                        &(customer_id as i32),
                        &customer_name,
                        &customer_email,
                        &order_date,
                        &status,
                        &total_amount,
                        &shipping_address,
                        &city,
                        &state,
                        &country,
                        &postal_code,
                        &payment_method,
                        &payment_status,
                        &shipping_method,
                        &tracking_number.as_deref(),
                        &notes.as_deref(),
                    ],
                )
                .await?;

            if exec_result > 0 {
                inserted += exec_result as usize;
            }
        }

        if batch_end % 10000 == 0 || batch_end == total_rows {
            println!("Inserted {} / {} rows", batch_end, total_rows);
        }
    }

    println!("Successfully inserted {} rows", inserted);
    Ok(())
}


fn generate_name() -> String {
    let first_names = ["John", "Jane", "Michael", "Sarah", "David", "Emily", "Robert", "Jessica", "William", "Ashley"];
    let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"];
    
    let first = first_names[rand::thread_rng().gen_range(0..first_names.len())];
    let last = last_names[rand::thread_rng().gen_range(0..last_names.len())];
    format!("{} {}", first, last)
}

fn generate_random_date() -> NaiveDate {
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let end = Utc::now().date_naive();
    let days = (end - start).num_days();
    start + chrono::Duration::days(rand::thread_rng().gen_range(0..=days))
}

fn generate_status() -> String {
    let statuses = ["Pending", "Processing", "Shipped", "Delivered", "Cancelled"];
    statuses[rand::thread_rng().gen_range(0..statuses.len())].to_string()
}

fn generate_address() -> String {
    let street_numbers = rand::thread_rng().gen_range(1..=9999);
    let street_names = ["Main St", "Oak Ave", "Park Blvd", "Elm St", "Maple Dr", "Cedar Ln", "Pine Rd", "First St"];
    format!("{} {}", street_numbers, street_names[rand::thread_rng().gen_range(0..street_names.len())])
}

fn generate_city() -> String {
    let cities = ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix", "Philadelphia", "San Antonio", "San Diego", "Dallas", "San Jose"];
    cities[rand::thread_rng().gen_range(0..cities.len())].to_string()
}

fn generate_state() -> String {
    let states = ["CA", "NY", "TX", "FL", "IL", "PA", "OH", "GA", "NC", "MI"];
    states[rand::thread_rng().gen_range(0..states.len())].to_string()
}

fn generate_country() -> String {
    let countries = ["USA", "Canada", "Mexico", "UK", "Germany", "France", "Japan", "Australia"];
    countries[rand::thread_rng().gen_range(0..countries.len())].to_string()
}

fn generate_postal_code() -> String {
    format!("{:05}", rand::thread_rng().gen_range(10000..=99999))
}

fn generate_payment_method() -> String {
    let methods = ["Credit Card", "Debit Card", "PayPal", "Bank Transfer", "Cash on Delivery"];
    methods[rand::thread_rng().gen_range(0..methods.len())].to_string()
}

fn generate_payment_status() -> String {
    let statuses = ["Paid", "Pending", "Failed", "Refunded"];
    statuses[rand::thread_rng().gen_range(0..statuses.len())].to_string()
}

fn generate_shipping_method() -> String {
    let methods = ["Standard", "Express", "Overnight", "Ground", "International"];
    methods[rand::thread_rng().gen_range(0..methods.len())].to_string()
}

fn generate_notes() -> String {
    let notes = [
        "Handle with care",
        "Fragile",
        "Gift wrap requested",
        "Leave at door",
        "Signature required",
        "Contact customer before delivery",
    ];
    notes[rand::thread_rng().gen_range(0..notes.len())].to_string()
}

