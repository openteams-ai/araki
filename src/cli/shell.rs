use clap::Parser;
use directories::UserDirs;
use std::fs::{Permissions, exists, remove_file, set_permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{
    fmt::{self},
    fs::{self, File},
    path::Path,
    str::FromStr,
};

use crate::cli::common::get_default_araki_bin_dir;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: ShellSubcommand,
}

#[derive(Parser, Debug, Default)]
pub struct ShellArg {
    shell: Option<String>,
}

#[derive(Parser, Debug)]
pub enum ShellSubcommand {
    /// Initialize the shell configuration (by editing ~/.bashrc etc)
    Init(ShellArg),

    /// Generate the environment changes to be evaluated by the shell so that the araki shims
    /// take precedence over other system binaries
    Generate(ShellArg),
}

enum Shell {
    Bash,
    Zsh,
    Unknown(String),
}

#[derive(Debug)]
struct ParseShellError;

impl FromStr for Shell {
    type Err = ParseShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.to_lowercase().as_str() {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            shell => Shell::Unknown(shell.to_string()),
        };

        Ok(result)
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shell_name = match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Unknown(name) => name,
        };
        write!(f, "{}", shell_name)
    }
}

impl Shell {
    /// Update the bash/zsh config file to fiddle the PATH so that araki shims are executed instead
    /// of `pip`, `uv`, `pixi`, `conda`, etc.
    ///
    /// * `path`: Path to the config file to edit
    fn update_posix_config(&self, path: &Path) -> Result<(), String> {
        let contents = fs::read_to_string(path)
            .map_err(|_| "Could not open {path} to read existing shell config.")?;

        let araki_posix_config = format!(
            "# Araki configuration\neval $(araki shell generate {})\n",
            self
        );
        if !contents.contains(&araki_posix_config) {
            let mut rcfile = File::options()
                .append(true)
                .open(path)
                .map_err(|_| "Could not open {path} to modify shell config.")?;

            write!(&mut rcfile, "{}", &araki_posix_config)
                .map_err(|_| "Unable to write araki shell config to {path}")?;
        }

        let dir = get_default_araki_bin_dir()?;
        for tool in ["pip", "uv", "pixi", "conda"] {
            let shim_path = dir.join(tool);
            if exists(&shim_path).is_ok_and(|val| val) {
                remove_file(&shim_path)
                    .map_err(|_| "Unable to remove existing shim at {shim_path:?}")?;
            }
            {
                let mut shim = File::create(&shim_path)
                    .map_err(|_| format!("Unable to write shim to {shim_path:?}"))?;

                let _ = writeln!(&mut shim, "#!/bin/{self}");
                let _ = writeln!(&mut shim, "araki shim -- {tool} $@");

                let perms = shim
                    .metadata()
                    .map_err(|_| "Could not get metadata for {shim_path:?}")?
                    .permissions();

                // Set the shim to be executable
                set_permissions(&shim_path, Permissions::from_mode(perms.mode() | 0o700))
                    .map_err(|_| "Unable to set permissions on {shim_path:?}")?;
            }
        }
        Ok(())
    }

    /// Print the environment variable changes to be evaluated by the shell
    fn print_env(&self) -> Result<(), String> {
        match self {
            Shell::Bash | Shell::Zsh => {
                print!(
                    "PATH={}:$PATH",
                    get_default_araki_bin_dir()?.to_string_lossy()
                );
                Ok(())
            }
            Shell::Unknown(shell) => {
                Err(format!("Cannot generate environment updates for {shell}"))
            }
        }
    }

    /// Get the shell configuration file
    fn get_shell_config(&self) -> Result<PathBuf, String> {
        let home_dir = UserDirs::new()
            .ok_or("Could not get the home directory for the system.")?
            .home_dir()
            .to_path_buf();
        match self {
            Shell::Bash => Ok(home_dir.join(".bashrc")),
            Shell::Zsh => Ok(home_dir.join(".zshrc")),
            Shell::Unknown(shell) => Err(format!(
                "Cannot get shell configuration for unknown shell: {shell}"
            )),
        }
    }

    /// Update the shell configuration so that araki shims take precedence
    fn update_shell_config(&self) -> Result<(), String> {
        let config = self.get_shell_config()?;
        match self {
            Shell::Bash => self.update_posix_config(&config),
            Shell::Zsh => self.update_posix_config(&config),
            Shell::Unknown(shell) => Err(format!(
                "{shell} is not one of the supported shells: {}",
                Shell::supported_shells().join(", ")
            )),
        }
    }

    /// A list of supported shells, for printing in the error message above. Maybe not needed if
    /// there's a way to iterate over enum types?
    fn supported_shells() -> Vec<&'static str> {
        vec!["bash", "zsh"]
    }

    /// See https://stackoverflow.com/a/78241067/8100451 for reference
    /// Detect the shell type and return a corresponding Shell instance.
    fn detect() -> Self {
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::default().with_processes(sysinfo::ProcessRefreshKind::default()),
        );
        let my_pid = sysinfo::get_current_pid().expect("unable to get PID of the current process");
        let parent_pid = system
            .process(my_pid)
            .expect("no self process?")
            .parent()
            .expect("unable to get parent process");
        let parent_process = system
            .process(parent_pid)
            .expect("unable to get parent process");
        let parent_name = parent_process.name().to_string_lossy();
        Self::from_str(&parent_name).unwrap_or(Shell::Unknown("".to_string()))
    }
}

pub fn execute(args: Args) {
    match args.subcommand {
        ShellSubcommand::Init(shell_arg) => {
            let shell: Shell = shell_arg
                .shell
                .map(|name| {
                    name.parse::<Shell>().unwrap_or_else(|_| {
                        unreachable!("All string shell names are valid Shell types")
                    })
                })
                .unwrap_or_else(Shell::detect);

            match shell.update_shell_config() {
                Ok(_) => println!("{shell} configuration updated."),
                Err(error) => eprintln!("{error}"),
            };
        }
        ShellSubcommand::Generate(shell_arg) => {
            let shell: Shell = shell_arg
                .shell
                .map(|name| {
                    name.parse::<Shell>().unwrap_or_else(|_| {
                        unreachable!("All string shell names are valid Shell types")
                    })
                })
                .unwrap_or_else(Shell::detect);

            let _ = shell.print_env().map_err(|error| eprintln!("{error}"));
        }
    }
}
