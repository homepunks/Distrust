use data::Database;
use std::sync::Arc;

pub mod data;
pub mod errors;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}
