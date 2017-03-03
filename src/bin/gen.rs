extern crate grammar_gen;

use std::env::args_os;
use std::fs::File;
use std::io::{Read, Write, stderr, stdin};
use std::path::PathBuf;
use std::process;

fn main() {
    // Get command-line argument
    let mut path: Option<PathBuf> = None;
    {
        let mut args = args_os();
        if args.next().is_some() {
            if let Some(arg) = args.next() {
                path = Some(arg.into());
                if args.next().is_some() {
                    writeln!(stderr(), "Too many arguments");
                    process::exit(1);
                }
            }
        }
    }

    // Read file to memory
    let mut buf = Vec::new();
    let read = match path {
        None => stdin().read_to_end(&mut buf),
        Some(path) => File::open(path).and_then(|mut f| f.read_to_end(&mut buf)),
    };
    if let Err(error) = read {
        writeln!(stderr(), "Error reading grammar: {}", error);
        process::exit(1);
    }

    // Parse grammar
    let grammar = grammar_gen::parse(&String::from_utf8_lossy(&buf));
    if let Err(error) = grammar {
        writeln!(stderr(), "Error parsing grammar: {}", error);
        process::exit(1);
    }
    let grammar = grammar.unwrap();

    // Generate text
    println!("{}", grammar.generate_random(&grammar_gen::Term("_".into())));
}
