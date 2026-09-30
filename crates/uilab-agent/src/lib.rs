//! One spoken instruction at one node of a `ui-spec/1` document, turned into one patch that
//! [`uilab_doc::admit`] accepts.
//!
//! A [`Proposer`] runs one agent run through the b10x harness loop (`harness-loop`). The run has
//! no tools and an approver that refuses everything; its only way to finish is the loop's own
//! `answer` tool, published with [`uilab_doc::patch_schema`] as its input schema, which the loop
//! validates locally before it counts as an answer. The answer is then read as a
//! [`uilab_doc::Patch`] and admitted. A refusal is fed back to the model once, on the same
//! conversation; a second refusal is [`ProposeError::Refused`].
//!
//! [`Proposer::answer_in`] is `propose` where the agent may instead move the target once, as a
//! [`Retarget`], when the instruction names a place outside it or only asks to go somewhere.
//!
//! [`Proposer::plan_goal`] runs the same way for a goal: one run whose answer is an ordered
//! [`Plan`] of at most N [`Step`]s, each an instruction `propose` can carry out at its target,
//! held to [`check_plan`] with the same one retry.

use std::path::PathBuf;
use std::sync::Arc;

use harness_loop::{
    AgentLoop, Budget, DenyAll, LoopConfig, LoopError, NullLoopSink, OutputSchema, RateCard,
    RunLedger,
};
use harness_wire::{
    Bearer, BearerSource, ModelPort, ToolCall, ToolOutcome, ToolPort, ToolSpec, WireError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uilab_doc::{Document, Layer, NodeContext, NodePath, Patch, PathError, Refusal};

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

/// Where the operator gave the instruction: the app canvas, or the Components workspace, where
/// what they ask for is reusable widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Workspace {
    #[default]
    App,
    Components,
}

impl Workspace {
    /// The line the model reads before the target; empty for the app canvas.
    fn note(self) -> &'static str {
        match self {
            Workspace::App => "",
            Workspace::Components => {
                "Workspace: Components. The operator is building the component library: what they \
                 ask for is reusable widgets declared under `widgets:` at the root (target `/`, \
                 layer `component`, each with a summary, typed params and a body), not page \
                 sections or pages. A set of components is one batch of widget inserts. Put a \
                 widget on a page only when they ask for that.\n\n"
            }
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

/// A move of the target the agent answered instead of a patch: the instruction names a place
/// outside the target, or only asks to go somewhere.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Retarget {
    /// Where the target moves; [`check_retarget`] accepted it.
    pub path: NodePath,
    /// One line the operator reads: why the target moves.
    pub reason: String,
    /// The instruction only asks to go there: nothing is proposed after the move.
    pub navigate_only: bool,
    /// Turns across every attempt.
    pub turns: u64,
    /// What every attempt cost, in millionths of a US dollar; `None` when no rate card priced it.
    pub cost_micro_usd: Option<u64>,
    /// Attempts made: 1, or 2 after one refusal.
    pub attempts: u32,
}

/// What [`Proposer::answer_in`] came back with: a patch at the target, or a move of the target.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Answer {
    Patch(Proposal),
    Retarget(Retarget),
}

/// An accepted answer where a move is allowed, before the attempts are counted in.
enum Reply {
    Patch(Patch),
    Move {
        path: NodePath,
        reason: String,
        navigate_only: bool,
    },
}

/// Whether the agent may move the target from `from` to `to` for `utterance`: `to` names a node
/// of `doc` (`/` for a new page and `nav` for a menu entry always do), a move that asks again is
/// not to `from` itself, and on the Components tab it stays at `/` or a `component:` path unless
/// the instruction names a page.
///
/// # Errors
///
/// The [`Refusal`] `path_resolves`, `retarget_same` or `retarget_workspace`.
pub fn check_retarget(
    doc: &Document,
    from: &NodePath,
    to: &NodePath,
    utterance: &str,
    navigate_only: bool,
    workspace: Workspace,
) -> Result<(), Refusal> {
    if uilab_doc::resolve(doc, to).is_err() {
        return Err(Refusal {
            check: "path_resolves".to_owned(),
            message: format!(
                "no node at `{to}`; move to a node that exists: `/` for a new page, `nav` for a \
                 menu entry, or the page, section, overlay or region the instruction names"
            ),
        });
    }
    if to == from && !navigate_only {
        return Err(Refusal {
            check: "retarget_same".to_owned(),
            message: format!("`{to}` is the target already; propose the patch here"),
        });
    }
    let among_widgets =
        to.0.first()
            .is_none_or(|segment| segment.layer == Layer::Component);
    if workspace == Workspace::Components && !among_widgets && !names_a_page(doc, utterance) {
        return Err(Refusal {
            check: "retarget_workspace".to_owned(),
            message: format!(
                "the instruction came from the Components tab and names no page; stay at `/` or \
                 a `component:` path rather than `{to}`"
            ),
        });
    }
    Ok(())
}

/// Whether `utterance` names a page as a place: a page's name or title said in words that are not
/// part of a widget name it also says ("the loan card" names the widget `loan_card`, not the page
/// `loans`), and said as a place: followed by "page" ("the loans page"), after "page", or after
/// in/on/to/into/onto/at, with or without "the" ("in members", "on the loans page"). A word that
/// only matches a page ("show the loan's due date", "as wide as the page") names none.
fn names_a_page(doc: &Document, utterance: &str) -> bool {
    const PLACE: [&str; 6] = ["in", "on", "to", "into", "onto", "at"];
    let said = words(utterance);
    let mut free = vec![true; said.len()];
    for widget in doc.widgets.keys() {
        for (start, len) in runs_of(&said, widget) {
            free[start..start + len].fill(false);
        }
    }
    let word = |i: Option<usize>| i.and_then(|i| said.get(i)).map(String::as_str);
    let as_place = |start: usize, len: usize| {
        let before = word(start.checked_sub(1));
        let preposition = |w: Option<&str>| w.is_some_and(|w| PLACE.contains(&w));
        word(Some(start + len)).is_some_and(|w| same_word(w, "page"))
            || before == Some("page")
            || preposition(before)
            || (before == Some("the") && preposition(word(start.checked_sub(2))))
    };
    let named = |name: &str| {
        runs_of(&said, name)
            .into_iter()
            .any(|(start, len)| free[start..start + len].iter().all(|f| *f) && as_place(start, len))
    };
    doc.pages
        .iter()
        .any(|(name, page)| named(name) || page.title.as_deref().is_some_and(named))
}

/// Where `said` says `name` as whole words (underscores as spaces, plurals as [`same_word`]): the
/// start and length of each run.
fn runs_of(said: &[String], name: &str) -> Vec<(usize, usize)> {
    let name = words(name);
    if name.is_empty() {
        return Vec::new();
    }
    said.windows(name.len())
        .enumerate()
        .filter(|(_, run)| {
            run.iter()
                .zip(&name)
                .all(|(spoken, word)| same_word(spoken, word))
        })
        .map(|(start, _)| (start, name.len()))
        .collect()
}

/// A goal broken into ordered steps, and what planning it took.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Plan {
    /// The steps, in the order they are to be proposed; [`check_plan`] accepted them.
    pub steps: Vec<Step>,
    /// Turns across every attempt.
    pub turns: u64,
    /// What every attempt cost, in millionths of a US dollar; `None` when no rate card priced it.
    pub cost_micro_usd: Option<u64>,
}

/// One instruction of a plan, for one [`Proposer::propose`] at its target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// What to change, as the operator would say it.
    pub instruction: String,
    /// The node the instruction is about: one that exists, a new child of one, or a new child of
    /// an earlier step's target.
    pub target: NodePath,
    /// What the step contributes to the goal.
    pub why: String,
}

/// A plan answer as the model gives it; `op` and `reason` are read before this.
#[derive(Deserialize)]
struct PlanAnswer {
    #[serde(default)]
    steps: Vec<RawStep>,
}

/// A step with its target still as text: a blank one would otherwise parse as the root.
#[derive(Deserialize)]
struct RawStep {
    instruction: String,
    target: String,
    why: String,
}

impl PlanAnswer {
    /// The steps with parsed targets; a blank or malformed target is refused, naming the step.
    fn steps(self) -> Result<Vec<Step>, Refusal> {
        self.steps
            .into_iter()
            .enumerate()
            .map(|(index, raw)| {
                if raw.target.trim().is_empty() {
                    return Err(Refusal {
                        check: "plan_step_blank".to_owned(),
                        message: format!(
                            "step {} has no target; name the node the instruction is about",
                            index + 1
                        ),
                    });
                }
                let target = raw.target.parse().map_err(|error: PathError| Refusal {
                    check: "plan_shape".to_owned(),
                    message: format!("step {}: {error}", index + 1),
                })?;
                Ok(Step {
                    instruction: raw.instruction,
                    target,
                    why: raw.why,
                })
            })
            .collect()
    }
}

/// An accepted answer and what the attempts took.
struct Answered<T> {
    value: T,
    turns: u64,
    cost_micro_usd: Option<u64>,
    attempts: u32,
}

/// What a refusal fed back asks to be corrected.
#[derive(Clone, Copy)]
struct Retry {
    answer: &'static str,
    request: &'static str,
}

impl Retry {
    const PATCH: Retry = Retry {
        answer: "patch",
        request: "instruction",
    };
    const PLAN: Retry = Retry {
        answer: "plan",
        request: "goal",
    };
    const ANSWER: Retry = Retry {
        answer: "answer",
        request: "instruction",
    };

    /// The message that feeds a refusal back for the next attempt.
    fn message(self, refusal: &Refusal) -> String {
        format!(
            "The document refused that {answer}. {check}: {message}\n\
             Propose a corrected {answer} for the same {request} by calling `answer` again.",
            answer = self.answer,
            request = self.request,
            check = refusal.check,
            message = refusal.message,
        )
    }
}

/// Whether `steps` is a plan the runner can carry out on `doc`, one proposal at a time: at least
/// one step, at most `max_steps`, every instruction non-blank, and every target valid. A target
/// is valid when it resolves in `doc`, or its parent resolves in `doc`, or its parent is an
/// earlier step's target; nothing else. So a step may target a node an earlier step inserts under
/// an existing node or under that earlier step's own target, but not one deeper than that.
///
/// This checks the plan against `doc` as it is, not as the earlier steps will leave it: a target
/// that an earlier step removes or replaces away still passes here. The runner re-checks each
/// step's target when it proposes it, on the document the earlier steps left.
///
/// # Errors
///
/// The [`Refusal`] `plan_empty`, `plan_too_long`, `plan_step_blank` or `plan_target`, naming the
/// step.
pub fn check_plan(doc: &Document, steps: &[Step], max_steps: usize) -> Result<(), Refusal> {
    if steps.is_empty() {
        return Err(Refusal {
            check: "plan_empty".to_owned(),
            message: "the plan has no steps; give at least one, or decline".to_owned(),
        });
    }
    if steps.len() > max_steps {
        return Err(Refusal {
            check: "plan_too_long".to_owned(),
            message: format!(
                "the plan has {} steps and the cap is {max_steps}; merge or drop steps",
                steps.len()
            ),
        });
    }
    let resolves = |path: &NodePath| uilab_doc::resolve(doc, path).is_ok();
    for (index, step) in steps.iter().enumerate() {
        if step.instruction.trim().is_empty() {
            return Err(Refusal {
                check: "plan_step_blank".to_owned(),
                message: format!(
                    "step {} has no instruction; say what to change at `{}`, or drop the step",
                    index + 1,
                    step.target
                ),
            });
        }
        let valid = resolves(&step.target)
            || step.target.parent().is_some_and(|parent| {
                resolves(&parent) || steps[..index].iter().any(|e| e.target == parent)
            });
        if !valid {
            return Err(Refusal {
                check: "plan_target".to_owned(),
                message: format!(
                    "step {} targets `{}`: neither it nor its parent is a node of the document, \
                     and its parent is no earlier step's target; target a node that exists, a \
                     new child of one, or a new child of an earlier step's target",
                    index + 1,
                    step.target
                ),
            });
        }
    }
    Ok(())
}

/// The answer schema of a plan: `op: plan` with at most `max_steps` steps, or `op: decline`.
fn plan_schema(max_steps: usize) -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["op"],
        "additionalProperties": false,
        "properties": {
            "op": {
                "enum": ["plan", "decline"],
                "description": "plan gives `steps`; decline changes nothing, with a `reason`, when the goal is not a request to change the UI"
            },
            "reason": {
                "type": "string",
                "description": "for decline only: why nothing is planned, or the answer to a question"
            },
            "steps": {
                "description": "for plan only: the steps in the order they run",
                "type": "array",
                "maxItems": max_steps,
                "items": {
                    "type": "object",
                    "required": ["instruction", "target", "why"],
                    "additionalProperties": false,
                    "properties": {
                        "instruction": {
                            "type": "string",
                            "minLength": 1,
                            "description": "one instruction, complete on its own, that one patch at `target` carries out"
                        },
                        "target": {
                            "type": "string",
                            "minLength": 1,
                            "description": "the node path the instruction is about: an existing node, a new child of one, or a new child of an earlier step's target"
                        },
                        "why": {
                            "type": "string",
                            "description": "one line: what this step contributes to the goal"
                        }
                    }
                }
            }
        }
    })
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
    #[error("the agent run ended without an answer: {0}")]
    Stopped(String),
    /// Every attempt was refused; this is the last refusal.
    #[error("the answer was refused on every attempt; last refusal {check}: {message}")]
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
        self.propose_in(doc, target, utterance, fields, Workspace::App)
    }

    /// [`propose_with`](Self::propose_with), given in `workspace`.
    ///
    /// # Errors
    ///
    /// As [`propose`](Self::propose).
    pub fn propose_in(
        &mut self,
        doc: &Document,
        target: &NodePath,
        utterance: &str,
        fields: &[(String, Vec<String>)],
        workspace: Workspace,
    ) -> Result<Proposal, ProposeError> {
        let context = uilab_doc::node_context(doc, target)?;
        let schema = uilab_doc::patch_schema(doc, target)?;
        let config = self.loop_config(INSTRUCTIONS, schema)?;
        let answered = self.attempts(
            config,
            request(doc, &context, target, utterance, fields, workspace),
            Retry::PATCH,
            |structured| admitted_patch(doc, structured),
        )?;
        Ok(Proposal {
            patch: answered.value,
            turns: answered.turns,
            cost_micro_usd: answered.cost_micro_usd,
            attempts: answered.attempts,
        })
    }

    /// Blocking. [`propose_in`](Self::propose_in), where the agent may instead move the target
    /// once: an instruction that names a place outside the target ("a new page", "in the menu",
    /// "on the members page") answers a [`Retarget`] there, which the caller carries out and then
    /// asks [`propose_in`](Self::propose_in) at the new target; an instruction that only asks to
    /// go somewhere answers one with `navigate_only`. The move is held to [`check_retarget`], with
    /// the same one retry as a patch.
    ///
    /// # Errors
    ///
    /// As [`propose`](Self::propose).
    pub fn answer_in(
        &mut self,
        doc: &Document,
        target: &NodePath,
        utterance: &str,
        fields: &[(String, Vec<String>)],
        workspace: Workspace,
    ) -> Result<Answer, ProposeError> {
        let context = uilab_doc::node_context(doc, target)?;
        let schema = with_retarget(uilab_doc::patch_schema(doc, target)?);
        let instructions = format!("{INSTRUCTIONS}\n\n{MOVE_INSTRUCTIONS}");
        let config = self.loop_config(&instructions, schema)?;
        let mut first = request(doc, &context, target, utterance, fields, workspace);
        first.push_str(&places(doc));
        let answered = self.attempts(config, first, Retry::ANSWER, |structured| {
            if structured["op"] != "retarget" {
                return admitted_patch(doc, structured).map(Reply::Patch);
            }
            let path = structured["path"].as_str().unwrap_or_default().trim();
            if path.is_empty() {
                return Err(Refusal {
                    check: "retarget_shape".to_owned(),
                    message: "a retarget names the `path` to move to".to_owned(),
                });
            }
            let path: NodePath = path.parse().map_err(|error: PathError| Refusal {
                check: "retarget_shape".to_owned(),
                message: error.to_string(),
            })?;
            let navigate_only = structured["navigate_only"].as_bool().unwrap_or(false);
            check_retarget(doc, target, &path, utterance, navigate_only, workspace)?;
            let reason = structured["reason"]
                .as_str()
                .map(str::trim)
                .filter(|reason| !reason.is_empty())
                .unwrap_or("the instruction names another place")
                .to_owned();
            Ok(Reply::Move {
                path,
                reason,
                navigate_only,
            })
        })?;
        Ok(match answered.value {
            Reply::Patch(patch) => Answer::Patch(Proposal {
                patch,
                turns: answered.turns,
                cost_micro_usd: answered.cost_micro_usd,
                attempts: answered.attempts,
            }),
            Reply::Move {
                path,
                reason,
                navigate_only,
            } => Answer::Retarget(Retarget {
                path,
                reason,
                navigate_only,
                turns: answered.turns,
                cost_micro_usd: answered.cost_micro_usd,
                attempts: answered.attempts,
            }),
        })
    }

    /// Blocking. One goal at one node → an ordered list of at most `max_steps` instructions,
    /// each one [`propose`](Self::propose) can carry out at its target. One harness run, on the
    /// same port and credential as `propose`.
    ///
    /// # Errors
    ///
    /// [`ProposeError::Target`] for a path that names no node, [`ProposeError::Config`] for a
    /// `max_steps` of 0, [`ProposeError::Declined`] for a goal that is not a UI change,
    /// [`ProposeError::Run`] and [`ProposeError::Stopped`] for a run that gave no answer, and
    /// [`ProposeError::Refused`] when [`MAX_ATTEMPTS`] plans were all refused by [`check_plan`].
    pub fn plan_goal(
        &mut self,
        doc: &Document,
        target: &NodePath,
        goal: &str,
        max_steps: usize,
    ) -> Result<Plan, ProposeError> {
        self.plan_goal_with(doc, target, goal, max_steps, &[])
    }

    /// [`plan_goal`](Self::plan_goal), telling the model which fields each view's rows carry, as
    /// [`propose_with`](Self::propose_with) does.
    ///
    /// # Errors
    ///
    /// As [`plan_goal`](Self::plan_goal).
    pub fn plan_goal_with(
        &mut self,
        doc: &Document,
        target: &NodePath,
        goal: &str,
        max_steps: usize,
        fields: &[(String, Vec<String>)],
    ) -> Result<Plan, ProposeError> {
        self.plan_goal_in(doc, target, goal, max_steps, fields, Workspace::App)
    }

    /// [`plan_goal_with`](Self::plan_goal_with), given in `workspace`.
    ///
    /// # Errors
    ///
    /// As [`plan_goal`](Self::plan_goal).
    pub fn plan_goal_in(
        &mut self,
        doc: &Document,
        target: &NodePath,
        goal: &str,
        max_steps: usize,
        fields: &[(String, Vec<String>)],
        workspace: Workspace,
    ) -> Result<Plan, ProposeError> {
        let context = uilab_doc::node_context(doc, target)?;
        if max_steps == 0 {
            return Err(ProposeError::Config(
                "a plan needs a cap of at least one step".to_owned(),
            ));
        }
        let config = self.loop_config(PLAN_INSTRUCTIONS, plan_schema(max_steps))?;
        let answered = self.attempts(
            config,
            plan_request(doc, &context, goal, max_steps, fields, workspace),
            Retry::PLAN,
            |structured| {
                let answer =
                    serde_json::from_value::<PlanAnswer>(structured).map_err(|error| Refusal {
                        check: "plan_shape".to_owned(),
                        message: error.to_string(),
                    })?;
                let steps = answer.steps()?;
                check_plan(doc, &steps, max_steps)?;
                Ok(steps)
            },
        )?;
        Ok(Plan {
            steps: answered.value,
            turns: answered.turns,
            cost_micro_usd: answered.cost_micro_usd,
        })
    }

    /// One run's configuration: these instructions, this answer schema, this proposer's bounds.
    fn loop_config(&self, instructions: &str, schema: Value) -> Result<LoopConfig, ProposeError> {
        let answer =
            OutputSchema::new(schema).map_err(|error| ProposeError::Config(error.to_string()))?;
        Ok(LoopConfig::new(self.model.clone(), instructions)
            .with_budget(Budget {
                max_turns: Some(u64::from(self.max_turns)),
                ..Budget::default()
            })
            .with_context_window(Some(self.context_window))
            .with_prices(self.prices.clone())
            .with_output_schema(Some(answer)))
    }

    /// Up to [`MAX_ATTEMPTS`] runs on one conversation: `accept` reads each structured answer,
    /// and its refusal is fed back for the next attempt. `op: decline` ends it as
    /// [`ProposeError::Declined`].
    fn attempts<T>(
        &mut self,
        config: LoopConfig,
        first: String,
        retry: Retry,
        mut accept: impl FnMut(Value) -> Result<T, Refusal>,
    ) -> Result<Answered<T>, ProposeError> {
        let mut items = Vec::new();
        let mut turns = 0;
        let mut costs = Vec::new();
        let mut input = first;
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
            match accept(structured) {
                Ok(value) => {
                    return Ok(Answered {
                        value,
                        turns,
                        cost_micro_usd: total(&costs),
                        attempts: attempt,
                    });
                }
                Err(refused) => {
                    input = retry.message(&refused);
                    refusal = Some(refused);
                }
            }
        }
        let refused = refusal.expect("every attempt that did not return was refused");
        Err(ProposeError::Refused {
            check: refused.check,
            message: refused.message,
        })
    }
}

/// A structured answer read as a patch and admitted against `doc`.
fn admitted_patch(doc: &Document, structured: Value) -> Result<Patch, Refusal> {
    let patch = serde_json::from_value::<Patch>(structured).map_err(|error| Refusal {
        check: "patch_shape".to_owned(),
        message: error.to_string(),
    })?;
    uilab_doc::admit(doc, &patch)?;
    Ok(patch)
}

/// A patch schema that also admits `op: retarget` with a `path`, a `reason` and `navigate_only`.
fn with_retarget(mut schema: Value) -> Value {
    let properties = &mut schema["properties"];
    if let Some(ops) = properties["op"]["enum"].as_array_mut() {
        ops.push(Value::from("retarget"));
    }
    let op = properties["op"]["description"].as_str().unwrap_or_default();
    properties["op"]["description"] = Value::from(format!(
        "{op}; retarget moves the target to `path` instead of changing anything here, when the \
         instruction names a place outside the target or only asks to go somewhere"
    ));
    properties["reason"]["description"] = Value::from(
        "for decline: why nothing is proposed, or the answer to a question; for retarget: one \
         line the operator reads, saying why the target moves",
    );
    properties["path"] = serde_json::json!({
        "type": "string",
        "minLength": 1,
        "description": "for retarget only: the node path to move to: `/` for a new page, `nav` for a menu entry, or the page, section, overlay or region the instruction names"
    });
    properties["navigate_only"] = serde_json::json!({
        "type": "boolean",
        "description": "for retarget only: true when the instruction only asks to go to or select that place; nothing is proposed after the move"
    });
    schema
}

/// The places a move can go: the outline two levels deep, one node per line.
fn places(doc: &Document) -> String {
    let mut lines = String::new();
    outline_lines(&uilab_doc::outline(doc), 0, 2, &mut lines);
    format!(
        "\n\nPlaces a move can go, the outline two levels deep (path, kind, title, view it \
         reads):\n{lines}"
    )
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
\"plot\" mean `chart`; \"card\", \"details\" or \"detail view\" mean `record`, but a reusable card \
or a \"card for each …\" is a widget (below); \"component\" means a widget; \"number\", \"KPI\" \
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

Widgets are app-defined, reusable components (not a board's `widgets`, which are ordinary \
composites). A widget is declared under `widgets:` at the document root: insert it at `/` with \
layer `component` and a short lower-case name that is not a composite kind (`member_card`). Its \
node has `summary` (one line, required), `params` (each `{type: …, required: true}` or with a \
`default`; a type is `string`, `number`, `boolean`, a named type such as `Member`, or a \
constructor map), optional `arrange` (`row`, `column` or `grid`) and a `body`: a list of named \
nodes, each with `name`. A body node is a built-in composite, an instance of another widget, or a \
primitive `{name: …, primitive: <kind>, …}`. The primitive kinds are `text` (`text` or `field`, \
optional `style: heading`), `badge` (`text`, `tone`), `icon` (`label`), `button` \
(`label`, `action`), `link` (`to` or `href`), `input` (`binds`), `toggle`, `image` (`src`, \
required `alt`) and `divider`. In a body, `args.<param>` refers to a param: `text: \
args.member.name`. Add a body node with layer `node` under `component:<widget>`; change a widget \
with `replace` on it. An instance is `{component: <widget>, args: {<param>: <value>}}` and goes \
wherever a composite goes: a section, an overlay, a board's widget, a collection's item. It \
supplies every required param and none the widget does not declare; in a collection item the \
row is `row` (`args: {member: row}`). The widgets the document declares are listed with the \
instruction; use one that fits. Make a new widget when the operator asks for something reusable, \
a \"component\", a \"card for each …\", or the same structure would repeat; declare it and use it \
in one `batch` (the insert at `/` first, then the uses, each with its own target). Otherwise keep \
using built-in composites. To use a widget inside an existing table, target that collection and \
insert an `item` node (layer `item`) holding the instance; when the batch starts at `/`, its \
second patch is an `insert` at `page:<p>/section:<s>` of layer `item`. The existing sections of \
the pages the instruction names are listed with their columns and children: target them there.

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
for a new child, and never replace a parent to add one child: never replace a page or a \
collection to add one child. A `replace` must repeat every existing prop, column and child it \
does not mean to change; a replace that drops columns, sections or overlays the operator did not \
ask about is wrong. When one instruction needs more \
than one node changed, use `batch` with `patches` in order, each with its own `target`: for \
example a row action that opens a drawer is an `insert` of the drawer overlay on the page \
followed by a `replace` of the collection adding `row_actions: [{opens: <drawer>}]`. \
Columns, a metric's `from` and form or record fields name fields of the rows the composite \
reads; when a view's fields are listed, use only those, and read a `draft.` view when the data \
asked for is not among them. An `opens` \
value must name an overlay of the page or its shell. If a patch you proposed is refused, the \
refusal names the check it failed; fix exactly that and answer again.";

/// What the model is told, after [`INSTRUCTIONS`], where it may move the target.
pub const MOVE_INSTRUCTIONS: &str = "\
A patch changes only the target node and what is below it. When the instruction names a place \
outside the target (\"a new page\", \"in the menu\", \"in the sidebar\", \"on the members page\", \
\"the header\"), do not squeeze the change into the target: answer `op: retarget` with `path` set \
to that place and a one-line `reason` the operator reads (\"a new page goes under the root\"). \
The selection moves there and you are asked again at `path` with the same instruction; you can \
move only once per instruction. `path` is a node that exists, listed with the instruction: `/` \
to add a page (a page in the menu or sidebar is still added at `/`), `nav` to add a menu \
section, `page:<name>` for a page, or the section, overlay or region the instruction names.
When the instruction only asks to go somewhere or to select something (\"go to the members \
page\", \"select the menu\", \"show me the loans page\"), answer `op: retarget` with \
`navigate_only: true`: the selection moves and nothing is proposed.
When the instruction names no other place (\"add a column\", \"make this a chart\", \"add a table \
of overdue loans\" at a page), propose the patch at the target as usual; \"this\" and \"here\" mean \
the target. On the Components tab, move only to `/` or a `component:` path unless the \
instruction names a page; a widget's name does not name one (\"the loan card\" is the widget \
`loan_card`, not the loans page).";

/// What the planner is told about `ui-spec/1` and its job.
pub const PLAN_INSTRUCTIONS: &str = "\
You plan changes to one user-interface document in the `ui-spec/1` format. The operator states a \
goal while pointing at one node; you break it into an ordered list of steps and answer by calling \
the `answer` tool once with `op: plan` and `steps`. Do not explain; call `answer`.

Each step is one instruction that another agent, the proposer, will carry out on its own as \
exactly one patch at the step's `target`: one insert of a child under the target, one replace of \
the target, one remove of it, or one batch of those. The proposer sees only the step's \
instruction, its target and the document, so write each `instruction` complete on its own, as \
the operator would say it: name the composite kind, the view it reads, its fields and its title, \
and the name of any node it creates. `why` says in one line what the step adds to the goal.

Steps run in order, each on the document the earlier steps left. A later step may target a node \
an earlier step creates: name that node by the path it will have, for example \
`page:members/section:details` after a step at `page:members` that inserts the section \
`details`, and give that name in the earlier step's instruction. Every target is a node of the \
outline below, a new child of one, or a new child of an earlier step's target; never deeper. \
Every instruction says something. Use as few steps as the goal needs \
and never more than the cap; one step that adds several related nodes as a batch is better than \
several tiny steps. A drawer or dialog and the row action that opens it belong in one step.

When the goal is not a request to change the UI (thanks, a greeting, small talk, a question \
about what you can do, noise), answer `op: decline` with a one-line `reason`; for a question, \
the reason is the short answer. An open request (\"be creative\", \"build out this page\") is a \
goal, not a reason to decline.

Paths are `layer:name` segments joined by `/`: `page:loans`, `page:loans/section:list`, \
`page:loans/overlay:edit`, `shell:app/region:nav`, `nav`. A page holds sections (composites, in \
layout order) and overlays (drawers, dialogs, fullscreen panes, popovers); a board holds \
widgets; a collection holds items. Composite kinds: collection (rows as a table or list), record \
(one row as fields: a details card), form (input bound to a command), choice, filter_bar \
(search and filters above a collection), header, overlay, confirm, metric, chart, board, \
graph_editor, rich_text, references. A step can also declare a widget, an app-defined reusable \
component under `widgets:` at the root (target `/`, layer `component`, a `summary`, typed \
`params` and a `body` of named composites, widget instances and primitives), and use it as \
`{component: <widget>, args: {…}}` wherever a composite goes; declare a widget and its first use \
in one step. Plan a widget when the goal asks for something reusable, a component, a card for \
each row, or repeats a structure; use a widget the document already declares when one fits. \
Declared widgets are `component:<name>` in the outline. Data comes from ESS views: prefer a view the document \
already reads; when none fits, read a placeholder `draft.<Name>` view. Names of new nodes are \
short, lower-case, with underscores, and unique among their siblings. If your plan is refused, \
the refusal names the check it failed; fix exactly that and answer again.";

/// `none` for nothing, else the items joined by commas.
fn join(items: Vec<String>) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}

/// Each view with the fields its rows carry, as `view (a, b)`.
fn view_fields(fields: &[(String, Vec<String>)]) -> String {
    join(
        fields
            .iter()
            .map(|(view, names)| format!("{view} ({})", names.join(", ")))
            .collect(),
    )
}

/// Every widget the document declares, one `- name: summary Params: p (type, required), …` line
/// each, or `none`.
fn declared_widgets(doc: &Document) -> String {
    if doc.widgets.is_empty() {
        return "none".to_owned();
    }
    doc.widgets
        .iter()
        .map(|(name, widget)| {
            let params: Vec<String> = widget
                .params
                .iter()
                .map(|(param, declared)| {
                    let ty = match &declared.ty {
                        Value::String(ty) => ty.clone(),
                        other => other.to_string(),
                    };
                    if declared.is_required() {
                        format!("{param} ({ty}, required)")
                    } else {
                        format!("{param} ({ty})")
                    }
                })
                .collect();
            format!(
                "\n- {name}: {summary} Params: {params}",
                summary = widget.summary,
                params = join(params)
            )
        })
        .collect()
}

/// The first message of a plan run: the goal, the cap, the node's context and the whole outline.
fn plan_request(
    doc: &Document,
    context: &NodeContext,
    goal: &str,
    max_steps: usize,
    fields: &[(String, Vec<String>)],
    workspace: Workspace,
) -> String {
    let mut outline = String::new();
    outline_lines(&uilab_doc::outline(doc), 0, usize::MAX, &mut outline);
    format!(
        "Goal: \"{goal}\"\n\
         At most {max_steps} steps.\n\n\
         {note}Target node: {path} ({kind})\n\
         Ancestors: {ancestors}\n\
         Child layers it can take: {layers}\n\
         Existing children: {children}\n\
         Views the document already reads: {views}\n\
         Fields of each view's rows: {fields}\n\
         Widgets the document declares: {widgets}\n\n\
         The document outline, one node per line (path, kind, title, view it reads):\n\
         {outline}\n\
         The target node as YAML:\n```yaml\n{yaml}```",
        path = context.path,
        kind = context.kind,
        ancestors = join(context.ancestors.clone()),
        layers = join(
            context
                .allowed_children
                .iter()
                .map(|layer| layer.as_str().to_owned())
                .collect()
        ),
        children = join(context.children.clone()),
        views = join(known_views(doc)),
        fields = view_fields(fields),
        widgets = declared_widgets(doc),
        note = workspace.note(),
        yaml = context.yaml,
    )
}

/// The outline below `node`, one indented line per node, down to `max_depth` below the start.
fn outline_lines(node: &uilab_doc::OutlineNode, depth: usize, max_depth: usize, out: &mut String) {
    use std::fmt::Write as _;
    let _ = write!(
        out,
        "{:indent$}{} ({})",
        "",
        node.path,
        node.kind,
        indent = depth * 2
    );
    if let Some(title) = &node.title {
        let _ = write!(out, " \"{title}\"");
    }
    if let Some(view) = &node.view {
        let _ = write!(out, " reads {view}");
    }
    out.push('\n');
    if depth >= max_depth {
        return;
    }
    for child in &node.children {
        outline_lines(child, depth + 1, max_depth, out);
    }
}

/// The lower-case words of `text`; anything not a letter or digit, `_` included, separates them.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether two words name the same thing: equal, or one is the other's plural by `s`, `es` or
/// `ies` for `y`. A plural counts only when the singular has at least four letters, so `news` is
/// not the plural of `new`; an exact word has no minimum.
fn same_word(a: &str, b: &str) -> bool {
    let plural_of = |plural: &str, singular: &str| {
        singular.chars().count() >= 4
            && (plural.strip_suffix('s') == Some(singular)
                || plural.strip_suffix("es") == Some(singular)
                || singular
                    .strip_suffix('y')
                    .is_some_and(|stem| plural.strip_suffix("ies") == Some(stem)))
    };
    a == b || plural_of(a, b) || plural_of(b, a)
}

/// The pages a request at `target` is about: the target page, then every other page whose name
/// or title the utterance says as whole words, in order (underscores as spaces, plurals as
/// [`same_word`]). None below a page.
fn named_pages<'a>(doc: &'a Document, target: &NodePath, utterance: &str) -> Vec<&'a str> {
    let own = match target.layer() {
        Layer::Root => None,
        Layer::Page => target.0.first().map(|segment| segment.name.as_str()),
        _ => return Vec::new(),
    };
    let said = words(utterance);
    let names = |name: &str| {
        let name = words(name);
        !name.is_empty()
            && said.windows(name.len()).any(|run| {
                run.iter()
                    .zip(&name)
                    .all(|(spoken, word)| same_word(spoken, word))
            })
    };
    let own = own.and_then(|own| doc.pages.get_key_value(own).map(|(name, _)| name.as_str()));
    own.into_iter()
        .chain(
            doc.pages
                .iter()
                .filter(|(name, page)| {
                    own != Some(name.as_str())
                        && (names(name) || page.title.as_deref().is_some_and(names))
                })
                .map(|(name, _)| name.as_str()),
        )
        .collect()
}

/// A section's columns, as `name, standing (as tag)`.
fn columns(composite: &uilab_doc::model::Composite) -> Option<String> {
    let columns = composite.props.get("columns")?.as_array()?;
    Some(
        columns
            .iter()
            .map(
                |column| match (column["field"].as_str(), column["as"].as_str()) {
                    (Some(field), Some(shown)) => format!("{field} (as {shown})"),
                    (Some(field), None) => field.to_owned(),
                    _ => column.to_string(),
                },
            )
            .collect::<Vec<_>>()
            .join(", "),
    )
}

/// One line per section and overlay of each page in `pages`: path, kind, view, columns and
/// children, so a patch can target them in place rather than rewrite the page.
fn page_nodes(doc: &Document, pages: &[&str]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for &name in pages {
        let page_path = NodePath::root().child(Layer::Page, name);
        let Some(page) = doc.pages.get(name) else {
            continue;
        };
        for (section, composite) in &page.sections {
            let Some(composite) = composite else { continue };
            let path = page_path.child(Layer::Section, section);
            let _ = write!(out, "\n- {path}: {}", composite.component.as_str());
            if let Some(reads) = &composite.reads {
                let _ = write!(out, " reads {}", reads.view);
            }
            if let Some(columns) = columns(composite) {
                let _ = write!(out, "; columns {columns}");
            }
            let children = uilab_doc::path::children(doc, &path)
                .unwrap_or_default()
                .into_iter()
                .map(|(layer, child)| format!("{layer}:{child}"))
                .collect();
            let _ = write!(out, "; children {}", join(children));
        }
        for (overlay, body) in &page.overlays {
            let Some(body) = body else { continue };
            let path = page_path.child(Layer::Overlay, overlay);
            let _ = write!(
                out,
                "\n- {path}: {} {}",
                serde_json::to_value(body.kind)
                    .ok()
                    .and_then(|kind| kind.as_str().map(str::to_owned))
                    .unwrap_or_default(),
                body.body.component.as_str()
            );
        }
    }
    if out.is_empty() {
        "none".to_owned()
    } else {
        out
    }
}

/// The first message of a run: the utterance and the node's context.
fn request(
    doc: &Document,
    context: &NodeContext,
    target: &NodePath,
    utterance: &str,
    fields: &[(String, Vec<String>)],
    workspace: Workspace,
) -> String {
    let pages = named_pages(doc, target, utterance);
    let places = if matches!(target.layer(), Layer::Root | Layer::Page) {
        format!(
            "Existing nodes of the pages the instruction is about (patch them in place): {}\n",
            page_nodes(doc, &pages)
        )
    } else {
        String::new()
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
         {note}Target node: {path} ({kind})\n\
         Ancestors: {ancestors}\n\
         Child layers it can take: {layers}\n\
         Composite kinds a new composite child can be: {kinds}\n\
         Existing children: {children}\n\
         Views the document already reads: {views}\n\
         Fields of each view's rows: {fields}\n\
         Widgets the document declares: {widgets}\n\
         {places}\n\
         The target node as YAML:\n```yaml\n{yaml}```",
        path = context.path,
        kind = context.kind,
        ancestors = join(context.ancestors.clone()),
        layers = join(layers),
        kinds = join(kinds),
        children = join(context.children.clone()),
        views = join(known_views(doc)),
        fields = view_fields(fields),
        widgets = declared_widgets(doc),
        note = workspace.note(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proposer_is_send() {
        fn send<T: Send>() {}
        send::<Proposer>();
    }

    #[test]
    fn an_instruction_from_the_components_workspace_asks_for_widgets() {
        let doc = library();
        let root = NodePath::root();
        let context = uilab_doc::node_context(&doc, &root).unwrap();
        let text = "put some basic set of components now";
        let app = request(&doc, &context, &root, text, &[], Workspace::App);
        assert!(!app.contains("Workspace:"), "{app}");
        let components = request(&doc, &context, &root, text, &[], Workspace::Components);
        assert!(
            components.contains("Workspace: Components.")
                && components.contains("under `widgets:`")
                && components.contains("not page sections"),
            "{components}"
        );
        let plan = plan_request(&doc, &context, text, 4, &[], Workspace::Components);
        assert!(plan.contains("Workspace: Components."), "{plan}");
        assert!(!plan_request(&doc, &context, text, 4, &[], Workspace::App).contains("Workspace:"));
    }

    #[test]
    fn cost_is_absent_when_any_attempt_is_unpriced() {
        assert_eq!(total(&[Some(3), Some(4)]), Some(7));
        assert_eq!(total(&[Some(3), None]), None);
        assert_eq!(total(&[]), Some(0));
    }

    fn library() -> Document {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/library/library.ui.yaml");
        Document::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn steps(targets: &[&str]) -> Vec<Step> {
        targets
            .iter()
            .map(|target| Step {
                instruction: format!("change {target}"),
                target: target.parse().unwrap(),
                why: "a test".to_owned(),
            })
            .collect()
    }

    #[test]
    fn the_plan_check_refuses_over_the_cap_empty_and_unreachable_targets() {
        let doc = library();
        let refused = check_plan(&doc, &steps(&["page:members", "page:loans"]), 1).unwrap_err();
        assert_eq!(refused.check, "plan_too_long");
        assert!(refused.message.contains('1'), "{}", refused.message);
        assert_eq!(check_plan(&doc, &[], 8).unwrap_err().check, "plan_empty");
        let refused =
            check_plan(&doc, &steps(&["page:members/section:details/item:loan"]), 8).unwrap_err();
        assert_eq!(refused.check, "plan_target");
        check_plan(&doc, &steps(&["page:members/section:details"]), 8)
            .expect("a new child of a node that resolves");
        check_plan(
            &doc,
            &steps(&[
                "page:members/section:details",
                "page:members/section:details/item:loan",
            ]),
            8,
        )
        .expect("a new child of an earlier step's target");
        check_plan(&doc, &steps(&["page:members/section:list"]), 8)
            .expect("a target that resolves");
    }

    #[test]
    fn a_step_at_the_root_admits_only_its_new_children() {
        let doc = library();
        check_plan(&doc, &steps(&["/", "page:reports"]), 8).expect("a new page under the root");
        let refused =
            check_plan(&doc, &steps(&["/", "page:reports/section:table"]), 8).unwrap_err();
        assert_eq!(refused.check, "plan_target");
        assert!(refused.message.contains("step 2"), "{}", refused.message);
    }

    #[test]
    fn a_blank_instruction_is_refused_by_the_check_and_the_schema() {
        let doc = library();
        let mut plan = steps(&["page:members"]);
        plan[0].instruction = " \t".to_owned();
        let refused = check_plan(&doc, &plan, 8).unwrap_err();
        assert_eq!(refused.check, "plan_step_blank");
        assert!(refused.message.contains("step 1"), "{}", refused.message);
        let schema = plan_schema(8);
        for field in ["instruction", "target"] {
            assert_eq!(
                schema["properties"]["steps"]["items"]["properties"][field]["minLength"],
                serde_json::json!(1),
                "{field}"
            );
        }
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
