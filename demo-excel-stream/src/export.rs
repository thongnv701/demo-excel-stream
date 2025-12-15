use crate::db::DbPool;
use crate::error::AppError;
use crate::config::Config;
use rust_xlsxwriter::{Workbook, Worksheet, XlsxError};
use std::path::PathBuf;
use std::sync::Arc;
use rust_decimal::prelude::ToPrimitive;

pub async fn export_to_excel(
    pool: Arc<DbPool>,
    config: &Config,
    output_path: Option<PathBuf>,
) -> Result<PathBuf, AppError> {

    // Create workbook - write directly to file for memory efficiency
    let file_path = output_path.unwrap_or_else(|| {
        // Save into project root (current working directory)
        std::env::current_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join(format!("orders_export_{}.xlsx", chrono::Utc::now().timestamp()))
    });

    let mut workbook = Workbook::new();
    let mut worksheet = workbook.add_worksheet();
    worksheet.set_name("Orders")?;

    // Write headers
    write_headers(&mut worksheet)?;

    // Use batch fetching to minimize memory usage
    let batch_size = config.batch_size;
    let mut offset = 0;
    let mut row_index = 1u32; // Start after header row

    loop {
        // Fetch batch of rows
        let rows = {
            let client = pool.get_client().await;
            client
                .query(
                    &format!(
                        "SELECT id, order_number, customer_id, customer_name, customer_email, 
                         order_date, status, total_amount, shipping_address, city, state, 
                         country, postal_code, payment_method, payment_status, shipping_method, 
                         tracking_number, notes, created_at 
                         FROM orders ORDER BY id LIMIT {} OFFSET {}",
                        batch_size, offset
                    ),
                    &[],
                )
                .await?
        };

        if rows.is_empty() {
            break;
        }

        let rows_len = rows.len();

        // Write each row immediately to Excel (streaming approach)
        for row in rows {
            write_order_row(&mut worksheet, row_index, &row)?;
            row_index += 1;
        }

        offset += batch_size;

        // Log progress every 10k rows
        if offset % 10000 == 0 {
            println!("Exported {} rows...", offset);
        }

        // If we got fewer rows than batch_size, we're done
        if rows_len < batch_size {
            break;
        }
    }

    // Save workbook to file
    workbook.save(&file_path)?;

    println!("Export completed. Total rows: {}", row_index - 1);
    Ok(file_path)
}

fn write_headers(worksheet: &mut Worksheet) -> Result<(), XlsxError> {
    let headers = [
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
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col as u16, *header)?;
    }

    Ok(())
}

fn write_order_row(
    worksheet: &mut Worksheet,
    row: u32,
    db_row: &tokio_postgres::Row,
) -> Result<(), AppError> {
    let id: i32 = db_row.get(0);
    let order_number: String = db_row.get(1);
    let customer_id: i32 = db_row.get(2);
    let customer_name: String = db_row.get(3);
    let customer_email: String = db_row.get(4);
    let order_date: chrono::NaiveDate = db_row.get(5);
    let status: String = db_row.get(6);
    let total_amount: rust_decimal::Decimal = db_row.get(7);
    let shipping_address: String = db_row.get(8);
    let city: String = db_row.get(9);
    let state: String = db_row.get(10);
    let country: String = db_row.get(11);
    let postal_code: String = db_row.get(12);
    let payment_method: String = db_row.get(13);
    let payment_status: String = db_row.get(14);
    let shipping_method: String = db_row.get(15);
    let tracking_number: Option<String> = db_row.get(16);
    let notes: Option<String> = db_row.get(17);
    let created_at: chrono::NaiveDateTime = db_row.get(18);

    worksheet.write_number(row, 0, id as f64)?;
    worksheet.write_string(row, 1, &order_number)?;
    worksheet.write_number(row, 2, customer_id as f64)?;
    worksheet.write_string(row, 3, &customer_name)?;
    worksheet.write_string(row, 4, &customer_email)?;
    worksheet.write_string(row, 5, &order_date.to_string())?;
    worksheet.write_string(row, 6, &status)?;
    if let Some(v) = total_amount.to_f64() {
        worksheet.write_number(row, 7, v)?;
    } else {
        worksheet.write_string(row, 7, &total_amount.to_string())?;
    }
    worksheet.write_string(row, 8, &shipping_address)?;
    worksheet.write_string(row, 9, &city)?;
    worksheet.write_string(row, 10, &state)?;
    worksheet.write_string(row, 11, &country)?;
    worksheet.write_string(row, 12, &postal_code)?;
    worksheet.write_string(row, 13, &payment_method)?;
    worksheet.write_string(row, 14, &payment_status)?;
    worksheet.write_string(row, 15, &shipping_method)?;
    worksheet.write_string(row, 16, &tracking_number.unwrap_or_default())?;
    worksheet.write_string(row, 17, &notes.unwrap_or_default())?;
    worksheet.write_string(row, 18, &created_at.format("%Y-%m-%d %H:%M:%S").to_string())?;

    Ok(())
}

