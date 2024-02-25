use serde::{Deserialize, Serialize};

use crate::structs::fragment_result::FragmentResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingData {
    pub result: FragmentResult,
    pub worker: String,
    pub iterations: Vec<f64>,
}
