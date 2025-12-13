use sqlx::{sqlite::SqlitePool, FromRow};
use std::fs;

#[derive(Debug, FromRow)]
pub struct Resource {
    pub uid:     String,
    pub content: String,
}

const POST: &str = "INSERT INTO resources (uid, content) VALUES (?, ?)";
const GET: &str = "SELECT uid, content FROM resources WHERE uid = ?";
const _DELETE: &str = "DELETE FROM resources WHERE uid = ?";

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(db_path: &std::path::PathBuf) -> sqlx::Result<Self> {
	if !db_path.exists() {
	    fs::File::create(db_path)?;
	    println!("[INFO] Created file database at {}", db_path.display());
	}

	let db_url = format!("sqlite:{}", db_path.display());
	let pool = SqlitePool::connect(&db_url).await?;
	let init_sql = include_str!("../schema/init.sql");
	
	sqlx::query(init_sql).execute(&pool).await?;

	Ok(Database{ pool })
    }

    pub async fn post_resource(&self, uid: &str, content: &str) -> sqlx::Result<()> {
	sqlx::query(POST)
	    .bind(uid).bind(content)
	    .execute(&self.pool)
	    .await?;

	Ok(())
    }

    pub async fn get_resource(&self, uid: &str) -> sqlx::Result<Option<Resource>> {
	let resource = sqlx::query_as::<_, Resource>(GET)
	    .bind(uid)
	    .fetch_optional(&self.pool)
	    .await?;

	Ok(resource)
    }
}
