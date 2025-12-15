use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub batch_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/demo_excel_stream".to_string()
        });

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid SERVER_PORT value".to_string())?;

        let batch_size = env::var("BATCH_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .unwrap_or(1000);

        Ok(Config {
            database_url,
            server_host,
            server_port,
            batch_size,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
