use axum::extract::State;
use jnickg_imaging::dyn_matrix::DynMatrix;
use mongodb::Database;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuntimeData {
    pub somethings: HashSet<u32>,
    pub matrices: HashMap<String, DynMatrix<f64>>,
    pub image_counter: usize,
    pub db: Option<Database>,
    pub bg_tasks: HashMap<Uuid, Arc<JoinHandle<()>>>,
}

impl RuntimeData {
    pub fn new() -> Self {
        RuntimeData {
            somethings: HashSet::<u32>::new(),
            matrices: HashMap::<String, DynMatrix<f64>>::new(),
            image_counter: 0,
            db: None,
            bg_tasks: HashMap::<Uuid, Arc<JoinHandle<()>>>::new(),
        }
    }
}

pub type AppState = State<Arc<RwLock<RuntimeData>>>;
