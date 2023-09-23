mod parser;

use crate::parser::Subcommand;
use clap::Parser;

fn main() {
    let Subcommand::Doc(args) = Subcommand::parse();
    println!("{:#?}", args);
}
