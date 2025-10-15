use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the environment
    #[arg(help="Name of the environment")]
    name: String,
}

pub fn execute(args: Args) {
     println!("(not) activating env: {}", args.name);
}