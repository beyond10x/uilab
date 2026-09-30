//! One spoken instruction at one node of a `ui-spec/1` document, turned into one patch that
//! [`uilab_doc::admit`] accepts.
//!
//! A [`Proposer`] runs one agent run through the b10x harness loop (`harness-loop`). The run has
//! no tools and an approver that refuses everything; its only way to finish is the loop's own
//! `answer` tool, published with [`uilab_doc::patch_schema`] as its input schema, which the loop
//! validates locally before it counts as an answer. The answer is then read as a
//! [`uilab_doc::Patch`] and admitted. A refusal is fed back to the model once, on the same
//! conversation; a second refusal is [`ProposeError::Refused`].

use std::path::PathBuf;
use std::sync::Arc;

use harness_loop::{
    AgentLoop, Budget, DenyAll, LoopConfig, LoopError, NullLoopSink, OutputSchema, RateCard,
    RunLedger,
};
use harness_wire::{
    Bearer, BearerSource, ModelPort, ToolCall, ToolOutcome, ToolPort, ToolSpec, WireError,
};
use serde::Serialize;
use uilab_doc::{Document, NodeContext, NodePath, Patch, PathError, Refusal};

/// The subscription route's base URL: the Messages API.
pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
/// The model a default proposer asks.
pub const DEFAULT_MODEL: &str = "claude-sonnet-5-5";
/// The subscription token file, relative to the home directory.
pub const DEFAULT_OAUTH_FILE: &str = ".claude/.credentials.json";
/// Where the access token sits in that file.
pub const DEFAULT_OAUTH_POINTER: &str = "/claudeAiOauth/accessToken";
/// The context window a default proposer declares, in tokens.
pub const DEFAULT_CONTEXT_WINDOW: u64 = 200_000;
/// Turns one attempt may take: an answer the schema refuses costs a turn inside the loop.
pub const DEFAULT_MAX_TURNS: u32 = 4;
/// Attempts per proposal: the first, and one retry with the refusal fed back.
pub const MAX_ATTEMPTS: u32 = 2;

/// Where the proposer reaches a model and how it authenticates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposerConfig {
    /// Origin plus API prefix of an anthropic-messages endpoint.
    pub base_url: String,
    /// Exact model identifier.
    pub model: String,
    /// Where the credential is read from, on every turn.
    pub credential: Credential,
    /// The model's context window, in tokens.
    pub context_window: u64,
    /// Turns one attempt may take.
    pub max_turns: u32,
}

/// Where the credential is read from. Never a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// A subscription token in a JSON file, at a JSON pointer. A leading `~/` is the home
    /// directory.
    OauthFile { path: PathBuf, pointer: String },
    /// An API key in the named environment variable.
    ApiKeyEnv { name: String },
}

impl Default for ProposerConfig {
    /// The subscription route: the Messages API with the token Claude Code keeps.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            model: DEFAULT_MODEL.to_owned(),
            credential: Credential::OauthFile {
                path: PathBuf::from("~").join(DEFAULT_OAUTH_FILE),
                pointer: DEFAULT_OAUTH_POINTER.to_owned(),
            },
            context_window: DEFAULT_CONTEXT_WINDOW,
            max_turns: DEFAULT_MAX_TURNS,
        }
    }
}

/// One admitted patch and what it took.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Proposal {
    /// The patch; [`uilab_doc::admit`] accepted it against the document it was proposed for.
    pub patch: Patch,
    /// Turns across every attempt.
    pub turns: u64,
    /// What every attempt cost, in millionths of a US dollar; `None` when no rate card priced it.
    pub cost_micro_usd: Option<u64>,
    /// Attempts made: 1, or 2 after one refusal.
    pub attempts: u32,
}

/// Why no patch came back.
#[derive(Debug, thiserror::Error)]
pub enum ProposeError {
    /// The proposer could not be built as configured.
    #[error("the proposer cannot be built: {0}")]
    Config(String),
    /// The target path names no node of the document.
    #[error(transparent)]
    Target(#[from] PathError),
    /// The agent run failed before it ended: the wire, the budget, the configuration.
    #[error("the agent run failed: {0}")]
    Run(#[from] LoopError),
    /// The agent run ended without an answer: a budget bound, or it answered in prose.
    #[error("the agent run ended without a patch: {0}")]
    Stopped(String),
    /// Every attempt was refused; this is the last refusal.
    #[error("the patch was refused on every attempt; last refusal {check}: {message}")]
    Refused { check: String, message: String },
    /// The words were not a request to change the UI (thanks, a greeting, a question, noise);
    /// nothing is proposed, and this says why or answers the question.
    #[error("declined: {0}")]
    Declined(String),
}

/// Proposes patches. One model connection, reused across proposals.
pub struct Proposer {
    model: String,
    max_turns: u32,
    context_window: u64,
    prices: Option<RateCard>,
    port: Box<dyn ModelPort + Send>,
}

impl Proposer {
    /// A proposer over the anthropic-messages endpoint the config names.
    ///
    /// # Errors
    ///
    /// [`ProposeError::Config`] for an endpoint that could not serve a request or a client that
    /// could not be built. The credential is read on each turn, not here.
    pub fn new(config: ProposerConfig) -> Result<Self, ProposeError> {
        let endpoint = harness_messages::Endpoint::new(
            config.base_url.clone(),
            config.model.clone(),
            config.context_window,
        )
        .map_err(|error| ProposeError::Config(error.to_string()))?;
        let client = harness_messages::MessagesClient::new(endpoint, bearer(&config.credential))
            .map_err(|error| ProposeError::Config(error.to_string()))?;
        Ok(Self::with_port(&config, Box::new(client)))
    }

    /// A proposer over a model port the caller built: another wire, or a scripted one in a test.
    /// `config.base_url` and `config.credential` are not read.
    pub fn with_port(config: &ProposerConfig, port: Box<dyn ModelPort + Send>) -> Self {
        Self {
            model: config.model.clone(),
            max_turns: config.max_turns,
            context_window: config.context_window,
            prices: None,
            port,
        }
    }

    /// Prices every run at this rate card, so [`Proposal::cost_micro_usd`] is reported.
    #[must_use]
    pub fn with_prices(mut self, prices: Option<RateCard>) -> Self {
        self.prices = prices;
        self
    }

    /// Blocking. One instruction at one node → one patch that [`uilab_doc::admit`] accepts.
    ///
    /// # Errors
    ///
    /// [`ProposeError::Target`] for a path that names no node, [`ProposeError::Run`] and
    /// [`ProposeError::Stopped`] for a run that gave no answer, and [`ProposeError::Refused`] when
    /// [`MAX_ATTEMPTS`] answers were all refused.
    pub fn propose(
        &mut self,
        doc: &Document,
        target: &NodePath,
        utterance: &str,
    ) -> Result<Proposal, ProposeError> {
        self.propose_with(doc, target, utterance, &[])
    }

    /// [`propose`](Self::propose), telling the model which fields each view's rows carry, so
    /// columns, `from` and form fields name real fields rather than guessed ones.
    ///
    /// # Errors
    ///
    /// As [`propose`](Self::propose).
    pub fn propose_with(
        &mut self,
        doc: &Document,
        target: &NodePath,
        utterance: &str,
        fields: &[(String, Vec<String>)],
    ) -> Result<Proposal, ProposeError> {
        let context = uilab_doc::node_context(doc, target)?;
        let schema = uilab_doc::patch_schema(doc, target)?;
        let answer =
            OutputSchema::new(schema).map_err(|error| ProposeError::Config(error.to_string()))?;
        let config = LoopConfig::new(self.model.clone(), INSTRUCTIONS)
            .with_budget(Budget {
                max_turns: Some(u64::from(self.max_turns)),
                ..Budget::default()
            })
            .with_context_window(Some(self.context_window))
            .with_prices(self.prices.clone())
            .with_output_schema(Some(answer));

        let mut items = Vec::new();
        let mut turns = 0;
        let mut costs = Vec::new();
        let mut input = request(doc, &context, utterance, fields);
        let mut refusal = None;
        for attempt in 1..=MAX_ATTEMPTS {
            let mut tools = NoTools;
            let mut approvals = DenyAll;
            let mut spend = RunLedger::default();
            let outcome = AgentLoop::new(
                self.port.as_mut(),
                &mut tools,
                &mut approvals,
                config.clone(),
            )
            .run_in(&mut items, &mut spend, input, &mut NullLoopSink);
            turns += spend.turns;
            costs.push(spend.cost_micro_usd);
            let outcome = outcome?;
            let structured = match (outcome.stop.is_completed(), outcome.structured) {
                (true, Some(structured)) => structured,
                (_, _) => return Err(ProposeError::Stopped(describe_stop(&outcome.stop))),
            };
            if structured["op"] == "decline" {
                let reason = structured["reason"]
                    .as_str()
                    .unwrap_or("not an instruction to change the UI");
                return Err(ProposeError::Declined(reason.to_owned()));
            }
            let refused = match serde_json::from_value::<Patch>(structured) {
                Ok(patch) => match uilab_doc::admit(doc, &patch) {
                    Ok(_) => {
                        return Ok(Proposal {
                            patch,
                            turns,
                            cost_micro_usd: total(&costs),
                            attempts: attempt,
                        });
                    }
                    Err(refused) => refused,
                },
                Err(error) => Refusal {
                    check: "patch_shape".to_owned(),
                    message: error.to_string(),
                },
            };
            input = retry(&refused);
            refusal = Some(refused);
        }
        let refused = refusal.expect("every attempt that did not return was refused");
        Err(ProposeError::Refused {
            check: refused.check,
            message: refused.message,
        })
    }
}

/// The sum of every attempt's cost, or `None` when any attempt was unpriced: a partial sum would
/// read as the whole.
fn total(costs: &[Option<u64>]) -> Option<u64> {
    costs
        .iter()
        .try_fold(0_u64, |sum, cost| cost.map(|cost| sum.saturating_add(cost)))
}

fn describe_stop(stop: &harness_loop::LoopStop) -> String {
    serde_json::to_string(stop).unwrap_or_else(|_| format!("{stop:?}"))
}

/// The credential source for a config. Read on every turn, so a renewed token is picked up.
fn bearer(credential: &Credential) -> Arc<dyn BearerSource> {
    match credential {
        Credential::OauthFile { path, pointer } => Arc::new(
            harness_credential::SubscriptionToken::new(harness_credential::NamedSource::file(
                expand_home(path),
            ))
            .at_pointer(pointer.clone()),
        ),
        Credential::ApiKeyEnv { name } => Arc::new(EnvApiKey(name.clone())),
    }
}

fn expand_home(path: &std::path::Path) -> PathBuf {
    match (path.strip_prefix("~"), std::env::var_os("HOME")) {
        (Ok(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => path.to_path_buf(),
    }
}

/// An API key in a named environment variable, presented as a key rather than a subscription
/// token.
struct EnvApiKey(String);

impl BearerSource for EnvApiKey {
    fn bearer(&self) -> Result<Bearer, WireError> {
        match std::env::var(&self.0) {
            Ok(value) if !value.trim().is_empty() => Ok(Bearer::new(value.trim())),
            Ok(_) => Err(WireError::unauthorized(format!(
                "the environment variable `{}` is empty",
                self.0
            ))),
            Err(_) => Err(WireError::unauthorized(format!(
                "the environment variable `{}` is not set",
                self.0
            ))),
        }
    }
}

/// The run's tools: none. The loop's own `answer` is the only thing the model can call.
struct NoTools;

impl ToolPort for NoTools {
    fn specs(&self) -> &[ToolSpec] {
        &[]
    }

    fn call(&mut self, _call: &ToolCall) -> ToolOutcome {
        ToolOutcome::failed("this run has no tools; finish by calling `answer`")
    }
}

/// What the model is told about `ui-spec/1` and its job.
pub const INSTRUCTIONS: &str = "\
You edit one user-interface document in the `ui-spec/1` format by proposing exactly one patch. \
The operator speaks an instruction while pointing at one node of the document; you answer by \
calling the `answer` tool once with a patch whose `target` is that node. Do not explain; call \
`answer`.

Not every utterance is an instruction. When the words do not ask for a change to the UI (thanks, \
a greeting, small talk, a question about what you can do, a fragment, or text speech recognition \
invents from silence such as \"Thank you.\" or \"you\"), answer `op: decline` with a one-line \
`reason`; for a question, the reason is the short answer. Never turn such words into a change. \
An open request is an instruction, not a reason to decline: \"show me what you can do\", \"be \
creative\", \"surprise me\", \"build the most complex form you can\", \"draw some charts\" ask \
for a substantial, sensible change at the target, usually a `batch` of several composites that \
fit the page and read views the document has. Decline only when there is nothing to build.

The instruction comes from speech-to-text. Ignore filler words (um, uh, like, so, please, can \
you), false starts and repetitions. Kind names may be mis-heard: \"table\", \"list\", \"grid\" or \
\"collections\" mean `collection`; \"filter bar\" or \"filters\" mean `filter_bar`; \"graph\" or \
\"plot\" mean `chart`; \"card\", \"details\" or \"detail view\" mean `record`; \"number\", \"KPI\" \
or \"stat\" mean `metric`; \"text\" or \"markdown\" mean `rich_text`; \"dashboard\" means `board`; \
\"drawer\", \"dialog\", \"modal\" or \"popup\" mean an overlay. Read a word by what it most \
plausibly means in a UI editor, never literally when that makes no sense.

A document has layers. A shell is an application frame made of regions (navigation, \
page_outlet, overlay_outlet, notifications, assistant, account_menu) and may hold overlays \
shared by every page. The navigation lists pages in menu sections. A page is one route: it has \
a kind (list_page, report_page, settings_page, dashboard_page, editor_page, form_page, or one \
the document declares), a title, sections in layout order and overlays. A section is a \
composite with its own read; an overlay is a drawer, dialog, fullscreen pane or popover holding \
one composite (`kind` plus `component` and its props inline). Inside a composite, a board holds \
`widgets` and a collection holds `item` composites per row.

The 14 composite kinds, and when to use each:
- collection: rows of a view, as a table or list. Props: `title`, `columns: [{field: …}]` \
(optionally `as: tag`), `row_actions: [{opens: <overlay>, label: …}]`.
- record: one row shown as fields (`title`, `fields: [...]`).
- form: input bound to a command (`does: <domain>.<Command>`, `fields: [...]`, `title`).
- choice: a pick from fixed options or from a view.
- filter_bar: search, time-window and filter inputs above a collection.
- header: a title, a total and actions at the top of a page.
- overlay: a nested overlay opened from inside a composite.
- confirm: a confirmation step before a command runs.
- metric: one number (`title`, `from: <field>`).
- chart: a series over time or categories (`chart: bar`, `line` or `pie`; `x: <field>`; \
`series: [{field: …}]`).
- board: a grid of widgets.
- graph_editor: nodes and edges edited as a whole.
- rich_text: formatted text.
- references: what uses a record.

Data comes from ESS views: `reads: {view: <domain>.<View>}`, with `params` for fixed filters \
(for example `params: {state: overdue}`). Prefer a view the document already reads when it holds \
the data asked for. When no existing view fits, read a placeholder `draft.<Name>` view (for \
example `draft.OverdueLoans`); it marks data the model does not provide yet. Never invent a \
non-draft view name.

Names of new nodes (sections, overlays, pages, widgets, items) are short, lower-case, and use \
underscores: `overdue`, `due_soon`. They must be unique among their siblings. Use `insert` to \
add a child under the target, `replace` to change the target node itself (give the whole new \
node, keeping what the operator did not ask to change), and `remove` to delete it. When the \
instruction changes the pointed-at node itself (its columns, title, fields, actions or props: \
\"also show X\", \"rename this\", \"add a column\"), use `replace` on that node; use `insert` only \
for a new child, and never replace a parent to add one child. When one instruction needs more \
than one node changed, use `batch` with `patches` in order, each with its own `target`: for \
example a row action that opens a drawer is an `insert` of the drawer overlay on the page \
followed by a `replace` of the collection adding `row_actions: [{opens: <drawer>}]`. \
Columns, a metric's `from` and form or record fields name fields of the rows the composite \
reads; when a view's fields are listed, use only those, and read a `draft.` view when the data \
asked for is not among them. An `opens` \
value must name an overlay of the page or its shell. If a patch you proposed is refused, the \
refusal names the check it failed; fix exactly that and answer again.";

/// The first message of a run: the utterance and the node's context.
fn request(
    doc: &Document,
    context: &NodeContext,
    utterance: &str,
    fields: &[(String, Vec<String>)],
) -> String {
    let join = |items: Vec<String>| {
        if items.is_empty() {
            "none".to_owned()
        } else {
            items.join(", ")
        }
    };
    let layers = context
        .allowed_children
        .iter()
        .map(|layer| layer.as_str().to_owned())
        .collect();
    let kinds = context
        .composite_kinds
        .iter()
        .map(|&kind| kind.to_owned())
        .collect();
    format!(
        "Instruction (speech-to-text): \"{utterance}\"\n\n\
         Target node: {path} ({kind})\n\
         Ancestors: {ancestors}\n\
         Child layers it can take: {layers}\n\
         Composite kinds a new composite child can be: {kinds}\n\
         Existing children: {children}\n\
         Views the document already reads: {views}\n\
         Fields of each view's rows: {fields}\n\n\
         The target node as YAML:\n```yaml\n{yaml}```",
        path = context.path,
        kind = context.kind,
        ancestors = join(context.ancestors.clone()),
        layers = join(layers),
        kinds = join(kinds),
        children = join(context.children.clone()),
        views = join(known_views(doc)),
        fields = join(
            fields
                .iter()
                .map(|(view, names)| format!("{view} ({})", names.join(", ")))
                .collect(),
        ),
        yaml = context.yaml,
    )
}

/// Every view the document reads or has fixtures for, sorted, once each.
fn known_views(doc: &Document) -> Vec<String> {
    let mut views: Vec<String> = uilab_doc::check::composites(doc)
        .into_iter()
        .filter_map(|(_, composite)| composite.reads.as_ref().map(|reads| reads.view.clone()))
        .chain(
            doc.fixtures
                .iter()
                .flat_map(|fixtures| fixtures.views.keys().cloned()),
        )
        .collect();
    views.sort();
    views.dedup();
    views
}

/// The message that feeds a refusal back for the second attempt.
fn retry(refusal: &Refusal) -> String {
    format!(
        "The document refused that patch. {check}: {message}\n\
         Propose a corrected patch for the same instruction by calling `answer` again.",
        check = refusal.check,
        message = refusal.message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proposer_is_send() {
        fn send<T: Send>() {}
        send::<Proposer>();
    }

    #[test]
    fn cost_is_absent_when_any_attempt_is_unpriced() {
        assert_eq!(total(&[Some(3), Some(4)]), Some(7));
        assert_eq!(total(&[Some(3), None]), None);
        assert_eq!(total(&[]), Some(0));
    }

    #[test]
    fn the_default_is_the_subscription_route() {
        let config = ProposerConfig::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        let Credential::OauthFile { path, pointer } = &config.credential else {
            panic!("an oauth file");
        };
        assert!(expand_home(path).ends_with(DEFAULT_OAUTH_FILE));
        assert_eq!(pointer, DEFAULT_OAUTH_POINTER);
    }
}
