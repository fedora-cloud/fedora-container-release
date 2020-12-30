mod prerelease;

use prerelease::prepare_containerfiles;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
struct CliOptions {
    #[structopt(short, long)]
    release: String,
}

fn main() -> std::io::Result<()> {
    let cliopt = CliOptions::from_args();
    let result = prepare_containerfiles(cliopt.release);
    return result
}
