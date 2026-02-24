use sqlx::{FromRow, sqlite::SqlitePool};
use std::{fs, env};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, FromRow)]
pub struct Paste {
    pub id: String,
    pub content_type: String,
    pub size: i64,
    pub created_at: i64,
    pub view_count: i64,
}

const INSERT: &str = r#"
    INSERT INTO pastes (id, content_type, size, created_at)
    VALUES (?, ?, ?, ?)
"#;

const SELECT: &str = r#"
    SELECT id, content_type, size, created_at, view_count
    FROM pastes WHERE id = ?
"#;

const INCREMENT_VIEWS: &str = r#"
    UPDATE pastes SET view_count = view_count + 1 WHERE id = ?
"#;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(db_path: &std::path::PathBuf) -> sqlx::Result<Self> {
        if !db_path.exists() {
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::File::create(db_path)?;
            println!("[INFO] Created file database at {}", db_path.display());
        }

        let cache_dir_path = env::current_dir()?.join("cache");
        let cache = Path::new(&cache_dir_path);
        if !cache.is_dir() {
            fs::create_dir(cache)?;
            println!("[INFO] Created cache directory at {}", cache.display());
        }

        let db_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;
        let init_sql = include_str!("../schema/init.sql");

        sqlx::query(init_sql).execute(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn create_paste(&self, id: &str, content_type: &str, size: i64) -> sqlx::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(INSERT)
            .bind(id)
            .bind(content_type)
            .bind(size)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_paste(&self, id: &str) -> sqlx::Result<Option<Paste>> {
        let paste = sqlx::query_as::<_, Paste>(SELECT)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if paste.is_some() {
            let _ = sqlx::query(INCREMENT_VIEWS)
                .bind(id)
                .execute(&self.pool)
                .await;
        }

        Ok(paste)
    }
}
