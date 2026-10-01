//! `uilab-propose --doc <file> --target <path> "<utterance>"`: one instruction, one admitted
//! patch, printed as YAML with the turns and the cost it took.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use uilab_agent::{Credential, Proposer, ProposerConfig};
use uilab_doc::{Document, NodePath};

#[derive(Debug, Parser)]
#[command(
    name = "uilab-propose",
    about = "Propose one patch of an ess-ui/1 document from one spoken instruction"
)]
struct Args {
    /// The `ess-ui/1` document.
    #[arg(long)]
    doc: PathBuf,
    /// The node the instruction is about, for example `page:loans`.
    #[arg(long)]
    target: NodePath,
    /// Exact model identifier.
    #[arg(long)]
    model: Option<String>,
    /// Origin plus API prefix of an anthropic-messages endpoint.
    #[arg(long)]
    base_url: Option<String>,
    /// Read an API key from this environment variable instead of the subscription token file.
    #[arg(long)]
    api_key_env: Option<String>,
    /// A harness rate card (JSON); without one the cost is reported as unknown.
    #[arg(long)]
    prices: Option<PathBuf>,
    /// Turns one attempt may take.
    #[arg(long)]
    max_turns: Option<u32>,
    /// The instruction, as speech-to-text wrote it.
    utterance: String,
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("uilab-propose: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let text = std::fs::read_to_string(&args.doc)
        .map_err(|error| format!("reading `{}`: {error}", args.doc.display()))?;
    let doc = Document::from_yaml(&text).map_err(|error| {
        format!(
            "`{}` is not an ess-ui/1 document: {error}",
            args.doc.display()
        )
    })?;

    let mut config = ProposerConfig::default();
    if let Some(model) = args.model {
        config.model = model;
    }
    if let Some(base_url) = args.base_url {
        config.base_url = base_url;
    }
    if let Some(name) = args.api_key_env {
        config.credential = Credential::ApiKeyEnv { name };
    }
    if let Some(max_turns) = args.max_turns {
        config.max_turns = max_turns;
    }
    let prices = match args.prices {
        None => None,
        Some(path) => {
            let text = std::fs::read_to_string(&path)
                .map_err(|error| format!("reading the rate card `{}`: {error}", path.display()))?;
            Some(
                harness_loop::RateCard::parse(&text)
                    .map_err(|error| format!("the rate card `{}`: {error}", path.display()))?,
            )
        }
    };

    let mut proposer = Proposer::new(config)
        .map_err(|error| error.to_string())?
        .with_prices(prices);
    let proposal = proposer
        .propose(&doc, &args.target, &args.utterance)
        .map_err(|error| error.to_string())?;
    let yaml = serde_yaml::to_string(&proposal).map_err(|error| error.to_string())?;
    print!("{yaml}");
    Ok(())
}
