use super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct FragmentRequest {
    pub worker_name: String,
    pub maximal_work_load: u32,
}

impl FragmentRequest {
    pub fn new(worker_name: &str, maximal_work_load: u32) -> Self {
        Self {
            worker_name: worker_name.to_string(),
            maximal_work_load,
        }
    }
}
