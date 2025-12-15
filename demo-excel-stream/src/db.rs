use crate::config::Config;
use tokio_postgres::{Client, NoTls};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DbPool {
    client: Arc<Mutex<Client>>,
}

impl DbPool {
    pub async fn new(config: &Config) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(&config.database_url, NoTls).await?;

        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(DbPool {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub async fn get_client(&self) -> tokio::sync::MutexGuard<'_, Client> {
        self.client.lock().await
    }

    pub async fn execute_query(&self, query: &str) -> Result<u64, tokio_postgres::Error> {
        let client = self.get_client().await;
        client.execute(query, &[]).await
    }
}

