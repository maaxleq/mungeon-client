mod model;
mod net;
mod runner;
mod session;

use std::env;
use std::error;

static ERROR_ARGUMENT_PARSE: &str = "Could not parse argument";
static ERROR_NO_URL: &str = "No URL was specified";

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut url = String::new();

    for arg in args.iter().skip(1) {
        if arg.starts_with("url=") {
            url = arg[4..arg.len()].to_string();
        } else {
            panic!("{} {}", ERROR_ARGUMENT_PARSE, arg);
        }
    }

    if url == "" {
        panic!("{}", ERROR_NO_URL);
    }

    runner::Runner::try_new(session::Session::new(url))?.run()?;

    Ok(())
}
