use clap::Parser;
use std::fs;

use crate::cli::common;

#[derive(Parser, Debug)]
#[clap(arg_required_else_help = true)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: EnvsSubcommand,
}

#[derive(Parser, Debug)]
pub enum EnvsSubcommand {
    // List environments
    #[clap(visible_alias = "ls", alias = "l")]
    List(ListArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct ListArgs { }

pub fn execute(args: Args) {
    match args.subcommand {
        EnvsSubcommand::List(_args) => {
            // Get the araki envs dir
            let Some(araki_envs_dir) = common::get_default_araki_envs_dir()
            else {
                println!("error!");
                return
            };

            println!("Available envs:");
            let paths = fs::read_dir(araki_envs_dir).unwrap();
            for path in paths {
                let env_path = path.unwrap().path();
                let env_name = env_path.file_name().expect("unable to get filename");
                println!("* {}", env_name.display())
            }
        }
    };
}
