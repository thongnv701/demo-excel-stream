//! Streaming PostgreSQL -> Excel export using excelstream and server-side cursor
//! Suitable for millions of rows with low memory footprint.

use chrono::{NaiveDate, NaiveDateTime, Utc};
use dotenv::dotenv;
use excelstream::types::CellValue;
use excelstream::writer::ExcelWriter;
use postgres::{Client, NoTls};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming export with excelstream (orders) ===\n");
    dotenv().ok();

    let connection_string = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/db");

    let output_file = "orders_export_streaming.xlsx";
    let batch_size: i32 = 500; // small batches to keep memory low

    println!("Config:");
    println!("  Output file: {}", output_file);
    println!("  Batch size: {}", batch_size);
    println!("  Query: SELECT … FROM orders ORDER BY id\n");

    let start = Instant::now();

    // Connect
    println!("Connecting to PostgreSQL...");
    let mut client = Client::connect(&connection_string, NoTls)?;
    println!("Connected.\n");

    // Transaction + cursor
    let mut tx = client.transaction()?;
    println!("Declaring server-side cursor...");
    tx.execute(
        r#"
        DECLARE orders_cursor CURSOR FOR
        SELECT id, order_number, customer_id, customer_name, customer_email,
               order_date, status, total_amount, shipping_address, city, state,
               country, postal_code, payment_method, payment_status, shipping_method,
               tracking_number, notes, created_at
        FROM orders
        ORDER BY id
        "#,
        &[],
    )?;

    // Excel write
    println!("Creating Excel workbook...");
    let mut writer = ExcelWriter::new(output_file)?;
    writer.set_flush_interval(500);
    writer.set_max_buffer_size(512 * 1024); // 512KB buffer to force frequent flushes

    writer.write_header([
        "ID",
        "Order Number",
        "Customer ID",
        "Customer Name",
        "Customer Email",
        "Order Date",
        "Status",
        "Total Amount",
        "Shipping Address",
        "City",
        "State",
        "Country",
        "Postal Code",
        "Payment Method",
        "Payment Status",
        "Shipping Method",
        "Tracking Number",
        "Notes",
        "Created At",
    ])?;
    println!("Header written.\n");

    let mut total_rows = 0usize;
    let mut batch_number = 0usize;
    let mut last_progress = Instant::now();

    println!("Starting streaming export...\n");

    loop {
        let batch_start = Instant::now();
        let fetch_query = format!("FETCH {} FROM orders_cursor", batch_size);
        let rows = tx.query(&fetch_query, &[])?;

        if rows.is_empty() {
            println!("\nNo more data. Export complete.");
            break;
        }

        batch_number += 1;
        let batch_len = rows.len();

        for row in rows {
            // Extract columns with tolerant parsing
            // NOTE: orders.id is an INTEGER, so use i32 and cast to i64 for Excel
            let id: i32 = row.get(0);
            let order_number: Option<String> = row.try_get(1).ok().flatten();
            let customer_id: i32 = row.get(2);
            let customer_name: Option<String> = row.try_get(3).ok().flatten();
            let customer_email: Option<String> = row.try_get(4).ok().flatten();
            let order_date: Option<NaiveDate> = row.try_get(5).ok().flatten();
            let status: Option<String> = row.try_get(6).ok().flatten();
            let total_amount: Option<f64> = row.try_get(7).ok().flatten();
            let shipping_address: Option<String> = row.try_get(8).ok().flatten();
            let city: Option<String> = row.try_get(9).ok().flatten();
            let state: Option<String> = row.try_get(10).ok().flatten();
            let country: Option<String> = row.try_get(11).ok().flatten();
            let postal_code: Option<String> = row.try_get(12).ok().flatten();
            let payment_method: Option<String> = row.try_get(13).ok().flatten();
            let payment_status: Option<String> = row.try_get(14).ok().flatten();
            let shipping_method: Option<String> = row.try_get(15).ok().flatten();
            let tracking_number: Option<String> = row.try_get(16).ok().flatten();
            let notes: Option<String> = row.try_get(17).ok().flatten();
            let created_at: NaiveDateTime = row
                .try_get(18)
                .unwrap_or_else(|_| Utc::now().naive_utc());

            let order_date_str = order_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            let created_at_str = created_at.format("%Y-%m-%d %H:%M:%S").to_string();

            writer.write_row_typed(&[
                CellValue::Int(id as i64),
                CellValue::String(order_number.unwrap_or_default()),
                CellValue::Int(customer_id as i64),
                CellValue::String(customer_name.unwrap_or_default()),
                CellValue::String(customer_email.unwrap_or_default()),
                CellValue::String(order_date_str),
                CellValue::String(status.unwrap_or_default()),
                CellValue::Float(total_amount.unwrap_or(0.0)),
                CellValue::String(shipping_address.unwrap_or_default()),
                CellValue::String(city.unwrap_or_default()),
                CellValue::String(state.unwrap_or_default()),
                CellValue::String(country.unwrap_or_default()),
                CellValue::String(postal_code.unwrap_or_default()),
                CellValue::String(payment_method.unwrap_or_default()),
                CellValue::String(payment_status.unwrap_or_default()),
                CellValue::String(shipping_method.unwrap_or_default()),
                CellValue::String(tracking_number.unwrap_or_default()),
                CellValue::String(notes.unwrap_or_default()),
                CellValue::String(created_at_str),
            ])?;
        }

        total_rows += batch_len;
        let batch_dur = batch_start.elapsed();

        if last_progress.elapsed() > Duration::from_secs(2) {
            let rows_per_sec = batch_len as f64 / batch_dur.as_secs_f64().max(0.001);
            println!(
                "  Batch {:>4} | Rows: {:>8} | Speed: {:>7.0} rows/sec | Batch: {:>5.2}s",
                batch_number,
                total_rows,
                rows_per_sec,
                batch_dur.as_secs_f64()
            );
            last_progress = Instant::now();
        }

        if batch_len < batch_size as usize {
            break;
        }
    }

    tx.execute("CLOSE orders_cursor", &[])?;
    tx.commit()?;

    println!("\nFinalizing Excel file...");
    writer.save()?;

    let dur = start.elapsed();
    println!("\n=== Streaming Export Stats ===");
    println!("Total rows: {}", total_rows);
    println!("Total time: {:?}", dur);
    println!(
        "Avg speed: {:.0} rows/sec",
        total_rows as f64 / dur.as_secs_f64().max(0.001)
    );
    println!("Output file: {}", output_file);

    if let Ok(meta) = std::fs::metadata(output_file) {
        let size_mb = meta.len() as f64 / 1_048_576.0;
        println!("File size: {:.2} MB", size_mb);
    }

    println!("\n✓ Export completed successfully with excelstream.");
    Ok(())
}

