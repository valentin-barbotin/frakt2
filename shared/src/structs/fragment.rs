use serde_json::json;

use super::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Fragment {
    Task(FragmentTask),
    Result(FragmentResult),
    Request(FragmentRequest),
}

impl Fragment {
    pub fn serialize(&self) -> String {
        match self {
            Fragment::Task(task) => self.to_json("FragmentTask", task),
            Fragment::Result(result) => self.to_json("FragmentResult", result),
            Fragment::Request(request) => self.to_json("FragmentRequest", request),
        }
    }

    fn to_json<T>(&self, fragment_type: &str, fragment: &T) -> String
    where
        T: Serialize,
    {
        json!({
            fragment_type: fragment,
        })
        .to_string()
    }
}
