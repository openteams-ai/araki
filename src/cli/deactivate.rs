use clap::Parser;
use std::env;

#[derive(Parser, Debug, Default)]
pub struct Args {}

// This approach is not really going to work. For example, it does not
// account for changes to  PATH, or CONDA_ env vars!
pub fn execute(_args: Args) {
    // Unset all variables prefixed with PIXI_
    let prefix = "PIXI_";

    for (key, _value) in env::vars() {
        if key.starts_with(prefix) {
            println!("unset {}", key)
        }
    }
}
