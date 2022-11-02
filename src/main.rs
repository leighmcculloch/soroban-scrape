mod cmd_invokes;
mod cmd_wasms;
mod horizon;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    version,
    about,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
enum RootCmd {
    Invokes(cmd_invokes::Cmd),
    Wasms(cmd_wasms::Cmd),
}

fn main() {
    match RootCmd::parse() {
        RootCmd::Invokes(cmd) => cmd.run(),
        RootCmd::Wasms(cmd) => cmd.run(),
    };
}
