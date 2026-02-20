use std::env;

fn main() {
    //Initialize the logging
    let env = Env::default();
    env_logger::init_from_env(env);
    let args: Vec<String> = env::args().collect();
    if let Err(e) = plc_driver::compile(&args) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
