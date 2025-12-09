use clap::Parser;
use std::env;
use std::process::{Command, exit};

use crate::common::get_araki_bin_dir;
use crate::settings::Settings;

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// This command is intended to be called by shim files, and lets araki know that the user
    /// has tried to use a shimmed environment management tool. Not intended to be called by the
    /// user directly.
    #[arg(num_args = 1..)]
    args: Vec<String>,
}

/// Given a PATH environment variable, this function strips out the araki bin directory.
///
/// * `path`: Colon-separated PATH environment variable to be stripped
fn strip_araki_shim_path(path: &str, shim_path: &str) -> Result<String, String> {
    Ok(path
        .split(":")
        .filter(|item| *item != shim_path)
        .collect::<Vec<&str>>()
        .join(":"))
}

pub fn execute(args: Args, _settings: Settings) {
    let value = env::var("ARAKI_OVERRIDE_SHIM").unwrap_or("false".to_string());
    if value.trim() == "1" {
        // Run the requested command using the modified PATH
        let current_path = env::var_os("PATH");

        let shim_path = get_araki_bin_dir().unwrap_or_else(|err| {
            eprintln!("Unable to get the araki bin directory: {err}");
            exit(1);
        });

        // Extract the tool to be run `pip`, etc... from the argument list passed to araki.
        // Call the tool and pass in any trailing arguments using the stripped PATH env variable.
        if let [tool, arguments @ ..] = args.args.as_slice() {
            let mut command = Command::new(tool);
            if let Some(path) = current_path {
                match strip_araki_shim_path(&path.to_string_lossy(), &shim_path.to_string_lossy()) {
                    Ok(new_env) => command.env("PATH", new_env),
                    Err(err) => {
                        eprintln!("Unable to strip the araki shim path from PATH:\n{err}");
                        return;
                    }
                };
            }
            let _ = command
                .args(arguments)
                .status()
                .map_err(|err| eprintln!("Error running command {tool}: {err}"));
        } else {
            eprintln!("Could not destructure the command you passed.");
        }
    } else {
        let passed_args = args.args.join(" ");
        eprintln!(
            "Unable to run {passed_args}; use araki for environment management. \
            Set ARAKI_OVERRIDE_SHIM=1 to run the command anyway."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_araki_shim_path() {
        let path = "/home/foo/.local/bin:/home/foo/.araki/bin:/usr/bin:/usr/local/bin";
        let shim_dir = "/home/foo/.araki/bin";

        // Check that the dir is in the path
        assert!(path.contains(shim_dir));
        let result =
            strip_araki_shim_path(path, shim_dir).expect("should be able to strip the path");
        assert!(!result.contains(shim_dir));
    }
}
