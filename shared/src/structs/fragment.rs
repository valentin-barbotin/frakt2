use serde_json::json;
use super::prelude::*;

// Each variant have his own struct and implementation. (Can't impl on a unique variant)
#[derive(Serialize, Deserialize, Debug)]
pub enum Fragment {
    Task(FragmentTask),
    Result(FragmentResult),
    Request(FragmentRequest),
}

impl Fragment {
    /*
        Serialize a fragment to a JSON string.
     */
    pub fn serialize(&self) -> String {
        match self {
            Fragment::Task(task) => self.to_json("FragmentTask", task),
            Fragment::Result(result) => self.to_json("FragmentResult", result),
            Fragment::Request(request) => self.to_json("FragmentRequest", request),
        }
    }

    /*
        Serialize a fragment to a JSON string.
        Fragment type is provided as a key and fragment as a value.
     */
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

#[macro_export]
macro_rules! fragment_from_json_value {
    ($fragment_type:ty, $variant:ident, $json_value:ident) => {
        match serde_json::from_value::<$fragment_type>($json_value) {
            Ok(v) => Some(Fragment::$variant(v)),
            Err(e) => {
                error!("Failed to get fragment: {}", e);
                None
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::extract_message;
    use rstest::*;

    #[rstest]
    fn test_fragment_task_from_json() {
        let fragment = Fragment::Task(FragmentTask::default());
        let json = fragment.serialize();

        let extracted_fragment = extract_message(&json).unwrap();

        match extracted_fragment {
            Fragment::Task(task) => {
                assert_eq!(task.id.offset, 0);
                assert_eq!(task.id.count, 0);
                // no need to check data
            }
            _ => panic!("Wrong fragment type"),
        }
    }
    
    #[rstest]
    fn test_fragment_request_from_json() {
        let fragment = Fragment::Request(FragmentRequest::new("test", 500));
        let json = fragment.serialize();

        let extracted_fragment = extract_message(&json).unwrap();

        match extracted_fragment {
            Fragment::Request(request) => {
                assert_eq!(request.worker_name, "test");
                assert_eq!(request.maximal_work_load, 500);
            }
            _ => panic!("Wrong fragment type"),
        }
    }
    
    #[rstest]
    fn test_fragment_result_from_json() {

        let fragment = Fragment::Result(FragmentResult::default());
        let json = fragment.serialize();

        let extracted_fragment = extract_message(&json).unwrap();

        match extracted_fragment {
            Fragment::Result(result) => {
                assert_eq!(result.id.offset, 0);
                assert_eq!(result.id.count, 0);
                // no need to check data
            }
            _ => panic!("Wrong fragment type"),
        }
    }

    #[rstest]
    fn test_serialize_task_fragment() {
        let fragment = Fragment::Task(FragmentTask::default());
        let json = fragment.serialize();

        assert_eq!(
            json,
            r#"{"FragmentTask":{"fractal":{"Mandelbrot":{}},"id":{"count":0,"offset":0},"max_iteration":0,"range":{"max":{"x":0.0,"y":0.0},"min":{"x":0.0,"y":0.0}},"resolution":{"nx":0,"ny":0}}}"#
        );
    }

    #[rstest]
    fn test_serialize_request_fragment() {
        let fragment = Fragment::Request(FragmentRequest::new("test", 500));
        let json = fragment.serialize();

        assert_eq!( 
            json,
            r#"{"FragmentRequest":{"maximal_work_load":500,"worker_name":"test"}}"#
        );
    }

    #[rstest]
    fn test_serialize_result_fragment() {
        let fragment = Fragment::Result(FragmentResult::default());
        let json = fragment.serialize();

        assert_eq!(
            json,
            r#"{"FragmentResult":{"id":{"count":0,"offset":0},"pixels":{"count":0,"offset":0},"range":{"max":{"x":0.0,"y":0.0},"min":{"x":0.0,"y":0.0}},"resolution":{"nx":0,"ny":0}}}"#
        );
    }
}

