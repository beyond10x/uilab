//! `uilab-plan --doc <file> --target <path> [--max-steps 8] "<goal>"`: one goal, an ordered plan
//! of instructions for `uilab-propose`, printed as YAML with the turns and the cost it took.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use uilab_agent::{Credential, Proposer, ProposerConfig};
use uilab_doc::{Document, Fixtures, NodePath};

#[derive(Debug, Parser)]
#[command(
    name = "uilab-plan",
    about = "Plan a goal for an ess-ui/1 document as ordered instructions, one patch each"
)]
struct Args {
    /// The `ess-ui/1` document.
    #[arg(long)]
    doc: PathBuf,
    /// The node the goal is about, for example `page:members`.
    #[arg(long)]
    target: NodePath,
    /// The most steps the plan may have.
    #[arg(long, default_value_t = 8)]
    max_steps: usize,
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
    /// The goal, in the operator's words.
    goal: String,
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("uilab-plan: {message}");
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
    let fixtures = Fixtures::load(&doc, args.doc.parent().unwrap_or(Path::new(".")))
        .map_err(|error| error.to_string())?;
    let fields: Vec<(String, Vec<String>)> = fixtures.fields().into_iter().collect();

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
    let plan = proposer
        .plan_goal_with(&doc, &args.target, &args.goal, args.max_steps, &fields)
        .map_err(|error| error.to_string())?;
    let yaml = serde_yaml::to_string(&plan).map_err(|error| error.to_string())?;
    print!("{yaml}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_defaults_to_eight_and_the_goal_is_positional() {
        let args = Args::try_parse_from([
            "uilab-plan",
            "--doc",
            "library.ui.yaml",
            "--target",
            "page:members",
            "build out the member area",
        ])
        .unwrap();
        assert_eq!(args.max_steps, 8);
        assert_eq!(args.goal, "build out the member area");
        assert_eq!(args.target.to_string(), "page:members");
    }
}
