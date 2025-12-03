use crate::backends::Backend;
use clap::Parser;
use std::process::exit;

use crate::backends;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: AuthSubcommand,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub enum AuthSubcommand {
    // Log in to the backend
    Login,
}

pub async fn execute(args: Args) {
    match args.subcommand {
        AuthSubcommand::Login => {
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
    }
}
