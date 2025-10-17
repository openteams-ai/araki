use clap::Parser;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the environment
    #[arg(help="Name of the environment")]
    name: String,
}

pub fn execute(args: Args) {
    // Get the akari envs dir
    let Some(akari_envs_dir) = common::get_default_akari_envs_dir()
    else {
        println!("error!");
        return
    };

    
}
