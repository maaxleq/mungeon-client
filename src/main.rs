mod model;
mod net;
mod session;

use std::env;

static ERROR_ARGUMENT_PARSE: &str = "Could not parse argument";
static ERROR_NO_URL: &str = "No URL was specified";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut url = String::new();

    for arg in args.iter().skip(1) {
        if arg.starts_with("url=") {
            url = arg[4..arg.len()].to_string();
        }
        else {
            panic!("{} {}", ERROR_ARGUMENT_PARSE, arg);
        }
    }

    if url == "" {
        panic!("{}", ERROR_NO_URL);
    }
    
    let mut session = session::Session::new(url);
    session.connect();
    session.look_room();
    session.look_entity(String::from("33"));
    println!("{:?}", session);
}
