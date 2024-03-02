#[macro_export]
macro_rules! acquire_server {
    ($server:expr) => {{
        $server.lock().expect("Failed to acquire server lock")
    }};
}
