use shared::{
    utils::random_string,
    structs::prelude::*,
};

pub struct FractalParams {
    pub fractal_type: String,
    pub resolution: Resolution,
    pub max_iteration: u16,
}

#[derive(Clone)]
pub struct Task {
    pub fragment: FragmentTask,
    pub data: Vec<u8>,
    pub id: String
}


pub fn create_fractal_tasks(params: FractalParams, split: u16) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();

    let min_range = Point::new(0.0, 0.0);
    let max_range = Point::new(4.0, 4.0);

    for i in 0..split {
        for j in 0..split {
            let range = create_range(min_range, max_range, split, i, j);

            let fractal_type = params.fractal_type.as_str();
            let fractal = match fractal_type {
                "mandelbrot" => {
                    let mandelbrot = Mandelbrot {};
                
                    FractalDescriptor::Mandelbrot(mandelbrot)
                },
                _ => {
                    return Err(format!("Unknown fractal type: {fractal_type}"));
                }
            };


            let task_id = random_string(8);
            let mut data = Vec::new();
            data.extend_from_slice(task_id.as_bytes());
            let id = U8Data::new(0, data.len() as u32);

            let resolution = Resolution::new(params.resolution.nx / split, params.resolution.ny / split);

            let fragment = FragmentTask::new(id, fractal, params.max_iteration, resolution, range);

            let task = Task {
                fragment,
                data,
                id: task_id
            };

            tasks.push(task);
        }
    }

    Ok(tasks)
}

fn create_range(min_range: Point, max_range: Point, split: u16, i: u16, j: u16) -> Range {
    Range::new(
        Point::new(
            min_range.x + (max_range.x - min_range.x) / split as f64 * i as f64,
            min_range.y + (max_range.y - min_range.y) / split as f64 * j as f64,
        ),
        Point::new(
            min_range.x + (max_range.x - min_range.x) / split as f64 * (i + 1) as f64,
            min_range.y + (max_range.y - min_range.y) / split as f64 * (j + 1) as f64,
        ),
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task() {
        let params = FractalParams {
            fractal_type: "mandelbrot".to_string(),
            resolution: Resolution {
                nx: 1000,
                ny: 1000
            },
            max_iteration: 500
        };

        let split = 2;
        let res = create_fractal_tasks(params, 2);
        assert!(res.is_ok());

        let tasks = res.unwrap();

        assert_eq!(tasks.len(), split * split);

        let mut ids: Vec<&str> = Vec::with_capacity(4);

        tasks.iter().for_each(|task| {
            assert_eq!(task.data.len(), 8);

            let fragment = &task.fragment;

            assert!(!ids.contains(&task.id.as_str()));
            ids.push(&task.id);

            assert_eq!(fragment.resolution.nx, 500);
            assert_eq!(fragment.resolution.ny, 500);

            assert_eq!(fragment.max_iteration, 500);

            // TODO: check Ranges
        });

        
    }

    #[test]
    fn test_create_range() {
        let min_range = Point::new(0.0, 0.0);
        let max_range = Point::new(4.0, 4.0);

        let res = create_range(min_range, max_range, 4, 0, 0);

        let expected_result = Range::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
        );

        assert_eq!(res.min.x, expected_result.min.x);
        assert_eq!(res.min.y, expected_result.min.y);
        assert_eq!(res.max.x, expected_result.max.x);
        assert_eq!(res.max.y, expected_result.max.y);
    }
}
