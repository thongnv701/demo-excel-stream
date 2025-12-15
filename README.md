### demo-excel-stream

**Goal**: demo how to export ~1M+ PostgreSQL rows to Excel in Rust with a small memory footprint, via:
- **HTTP API** using `actix-web`
- **CLI streaming export** using `excelstream`

### Project layout

- **`demo-excel-stream/` (crate root)**
  - `src/lib.rs` – shared modules:
    - `config.rs` – env config (`DATABASE_URL`, `SERVER_HOST`, `SERVER_PORT`, `BATCH_SIZE`)
    - `db.rs` – lightweight PostgreSQL client wrapper
    - `error.rs` – simple `AppError` with Actix integration
    - `export.rs` – batch export to `.xlsx` using `rust_xlsxwriter`
    - `insert_data.rs` – random test data generator for the `orders` table
  - `src/bin/server.rs` – HTTP server:
    - `POST /insert-data` – seed ~1.6M random orders
    - `GET  /export` – export all orders to Excel
    - `GET  /health` – health check
  - `src/bin/export_stream.rs` – CLI streaming export using `excelstream`
  - `src/main.rs` – tiny helper telling you to use `--bin server`
- **`sql/schema.sql`** – schema for the `orders` table and indexes

### Prerequisites

- Rust (stable)
- PostgreSQL running locally

### 1. Clone & configure

```bash
git clone <this-repo-url>
cd demo-excel-stream

# Copy and edit environment variables
cp demo-excel-stream/.env.example demo-excel-stream/.env
```

Open `.env` and set at least:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/demo_excel_stream
```

Adjust user/password/host/port as needed.

### 2. Create database + schema

Create the database `demo_excel_stream` (or whatever you put into `DATABASE_URL`), then run:

```bash
psql "$DATABASE_URL" -f sql/schema.sql
```

This creates the `orders` table with 19 columns plus indexes.

### 3. Run the HTTP demo (`/insert-data` + `/export`)

In the crate directory (`demo-excel-stream/`):

```bash
cargo run --bin server
```

You should see log lines similar to:

- database connection info
- server address (default `127.0.0.1:8080`)
- available endpoints

**Endpoints:**

- `GET  /health`  
  Quick check:
  ```bash
  curl http://127.0.0.1:8080/health
  ```

- `POST /insert-data` – seed ~1.6M random orders  
  ```bash
  curl -X POST http://127.0.0.1:8080/insert-data
  ```

- `GET  /export` – export all orders to an `.xlsx` file  
  ```bash
  curl http://127.0.0.1:8080/export
  ```

The export endpoint responds with JSON containing the `file_path` of the generated Excel file, e.g.:

```json
{
  "message": "Export completed",
  "file_path": "E:/work/sendo/source/demo-excel-stream/orders_export_1700000000.xlsx"
}
```

### 4. Run the CLI streaming export (`excelstream`)

If you already have data in `orders`, you can run the pure streaming export example:

```bash
cargo run --bin export_stream
```

This:

- Opens a server-side cursor (`DECLARE orders_cursor CURSOR FOR SELECT … FROM orders ORDER BY id`)
- Fetches in small batches (default `batch_size = 500`)
- Streams directly to `orders_export_streaming.xlsx` with `excelstream`
- Prints progress: batch number, total rows exported, rows/sec, and final file size

### 5. How to talk about this in a blog / LinkedIn post

- **One-line pitch**:  
  “Export 1M+ PostgreSQL rows to Excel in Rust with low memory using Actix and excelstream.”
- **Key points to highlight**:
  - `export.rs` shows a *simple* batch export using `rust_xlsxwriter`.
  - `export_stream.rs` shows a *true streaming* approach using a server-side cursor + `excelstream`.
  - The crate is split into a clean library (`src/lib.rs`) and two entrypoints (`server` + `export_stream`).
  - Config is via `.env` with an example file so people can run it in a few commands.

You can now link to this repo, paste the quickstart, and readers should be able to:

1. Clone
2. Configure `.env`
3. Apply `sql/schema.sql`
4. Run `cargo run --bin server` or `cargo run --bin export_stream`
