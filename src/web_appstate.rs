use axum::extract::State;
use jnickg_imaging::dyn_matrix::DynMatrix;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RuntimeData {
    pub somethings: HashSet<u32>,
    pub matrices: HashMap<String, DynMatrix<f64>>,
}

impl RuntimeData {
    pub fn new() -> Self {
        RuntimeData {
            somethings: HashSet::<u32>::new(),
            matrices: HashMap::<String, DynMatrix<f64>>::new(),
        }
    }
}

pub type AppState = State<Arc<RwLock<RuntimeData>>>;
