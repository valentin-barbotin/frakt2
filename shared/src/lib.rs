pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod colors;
pub mod network;
pub mod structs;

#[macro_export]
macro_rules! loop_sleep {
    () => {
        std::thread::sleep(std::time::Duration::from_millis(*LOOP_SLEEP_DURATION));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
