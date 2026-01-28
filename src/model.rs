use sqlx::{sqlite::SqlitePool, Row};
use directories::ProjectDirs;
use std::fs;
use chrono;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub text: String,
    pub completed: bool,
    pub created_at: i64,
}

#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let proj_dirs = ProjectDirs::from("com", "pomimi", "pomimi").unwrap();
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            let _ = fs::create_dir_all(data_dir);
        }
        let db_path = data_dir.join("pomimi.db");
        // Ensure the file exists so sqlite can open it
        if !db_path.exists() {
            fs::File::create(&db_path).expect("Failed to create db file");
        }

        let db_url = format!("sqlite://{}", db_path.to_string_lossy());

        let pool = SqlitePool::connect(&db_url).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )"
        ).execute(&pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )"
        ).execute(&pool).await?;

        sqlx::query(
             "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                duration_seconds INTEGER NOT NULL
             )"
        ).execute(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT id, text, completed, created_at FROM tasks ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }

    pub async fn add_task(&self, text: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO tasks (text, completed, created_at) VALUES (?, 0, ?)"
        )
        .bind(text)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_task(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Preferences
    pub async fn get_preference(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query("SELECT value FROM preferences WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("value")))
    }

    pub async fn set_preference(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO preferences (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Sessions
    pub async fn add_session(&self, duration_seconds: i64) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO sessions (start_time, duration_seconds) VALUES (?, ?)"
        )
        .bind(now)
        .bind(duration_seconds)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_today_focus_time(&self) -> Result<i64, sqlx::Error> {
        // Start of today
        let today = chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().timestamp();

        // Use query_scalar to get a single value (Option<i64> because SUM can be NULL)
        let result: Option<i64> = sqlx::query_scalar(
            "SELECT SUM(duration_seconds) FROM sessions WHERE start_time >= ?"
        )
        .bind(today)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(None); // fetch_one might fail if table empty? No, aggregation always returns a row.

        // Actually fetch_one returns Result<Row>. query_scalar returns Result<T>.
        // If no rows match, SUM returns NULL, which maps to Option::None.

        Ok(result.unwrap_or(0))
    }
}
