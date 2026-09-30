---
format: aep.planning-md/3
id: story:public-docs-site
kind: story
status: active
title: Public documentation site for uilab
relations:
- serves: vision:website-harness
scope:
- confidence: cited
  path: website
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:16:15Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T13:16:15Z", actor: "human:timo", revision: 4}
---
## Outcome

A public documentation site for uilab, built with Docusaurus in `website/` the way the other beyond10x repositories build theirs (`ess/website`, `aep/website`, `entity-runtime/website`), ready to be served at `https://beyond10x.github.io/uilab/`. It introduces uilab to a traditional front-end engineering team that is cautious about new tooling: what uilab is (a browser workbench paired with an agent that builds `ui-spec/1` UI specifications by voice or text), how a session works (select, say, review the proposal, accept or undo; goals; the Components tab), what the specification is and how it relates to ESS, and how a team adopts it next to its existing stack. ESS itself is introduced briefly and linked (repository and crates), not documented here. The design is futuristic and high fidelity: a custom landing page, not the stock template.

## Found by

Operator request, 2026-09-30.

## Acceptance

- `website/` with Docusaurus config (`url: https://beyond10x.github.io`, `baseUrl: /uilab/`), following the sibling sites' structure, theme and docs manifest conventions (read them; do not invent a new publishing path).
- Pages: landing; Introduction (why, for whom); Getting started (run uilab locally, the speech model optional); Concepts (the spec document, nodes and layers, proposals and review, widgets, goals, drafts and sample data); Working with the agent (instructions, the Components tab, goals, what the agent can and cannot change); The specification (`ui-spec/1` by example from `examples/library`); Adopting uilab in a front-end team (workflow next to a component library and design system, review and version control, what stays hand-written); FAQ.
- Screenshots of the real app taken from a scratch server over `examples/library` (no customer names, lending-library example only).
- `pnpm build` of the site passes with no broken links; the repository's own gate is unaffected.
- No public push, deploy or release: the site builds locally; publication follows the organization's documentation process later.
