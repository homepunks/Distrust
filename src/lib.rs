use std::sync::Arc;
use data::Database;

pub mod data;
pub mod errors;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}
