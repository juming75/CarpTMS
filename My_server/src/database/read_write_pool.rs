//! 高性能读写分离连接池

use crate::errors::AppError;
use sqlx::{Acquire, PgPool, Postgres};
use std::sync::Arc;

/// 读写分离连接池
pub struct ReadWritePool {
    read_pool: PgPool,
    write_pool: PgPool,
    #[allow(dead_code)]
    read_only_threshold: u32,
}

impl ReadWritePool {
    pub async fn new(read_url: &str, write_url: &str) -> Result<Self, AppError> {
        let read_pool = PgPool::connect(read_url).await?;
        let write_pool = PgPool::connect(write_url).await?;

        Ok(Self {
            read_pool,
            write_pool,
            read_only_threshold: 1000,
        })
    }

    pub async fn get_read_connection(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<Postgres>, AppError> {
        Ok(self.read_pool.acquire().await?)
    }

    pub async fn get_write_connection(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<Postgres>, AppError> {
        Ok(self.write_pool.acquire().await?)
    }

    pub fn read_pool(&self) -> &PgPool {
        &self.read_pool
    }

    pub fn write_pool(&self) -> &PgPool {
        &self.write_pool
    }

    pub async fn execute_read<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::PgConnection) -> Result<T, AppError>,
    {
        let mut conn = self.read_pool.acquire().await?;
        f(&mut conn)
    }

    pub async fn execute_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::PgConnection) -> Result<T, AppError>,
    {
        let mut conn = self.write_pool.acquire().await?;
        let mut tx = conn.begin().await?;

        let result = match f(&mut tx) {
            Ok(t) => {
                tx.commit().await?;
                Ok(t)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        };

        result
    }
}

/// 全局读写分离池（懒加载）
static READ_WRITE_POOL: once_cell::sync::Lazy<Arc<ReadWritePool>> =
    once_cell::sync::Lazy::new(|| {
        let read_url = std::env::var("DATABASE_READ_URL").unwrap_or_else(|_| {
            std::env::var("DATABASE_URL")
                .unwrap_or("postgres://user:pass@localhost:5432/carptms".to_string())
        });
        let write_url = std::env::var("DATABASE_WRITE_URL").unwrap_or_else(|_| {
            std::env::var("DATABASE_URL")
                .unwrap_or("postgres://user:pass@localhost:5432/carptms".to_string())
        });

        Arc::new({
            let urls = (read_url, write_url);
            std::thread::spawn(move || {
                tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime for read-write pool")
                    .block_on(async {
                        ReadWritePool::new(&urls.0, &urls.1)
                            .await
                            .expect("Failed to initialize read-write database pool")
                    })
            })
            .join()
            .expect("Read-write pool initialization thread panicked")
        })
    });

pub fn get_read_write_pool() -> &'static Arc<ReadWritePool> {
    &READ_WRITE_POOL
}
