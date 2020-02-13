use std::error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Command {
    /// Generate HDL templates
    Generate {},
}

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let _ = Opt::from_args();
    Ok(())
}
