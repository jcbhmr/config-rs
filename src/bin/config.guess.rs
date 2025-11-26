use defer::defer;
use glob::Pattern;
use memoize::memoize;
use regex_lite::Regex;
use which::which;
use std::{
    cell::OnceCell,
    collections::VecDeque,
    env::{args_os, current_exe, join_paths, set_var, split_paths, var_os},
    error::Error,
    fs::{File, exists, metadata, remove_dir_all, write},
    iter::once,
    process::{Command, Stdio, exit},
};
use tempdir::TempDir;

const TIMESTAMP: &str = "2025-07-10";

fn uname_sysname() -> Result<String, std::io::Error> {
    Ok(format!("sysname"))
}

fn uname_release() -> Result<String, std::io::Error> {
    Ok(format!("release"))
}

fn uname_version() -> Result<String, std::io::Error> {
    Ok(format!("version"))
}

fn uname_machine() -> Result<String, std::io::Error> {
    Ok(format!("machine"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let current_exe = current_exe()?;
    let me = current_exe
        .file_name()
        .or_else(|| format!("no file name for {:?}", &current_exe))?;

    let usage = format!(
        r#"Usage: {} [OPTION]

Output the configuration name of the system '{}' is run on.

Options:
  -h, --help         print this help, then exit
  -t, --time-stamp   print date of last modification, then exit
  -v, --version      print version number, then exit

Report bugs and patches to https://github.com/jcbhmr/config-rs."#,
        current_exe.display(),
        me.display()
    );

    let version = format!(
        r#"GNU config.guess ({})

Originally written by Per Bothner.
Copyright 1992-2025 Free Software Foundation, Inc.
Copyright 2025 Jacob Hummer

This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE."#,
        TIMESTAMP
    );

    let help = format!(r#"Try '{} --help' for more information."#, me.display());

    // Parse command line
    let mut args_os = args_os().collect::<VecDeque<_>>();
    while let Some(arg_os) = args_os.pop_front() {
        match arg_os.to_str() {
            Some("--time-stamp") | Some("-t") | Some(s) if Pattern::new("--time*")?.matches(s) => {
                println!("{}", TIMESTAMP);
                return Ok(());
            }
            Some("--version") | Some("-v") => {
                println!("{}", version);
                return Ok(());
            }
            Some("--help") | Some("-h") | Some(s) if Pattern::new("--h*")?.matches(s) => {
                println!("{}", usage);
                return Ok(());
            }
            Some("--") => {
                // Stop option processing
                args_os.pop_front();
                break;
            }
            Some("-") => break, // Use stdin as input
            Some(s) if Pattern::new("-*")?.matches(s) => {
                Err(format!("{}: invalid option {}\n{}", me.display(), s, help))?;
            }
            _ => break,
        };
    }

    if args_os.len() != 0 {
        Err(format!("{}: too many arguments\n{}", me.display(), help))?;
    }

    // Just in case it came from the environment
    // SAFETY: This program is single-threaded. setenv() is safe in single-threaded contexts.
    unsafe { set_var("GUESS", "") };

    // This is needed to find uname on a Pyramid OSx when run in the BSD universe.
    // (ghazi@noc.rutgers.edu 1994-08-24)
    if metadata("/.attbin/uname") {
        let new_path = join_paths(split_paths(var_os("PATH")).chain(once("/.attbin")))?;
        unsafe { set_var("PATH", new_path) };
    }

    let uname_machine = uname_machine().unwrap_or_else(|_| "unknown".into());
    let uname_release = uname_release().unwrap_or_else(|_| "unknown".into());
    let uname_system = uname_sysname().unwrap_or_else(|_| "unknown".into());
    let uname_version = uname_version().unwrap_or_else(|_| "unknown".into());

    let mut libc = "unknown".to_owned();
    if Pattern::new("Linux|GNU|GNU/*")?.matches(uname_system) {
        libc = "unknown".to_owned();
        write(
            "dummy.c",
            r#"
#if defined(__ANDROID__)
LIBC=android
#else
#include <features.h>
#if defined(__UCLIBC__)
LIBC=uclibc
#elif defined(__dietlibc__)
LIBC=dietlibc
#elif defined(__GLIBC__)
LIBC=gnu
#elif defined(__LLVM_LIBC__)
LIBC=llvm
#else
#include <stdarg.h>
/* First heuristic to detect musl libc.  */
#ifdef __DEFINED_va_list
LIBC=musl
#endif
#endif
#endif
"#,
        );
        let output = Command::new("/bin/sh")
            .args(["-E", "dummy.c"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()?;
        if output.status.success() {
            let output = output.stdout;
        }

        // Second heuristic to detect musl libc.
        if libc == "unknown" {
            if let Ok(ldd_path) = which("ldd") {
                let output = Command::new(ldd_path)
                    .arg("--version")
                    .stdin(Stdio::null())
                    .stderr(Stdio::null())
                    .output()?;
                if output.status.success() {
                    if Regex::new("(?m)^musl")?.is_match(output.stdout) {
                        libc = "musl".to_owned();
                    }
                }
            }
        }

        // If the system lacks a compiler, then just pick glibc. We could probably try harder.
        if libc == "unknown" {
            libc = "gnu".to_owned();
        }
    }

    // Note: Order is significant - the branches are not exclusive.
    match (uname_machine.as_ref(), uname_system.as_ref(), uname_release.as_ref(), uname_version.as_ref()) {
        (_, "NetBSD", _, _) => {
            // NetBSD (nbsd) targets should (where applicable) match one or more of the tuples:
            // "*-*-netbsdelf*", "*-*netbsdaout*", "*-*netbsdecoff*" and "*-*-netbsd*". For targets that recently
            // switched to ELF, "*-*-netbsd" would select the old object file format. This provides both forward compatibility
            // and a consistent mechanism for selecting the object file format.
            //
            // Note: NetBSD doesn't particularly care about the vendor portion of the name. We always set it to "unknown".
            
            let uname_machine_arch = {
                let output = Command::new("uname").arg("-p").stdin(Stdio::null()).stderr(Stdio::null()).output()?;
                if output.status.success() {
                    String::from_utf8(output.stdout)?.trim_end().to_owned()
                } else {
                    let sysctl_path = which("/sbin/sysctl").or_else(|_| which("/usr/sbin/sysctl"))?;
                    let output = Command::new(sysctl_path).args(["-n", "hw.machine_arch"]).stdin(Stdio::null()).stderr(Stdio::null()).output()?;
                    if output.status.success() {
                        String::from_utf8(output.stdout)?.trim_end().to_owned()
                    } else {
                        "unknown".to_owned()
                    }
                }
            };
            match uname_machine_arch.as_ref() {
                "aarch64eb" => {}
            };
        }
    }

    Ok(())
}
