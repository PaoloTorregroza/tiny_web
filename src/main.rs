use std::env;

use file_loader::read_html_file;
use parser::Parser;

mod dom;
mod parser;
mod file_loader;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let html = read_html_file(file_path);
    let nodes = Parser::parse(html);

    println!("{:?}", nodes);
}
