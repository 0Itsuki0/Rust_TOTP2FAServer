use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::user::User;

/// A dummy database
#[derive(Clone, Default)]
pub struct AppState {
    pub db: Arc<Mutex<Vec<User>>>,
}
