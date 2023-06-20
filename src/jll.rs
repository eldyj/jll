mod interpreter;
use std::env;

fn main() -> () {
	let argv: Vec<String> = env::args().collect();
	interpreter::run_file_init(argv[1].as_str());
}
