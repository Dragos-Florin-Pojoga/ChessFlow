mod chess_lib;
use chess_lib::*;

use std::io::{self, BufRead};

// WIP

fn main() {
    let stdin = io::stdin();
    for maybe_line in stdin.lock().lines() {
        match maybe_line {
            Ok(line) => {
                let cmd = parse_command(&line);
                //println!("{:?}", cmd);
                match cmd {
                    Ok(cmd) => {
                        if let UciCommand::Quit = cmd {
                            break;
                        }
                        println!("{}", execute_command(cmd));
                    },
                    Err(_) => {}
                }
            }
            Err(err) => {
                eprintln!("error reading stdin: {}", err);
                break;
            }
        }
    }
}