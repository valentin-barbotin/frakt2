use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Range {
    pub min: Point,
    pub max: Point,
}

impl Range {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }
}
