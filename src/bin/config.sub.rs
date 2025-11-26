use std::{env::args, error::Error, path::Path, process::exit};

use regex_lite::Regex;

fn main() {
    let me = Path::new(args().next().unwrap()).file_name().unwrap().to_str().unwrap().to_owned();
    
    let usage = format!(r#"Usage: {} [OPTION] CPU-MFR-OPSYS or ALIAS

Canonicalize a configuration name.

Options:
  -h, --help         print this help, then exit
  -t, --time-stamp   print date of last modification, then exit
  -v, --version      print version number, then exit

Report bugs and patches to <https://github.com/jcbhmr/config-rs>."#, args().next().unwrap());

    let version = format!(r#"GNU config.sub ({})

Copyright 1992-2025 Free Software Foundation, Inc.

This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE."#, "TODO: Add timestamp");

    let help = format!("\nTry '{} --help' for more information.", me.as_ref());

    // Parse the command line
    let mut args = args().skip(1).collect::<Vec<_>>();
    while args.len() > 0 {
        match args[0].as_ref() {
            x if x == "--time-stamp" | Regex::new(r"^--time.*?$").unwrap().is_match(x) | x == "-t" => {
                println!("{}", "TODO: Add timestamp");
                exit(0);
            },
            "--version" | "-v" => {
                println!("{}", version.as_ref());
                exit(0);
            },
            x if x == "--help" | Regex::new(r"^--h.*?$").unwrap().is_match(x) | x == "-h" => {
                println!("{}", usage.as_ref());
                exit(0);
            },
            "--" => {
                // Stop option processing
                args.remove(0);
                break;
            },
            "-" => {
                // Use stdin as input
                break;
            },
            x if Regex::new(r"^-.*?$").unwrap().is_match(x) => {
                eprintln!("{}: invalid option {}{}", me.as_ref(), args[0].as_ref(), help.as_ref());
                exit(1);
            },
            x if Regex::new("^.*?local.*?$").unwrap().is_match(x) => {
                // First pass through any local machine types
                println!("{}", args[0].as_ref());
                exit(0);
            },
            _ => {
                break;
            }
        };
    }

    match args.len() {
        0 => {
            eprintln!("{}: missing argument{}", me.as_ref(), help.as_ref());
            exit(1);
        },
        1 => {},
        _ => {
            eprintln!("{}: too many arguments{}", me.as_ref(), help.as_ref());
            exit(1);
        },
    }

    let cpu_mfr_opsys_or_alias = args.remove(0);
}