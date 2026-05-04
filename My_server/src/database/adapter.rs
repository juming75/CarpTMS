//! 多数据库适配器 - 支持 PostgreSQL、GoldenDB 和达梦 DM8

use crate::errors::AppError;
use sqlx::{Acquire, MySqlPool, PgPool};
use std::sync::Arc;

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    PostgreSQL,
    GoldenDB,
    Dameng,
}

impl DatabaseType {
    pub fn from_env() -> Self {
        match std::env::var("DATABASE_TYPE").as_deref() {
            Ok("goldendb") | Ok("GoldenDB") => DatabaseType::GoldenDB,
            Ok("dameng") | Ok("Dameng") | Ok("DM8") => DatabaseType::Dameng,
            _ => DatabaseType::PostgreSQL,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::GoldenDB => "goldendb",
            DatabaseType::Dameng => "dameng",
        }
    }
}

/// 数据库连接池包装器
pub enum DatabasePool {
    PostgreSQL(Arc<PgPool>),
    GoldenDB(Arc<MySqlPool>),
    Dameng(Arc<sqlx::AnyPool>),
}

impl DatabasePool {
    pub fn db_type(&self) -> DatabaseType {
        match self {
            DatabasePool::PostgreSQL(_) => DatabaseType::PostgreSQL,
            DatabasePool::GoldenDB(_) => DatabaseType::GoldenDB,
            DatabasePool::Dameng(_) => DatabaseType::Dameng,
        }
    }

    pub fn as_postgres(&self) -> Option<&Arc<PgPool>> {
        match self {
            DatabasePool::PostgreSQL(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn as_goldendb(&self) -> Option<&Arc<MySqlPool>> {
        match self {
            DatabasePool::GoldenDB(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn as_dameng(&self) -> Option<&Arc<sqlx::AnyPool>> {
        match self {
            DatabasePool::Dameng(pool) => Some(pool),
            _ => None,
        }
    }
}

/// 读写分离连接池
pub struct ReadWritePool {
    read_pool: DatabasePool,
    write_pool: DatabasePool,
    db_type: DatabaseType,
}

impl ReadWritePool {
    pub async fn new(
        read_url: &str,
        write_url: &str,
        db_type: DatabaseType,
    ) -> Result<Self, AppError> {
        let (read_pool, write_pool) = match db_type {
            DatabaseType::PostgreSQL => {
                let read_pool = Arc::new(PgPool::connect(read_url).await?);
                let write_pool = Arc::new(PgPool::connect(write_url).await?);
                (
                    DatabasePool::PostgreSQL(read_pool),
                    DatabasePool::PostgreSQL(write_pool),
                )
            }
            DatabaseType::GoldenDB => {
                let read_pool = Arc::new(MySqlPool::connect(read_url).await?);
                let write_pool = Arc::new(MySqlPool::connect(write_url).await?);
                (
                    DatabasePool::GoldenDB(read_pool),
                    DatabasePool::GoldenDB(write_pool),
                )
            }
            DatabaseType::Dameng => {
                let read_pool = Arc::new(sqlx::AnyPool::connect(read_url).await?);
                let write_pool = Arc::new(sqlx::AnyPool::connect(write_url).await?);
                (
                    DatabasePool::Dameng(read_pool),
                    DatabasePool::Dameng(write_pool),
                )
            }
        };

        Ok(Self {
            read_pool,
            write_pool,
            db_type,
        })
    }

    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    pub fn get_pool(&self, read_only: bool) -> &DatabasePool {
        if read_only {
            &self.read_pool
        } else {
            &self.write_pool
        }
    }

    pub async fn execute_postgres_read<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::PgConnection) -> Result<T, AppError>,
    {
        match &self.read_pool {
            DatabasePool::PostgreSQL(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut conn)
            }
            _ => Err(AppError::internal_error(
                "Not a PostgreSQL connection",
                None,
            )),
        }
    }

    pub async fn execute_goldendb_read<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::MySqlConnection) -> Result<T, AppError>,
    {
        match &self.read_pool {
            DatabasePool::GoldenDB(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut conn)
            }
            _ => Err(AppError::internal_error("Not a GoldenDB connection", None)),
        }
    }

    pub async fn execute_dameng_read<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::AnyConnection) -> Result<T, AppError>,
    {
        match &self.read_pool {
            DatabasePool::Dameng(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut conn)
            }
            _ => Err(AppError::internal_error("Not a Dameng connection", None)),
        }
    }

    pub async fn execute_postgres_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::PgConnection) -> Result<T, AppError>,
    {
        match &self.write_pool {
            DatabasePool::PostgreSQL(pool) => {
                let mut conn = pool.acquire().await?;
                let mut tx = conn.begin().await?;
                match f(&mut tx) {
                    Ok(t) => {
                        tx.commit().await?;
                        Ok(t)
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        Err(e)
                    }
                }
            }
            _ => Err(AppError::internal_error(
                "Not a PostgreSQL connection",
                None,
            )),
        }
    }

    pub async fn execute_goldendb_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::MySqlConnection) -> Result<T, AppError>,
    {
        match &self.write_pool {
            DatabasePool::GoldenDB(pool) => {
                let mut conn = pool.acquire().await?;
                let mut tx = conn.begin().await?;
                match f(&mut tx) {
                    Ok(t) => {
                        tx.commit().await?;
                        Ok(t)
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        Err(e)
                    }
                }
            }
            _ => Err(AppError::internal_error("Not a GoldenDB connection", None)),
        }
    }

    pub async fn execute_dameng_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut sqlx::AnyConnection) -> Result<T, AppError>,
    {
        match &self.write_pool {
            DatabasePool::Dameng(pool) => {
                let mut conn = pool.acquire().await?;
                let mut tx = conn.begin().await?;
                match f(&mut tx) {
                    Ok(t) => {
                        tx.commit().await?;
                        Ok(t)
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        Err(e)
                    }
                }
            }
            _ => Err(AppError::internal_error("Not a Dameng connection", None)),
        }
    }

    pub async fn query<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&DatabasePool) -> Result<T, AppError>,
    {
        f(&self.read_pool)
    }

    pub async fn execute<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&DatabasePool) -> Result<T, AppError>,
    {
        f(&self.write_pool)
    }
}

/// SQL 语法适配器
pub struct SqlAdapter;

impl SqlAdapter {
    pub fn adapt_sql(sql: &str, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::PostgreSQL => sql.to_string(),
            DatabaseType::GoldenDB => Self::adapt_to_goldendb(sql),
            DatabaseType::Dameng => Self::adapt_to_dameng(sql),
        }
    }

    fn adapt_to_goldendb(sql: &str) -> String {
        let mut result = sql.to_string();

        result = regex::Regex::new(r"LIMIT\s+(\d+)\s+OFFSET\s+(\d+)")
            .expect("valid regex")
            .replace_all(&result, "LIMIT $2, $1")
            .to_string();

        result = result.replace("NOW()", "CURRENT_TIMESTAMP");
        result = result.replace("SERIAL PRIMARY KEY", "INT AUTO_INCREMENT PRIMARY KEY");
        result = result.replace("BIGSERIAL PRIMARY KEY", "BIGINT AUTO_INCREMENT PRIMARY KEY");
        result = result.replace("TEXT", "VARCHAR(65535)");
        result = result.replace("UUID", "VARCHAR(36)");
        result = result.replace("jsonb", "JSON");
        result = result.replace("ARRAY[", "JSON_ARRAY(");
        result = regex::Regex::new(r"(\w+)\s*@>\s*(\w+)")
            .expect("valid regex")
            .replace_all(&result, "JSON_CONTAINS($1, $2)")
            .to_string();
        result = regex::Regex::new(r"(\w+)\s*::\s*text")
            .expect("valid regex")
            .replace_all(&result, "CAST($1 AS CHAR)")
            .to_string();
        result = result.replace("pg_sleep(", "SLEEP(");

        result
    }

    fn adapt_to_dameng(sql: &str) -> String {
        let mut result = sql.to_string();

        result = regex::Regex::new(r"LIMIT\s+(\d+)\s+OFFSET\s+(\d+)")
            .expect("valid regex")
            .replace_all(&result, "TOP $1 START WITH $2")
            .to_string();

        result = result.replace("LIMIT", "TOP");
        result = result.replace("NOW()", "SYSDATE");
        result = result.replace("SERIAL PRIMARY KEY", "INT IDENTITY PRIMARY KEY");
        result = result.replace("BIGSERIAL PRIMARY KEY", "BIGINT IDENTITY PRIMARY KEY");
        result = result.replace("TEXT", "CLOB");
        result = result.replace("UUID", "VARCHAR(36)");
        result = result.replace("jsonb", "TEXT");
        result = regex::Regex::new(r"(\w+)\s*@>\s*(\w+)")
            .expect("valid regex")
            .replace_all(&result, "CONTAINS($1, $2)")
            .to_string();
        result = regex::Regex::new(r"(\w+)\s*::\s*text")
            .expect("valid regex")
            .replace_all(&result, "CAST($1 AS VARCHAR)")
            .to_string();
        result = result.replace("pg_sleep(", "SLEEP(");
        result = result.replace("TRUE", "1");
        result = result.replace("FALSE", "0");

        result
    }

    pub fn adapt_params(params: &[&str], db_type: DatabaseType) -> Vec<String> {
        match db_type {
            DatabaseType::Dameng => params
                .iter()
                .map(|p| p.replace("'", "''").replace("\\", "\\\\"))
                .collect(),
            _ => params.iter().map(|p| p.replace("'", "''")).collect(),
        }
    }
}

/// 全局读写分离池（懒加载）
static READ_WRITE_POOL: once_cell::sync::Lazy<Arc<ReadWritePool>> =
    once_cell::sync::Lazy::new(|| {
        let db_type = DatabaseType::from_env();

        let (read_url, write_url) = match db_type {
            DatabaseType::PostgreSQL => (
                std::env::var("DATABASE_READ_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("postgres://user:pass@localhost:5432/carptms".to_string())
                }),
                std::env::var("DATABASE_WRITE_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("postgres://user:pass@localhost:5432/carptms".to_string())
                }),
            ),
            DatabaseType::GoldenDB => (
                std::env::var("DATABASE_READ_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("mysql://user:pass@localhost:3306/carptms".to_string())
                }),
                std::env::var("DATABASE_WRITE_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("mysql://user:pass@localhost:3306/carptms".to_string())
                }),
            ),
            DatabaseType::Dameng => (
                std::env::var("DATABASE_READ_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("dm://user:pass@localhost:5236/carptms".to_string())
                }),
                std::env::var("DATABASE_WRITE_URL").unwrap_or_else(|_| {
                    std::env::var("DATABASE_URL")
                        .unwrap_or("dm://user:pass@localhost:5236/carptms".to_string())
                }),
            ),
        };

        Arc::new({
            let urls = (read_url, write_url);
            std::thread::spawn(move || {
                tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime for DB pool")
                    .block_on(async {
                        ReadWritePool::new(&urls.0, &urls.1, db_type)
                            .await
                            .expect("Failed to initialize read-write database pool")
                    })
            })
            .join()
            .expect("DB pool initialization thread panicked")
        })
    });

pub fn get_read_write_pool() -> &'static Arc<ReadWritePool> {
    &READ_WRITE_POOL
}

pub fn get_db_type() -> DatabaseType {
    DatabaseType::from_env()
}

#[macro_export]
macro_rules! db_query {
    ($pool:expr, $sql:expr) => {{
        let db_type = $pool.db_type();
        let adapted_sql = SqlAdapter::adapt_sql($sql, db_type);

        match db_type {
            DatabaseType::PostgreSQL => {
                $pool
                    .execute_postgres_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_all(conn).await?)
                    })
                    .await
            }
            DatabaseType::GoldenDB => {
                $pool
                    .execute_goldendb_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_all(conn).await?)
                    })
                    .await
            }
            DatabaseType::Dameng => {
                $pool
                    .execute_dameng_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_all(conn).await?)
                    })
                    .await
            }
        }
    }};
}

#[macro_export]
macro_rules! db_execute {
    ($pool:expr, $sql:expr) => {{
        let db_type = $pool.db_type();
        let adapted_sql = SqlAdapter::adapt_sql($sql, db_type);

        match db_type {
            DatabaseType::PostgreSQL => {
                $pool
                    .execute_postgres_write(|conn| {
                        Ok(sqlx::query(&adapted_sql).execute(conn).await?)
                    })
                    .await
            }
            DatabaseType::GoldenDB => {
                $pool
                    .execute_goldendb_write(|conn| {
                        Ok(sqlx::query(&adapted_sql).execute(conn).await?)
                    })
                    .await
            }
            DatabaseType::Dameng => {
                $pool
                    .execute_dameng_write(|conn| {
                        Ok(sqlx::query(&adapted_sql).execute(conn).await?)
                    })
                    .await
            }
        }
    }};
}

#[macro_export]
macro_rules! db_query_one {
    ($pool:expr, $sql:expr) => {{
        let db_type = $pool.db_type();
        let adapted_sql = SqlAdapter::adapt_sql($sql, db_type);

        match db_type {
            DatabaseType::PostgreSQL => {
                $pool
                    .execute_postgres_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_one(conn).await?)
                    })
                    .await
            }
            DatabaseType::GoldenDB => {
                $pool
                    .execute_goldendb_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_one(conn).await?)
                    })
                    .await
            }
            DatabaseType::Dameng => {
                $pool
                    .execute_dameng_read(|conn| {
                        Ok(sqlx::query(&adapted_sql).fetch_one(conn).await?)
                    })
                    .await
            }
        }
    }};
}
