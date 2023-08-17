use std::io;

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteQueryResult, Error, FromRow, Pool, Sqlite, SqlitePool};

#[derive(Debug)]
pub struct Db {
    pub pool: Pool<Sqlite>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct DbVariables {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub size: i64,
    pub offset: i64,
    pub process_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ClientVariables {
    pub name: String,
    pub description: Option<String>,
    pub size: i64,
    pub offset: i64,
    pub process_id: i64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct DbProcesses {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientProcesses {
    pub name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DbSettings {
    pub sizing: i64,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self, io::Error> {
        let pool = SqlitePool::connect(path)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "db file was not found"))?;
        sqlx::query!(
            "
            CREATE TABLE IF NOT EXISTS variables (
                                                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                                                name TEXT NOT NULL, 
                                                description TEXT,
                                                size INTEGER NOT NULL,
                                                offset INTEGER NOT NULL,
                                                process_id INTEGER NOT NULL,
                                                FOREIGN KEY (process_id) REFERENCES processes (id) ON DELETE CASCADE);
            CREATE TABLE IF NOT EXISTS settings (
                                                sizing INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS processes (
                                                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                                                name TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS recepies (
                                                name TEXT NOT NULL,
                                                description TEXT);
            CREATE TABLE IF NOT EXISTS recepie_components (
                                                name TEXT NOT NULL,
                                                description TEXT);

            INSERT INTO settings (sizing) VALUES (4);
            "
        )
        .execute(&pool)
        .await
        .unwrap();

        let db = Db { pool };
        return Ok(db);
    }

    pub async fn get_settings(&self) -> Result<DbSettings, sqlx::Error> {
        let settings = sqlx::query_as!(DbSettings, "SELECT sizing FROM settings")
            .fetch_one(&self.pool)
            .await;
        return settings;
    }

    pub async fn save_settings(&self, settings: DbSettings) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            DbSettings,
            "INSERT INTO settings (sizing) VALUES (?)",
            settings.sizing
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_variable(&self, variable: ClientVariables) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO variables (name, description, size, offset, process_id) VALUES (?, ?, ?, ?, ?)",
            variable.name,
            variable.description,
            variable.size,
            variable.offset,
            variable.process_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_variables_by_id(&self, id: i32) -> Result<DbVariables, Error> {
        sqlx::query_as!(DbVariables, "SELECT * FROM variables WHERE id=?", id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn get_variables(
        &self,
        skip: Option<u32>,
        take: Option<u32>,
    ) -> Result<Vec<DbVariables>, Error> {
        let skip = skip.unwrap_or(0);
        let take = take.unwrap_or(50);

        sqlx::query_as!(
            DbVariables,
            "SELECT * FROM variables LIMIT ? OFFSET ?",
            take,
            skip
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn remove_variable_by_id(&self, id: i32) -> Result<SqliteQueryResult, Error> {
        sqlx::query!("DELETE FROM variables WHERE id=?", id)
            .execute(&self.pool)
            .await
    }

    pub async fn delete_process(&self, id: i32) -> Result<SqliteQueryResult, Error> {
        sqlx::query!("DELETE FROM processes WHERE id=?;", id)
            .execute(&self.pool)
            .await
    }

    pub async fn save_process(&self, process: ClientProcesses) -> Result<SqliteQueryResult, Error> {
        sqlx::query!("INSERT INTO processes (name) VALUES (?)", process.name)
            .execute(&self.pool)
            .await
    }
}
