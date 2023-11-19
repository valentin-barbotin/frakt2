use super::super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Julia {
    pub c: Complex,
    pub divergence_threshold_square: f64,
}

