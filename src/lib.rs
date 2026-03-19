use data::Database;
use std::{path, sync::Arc};

pub mod data;
pub mod errors;
pub mod routes;

pub const MAX_SIZE: usize = 1024 * 1024 * 10;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub cache_dir: path::PathBuf,
}
