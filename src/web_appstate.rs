use axum::extract::State;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RuntimeData {
    pub somethings: HashSet<u32>,
}

impl RuntimeData {
    pub fn new() -> Self {
        RuntimeData {
            somethings: HashSet::<u32>::new(),
        }
    }
}

pub type AppState = State<Arc<RwLock<RuntimeData>>>;
