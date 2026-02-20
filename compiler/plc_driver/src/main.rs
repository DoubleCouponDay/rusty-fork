use std::env;

use env_logger::Env;

fn main() {
    //Initialize the logging
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let args: Vec<String> = env::args().collect();
    if let Err(e) = plc_driver::compile(&args) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
