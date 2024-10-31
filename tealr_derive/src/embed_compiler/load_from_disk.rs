fn run_find_command(is_local: bool) -> String {
    let mut command = Command::new("luarocks");
    command.arg("which").arg("tl.tl");
    if is_local {
        command.arg("--local");
    }
    let command = command.output().unwrap_or_else(|e| {
        panic!(
            "Could not execute `luarocks which tl.tl` to discover location of tl.tl. Error:\n{}",
            e
        )
    });
    let stdout = String::from_utf8(command.stdout).unwrap();
    if !command.status.success() {
        if !is_local {
            return run_find_command(true);
        } else {
            panic!(
                "`luarocks which tl.tl` did not exit successfully. Status code : {}\n StdErr :\n{}]\n\nstdOut:\n{}",
                command.status,
                String::from_utf8(command.stderr).unwrap(),
                stdout
            );
        }
    }
    stdout
        .lines()
        .next()
        .expect("Did not get the expected output from luarocks")
        .to_string()
}

use std::{fs::read_to_string, process::Command};

pub(crate) fn discover_tl_tl() -> String {
    run_find_command(true)
}
pub(crate) fn get_local_teal(path: String) -> String {
    let build_dir = tempfile::tempdir().expect("Could not get a temporary directory to build teal");
    let compiler = Command::new("tl")
        .current_dir(build_dir.path())
        .args(["gen", "-o", "output.lua", "--skip-compat53"])
        .arg(path)
        .spawn()
        .map_err(|e| {
            (match e.kind() {
                std::io::ErrorKind::NotFound => "Could not compile teal. Command `tl` not found.",
                std::io::ErrorKind::PermissionDenied => {
                    "Permission denied when running the teal compiler."
                }
                _ => "Error while running teal. Is it available as `tl` in the path?",
            }, e)
        });


    let mut compiler = match compiler {
        Ok(v) => v,
        Err((msg, e)) => {
            if let Err(error) = build_dir.close() {
                eprint!("Could not close temporary directory : {}", error);
            }
            panic!(
                "Could not compile teal:{msg}\nRaw error:{}\n{}",
                e.raw_os_error()
                    .map(|e| format!("Kind:{e}"))
                    .unwrap_or_default(),
                e.kind()
            )
        },
    };

    if !compiler
        .wait()
        .expect("Could not wait for compiler")
        .success()
    {
        if let Err(e) = build_dir.close() {
            eprint!("Could not close temporary directory : {}", e);
        }
        panic!(
            "Could not compile teal without compatibility library. Is `tl` available in the path?"
        )
    }
    read_to_string(build_dir.path().join("output.lua")).expect("Could not read compiled compiler")
}
