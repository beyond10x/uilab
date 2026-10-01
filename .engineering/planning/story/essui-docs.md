---
format: aep.planning-md/3
id: story:essui-docs
kind: story
status: implemented
title: README, AGENTS.md and the docs site say ess-ui/1
relations:
- decomposes: epic:ess-ui-adoption
- depends_on: story:essui-app-widget
- depends_on: story:essui-agent-schema
- serves: vision:website-harness
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: README.md
- confidence: cited
  path: website/docs
- confidence: cited
  path: website/src
- confidence: cited
  path: website/static/img/screens
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:21Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":6}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:21Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":6}}}
- {from: "active", to: "implemented", at: "2026-10-01T17:43:23Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":6}}}
---
## Outcome

Everything a reader sees says `ess-ui/1` and points at ESS's own reference for the format,
https://beyond10x.github.io/ess/docs/reference/ess-ui: the README, the AGENTS.md layout lines, and
the public docs site (beyond10x.github.io/uilab). The site's screenshots are retaken from the
`ess-ui/1` app, with the Preview button.

## Acceptance

- `grep -rn 'ui-spec' README.md AGENTS.md website/docs website/src` prints nothing and exits 1.
- `grep -c 'https://beyond10x.github.io/ess/docs/reference/ess-ui'` prints at least 1 for each of
  `README.md`, `AGENTS.md`, `website/docs/specification.md`, `concepts/the-document.md`,
  `concepts/drafts-and-sample-data.md`, `adopting.md`, `faq.md`, `index.md` and
  `working-with-the-agent.md`; `concepts/drafts-and-sample-data.md` contains `reads: {placeholder`
  and no `draft.` view.
- `git diff --stat main -- website/static/img/screens` lists every file in that directory, and
  every screenshot of the canvas shows the Preview/Structure toggle (review criterion b below).
- Deployed: the `docs` workflow run for the merge commit on `main` concludes `success`, and
  `curl -s https://beyond10x.github.io/uilab/docs/specification` contains `ess-ui/1`.
- One independent visual review of the deployed site returns PASS against these criteria, recorded
  as a review-result: (a) wherever the format is named, it is `ess-ui/1` and links ESS's reference;
  (b) every canvas screenshot shows the Preview/Structure toggle; (c) the screenshots show the
  converted library example, including the Loans page's inherited `filters` section marked as
  inherited; (d) no broken image, and the navbar and menu are legible in light and dark.

## Scope

- README.md, AGENTS.md (layout and the uilab-doc paragraph) (cited)
- website/docs/**, website/src/** (cited: `src/pages/index.tsx:143,293` say `ui-spec/1`),
  website/static/img/screens/** (cited)

Depends on story:essui-app-widget (the screenshots) and story:essui-agent-schema (the agent
page).
