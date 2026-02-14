use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use sqlx::{sqlite::SqlitePool, FromRow};

#[derive(Debug, FromRow)]
pub struct Paste {
    pub id:     String,
    pub content: Vec<u8>,
    pub content_type: String,
    pub size: i64,
    pub created_at: i64,
    pub view_count: i64,
}

#[derive(Debug)]
pub struct NewPaste {
    pub content: Vec<u8>,
    pub content_type: String,
}

const INSERT: &str = r#"
    INSERT INTO pastes (id, content, content_type, size, created_at)
    VALUES (?, ?, ?, ?, ?)
"#;

const SELECT: &str = r#"
    SELECT id, content, content_type, size, created_at, view_count
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

	let db_url = format!("sqlite:{}", db_path.display());
	let pool = SqlitePool::connect(&db_url).await?;
	let init_sql = include_str!("../schema/init.sql");
	
	sqlx::query(init_sql).execute(&pool).await?;

	Ok(Database{ pool })
    }
    
    pub async fn create_paste(&self, id: &str, paste: NewPaste) -> sqlx::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let size = paste.content.len() as i64;

        sqlx::query(INSERT)
            .bind(id)
            .bind(&paste.content)
            .bind(&paste.content_type)
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
