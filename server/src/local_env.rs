use std::{env, time::Duration};

use lazy_static::lazy_static;

/*
    Static variables that are initialized with the environment variables.
    lazy_static != static, (static => compile time, lazy_static => runtime)
*/
lazy_static! {
    pub static ref RUST_ENV: String = env::var("RUST_ENV").unwrap_or("info".to_string());
    pub static ref PORT: u16 = env::var("PORT")
        .unwrap_or_else(|_e| { panic!("{}", var_not_defined("PORT")) })
        .parse()
        .unwrap_or_else(|e| { panic!("PORT is not a valid number: {}", e) });
    pub static ref HOST: String = env::var("HOST")
        .unwrap_or_else(|_e| { panic!("{}", var_not_defined("HOST")) })
        .parse()
        .unwrap_or_else(|e| { panic!("PORT is not a valid string: {}", e) });
    pub static ref SERVER_HOST: String = env::var("SERVER_HOST")
    .unwrap_or_else(|_e| { panic!("{}", var_not_defined("SERVER_HOST")) })
    .parse()
    .unwrap_or_else(|e| { panic!("SERVER_PORT is not a valid string: {}", e) });

    pub static ref LOOP_SLEEP_DURATION: u64 = match RUST_ENV.as_str() {
        "debug" => 500,
        "trace" => 500,
        _ => 0,
    };
}

/*
    Use this function to get a nice error message when a variable is not defined.
*/
fn var_not_defined(var: &str) -> String {
    format!("{} is not defined in the environment", var)
}

/*
    Check if all variables are defined. If not, panic.
*/
pub fn check_vars() {
    lazy_static::initialize(&RUST_ENV); // don't panic if RUST_ENV is not defined
    lazy_static::initialize(&PORT);
    lazy_static::initialize(&HOST);
    lazy_static::initialize(&LOOP_SLEEP_DURATION);
}
