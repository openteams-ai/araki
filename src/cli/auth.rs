use crate::backends::Backend;
use clap::Parser;
use std::process::exit;

use crate::backends;

#[derive(Parser, Debug, Default)]
pub struct Args {}

pub async fn execute(_args: Args) {
    let backend = backends::get_current_backend().unwrap_or_else(|err| {
        eprintln!("Unable to get the current backend: {err}");
        exit(1);
    });
    backend.login().await.unwrap_or_else(|err| {
        eprintln!("Unable to login: {err}");
        exit(1);
    });

    println!("Successfully authenticated.");
}
