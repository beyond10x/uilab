// generated from uilab v1
// model digest d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13
// contract digest 9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0
// do not edit: regenerate with `ess synthesize`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let _ = options;
eprintln!("unmet startup obligations:\ncommand behaviour: uilab.session.AcceptProposal\ncommand behaviour: uilab.session.OpenDocument\ncommand behaviour: uilab.session.ProposePatch\ncommand behaviour: uilab.session.SelectNode\ncommand behaviour: uilab.session.UndoProposal");
std::process::ExitCode::FAILURE
}
