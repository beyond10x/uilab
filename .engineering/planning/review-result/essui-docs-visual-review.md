---
format: aep.planning-md/3
id: review-result:essui-docs-visual-review
kind: review-result
status: active
title: Independent visual review of the deployed docs (story:essui-docs)
relations:
- reviews: story:essui-docs
revision: 1
---
approve

PASS

Nothing fails any of the four criteria. Two points that don't fail but are worth a look:
- https://beyond10x.github.io/uilab/ — the hero picture (top right) is a drawn mock-up of the canvas, not a screenshot. It has no Preview button and doesn't show the Loans page. It sits outside (b) and (c), which ask only about screenshots, but the next section is titled "Screenshots, not mock-ups". This is a judgement call.
- https://beyond10x.github.io/uilab/ — the "One file is the record" card names `ess-ui/1` but links only to github.com/beyond10x/ess, not to the ESS reference. I counted the card as naming the format, not describing it, so it is outside (a).

**How each criterion held**

| Criterion | Result |
|---|---|
| (a) | `ui-spec/1` appears in no page's text or HTML, and is not visible in any of the 6 distinct screenshots. Every page that describes the format links https://beyond10x.github.io/ess/docs/reference/ess-ui, and that link returns 200: Introduction, The document, Nodes and layers, Widgets, Drafts and sample data, Working with the agent, The specification, Adopting, FAQ. Getting started, Proposals and review and Goals don't name the format. |
| (b) | Every screenshot taken on the UI tab shows the "Preview p" button: canvas, proposal, and help (where it shows behind the dialog). Getting started and the help view both say this button switches the canvas between structure and preview. The YAML, Docs and Components screenshots show other tabs, not the canvas. |
| (c) | The canvas screenshot shows the library example on the Loans page. The `filters · filter_bar` section has a dashed outline on the canvas and is marked INHERITED in the tree. The proposal, YAML, Docs and help screenshots also show it marked INHERITED in the tree. |
| (d) | All images loaded with real sizes (2880x1800 screenshots, 150x150 logos), and no request returned 4xx or 5xx in either theme. Navbar and sidebar are readable in both themes. Dark: navbar background rgb(10,13,23), text rgb(217,221,236). Light: navbar background rgb(251,251,254), text rgb(35,39,58). I also checked them by eye in the screenshots. |

**What I checked**
- **Pages:** the home page and all 12 sidebar pages, in light and dark, at 1440x900 with Playwright 1.62.1 Chromium. Sidebar pages: Introduction, Getting started, The document, Nodes and layers, Proposals and review, Widgets, Goals, Drafts and sample data, Working with the agent, The specification, Adopting uilab in a front-end team, FAQ.
- **My screenshots:** 52 PNGs (a full-page and a first-screen shot per page and theme) in `~/.cache/uilab-wave-w13/review/`. The crawl data is in `report.json` there.
- **The site's screenshots:** I downloaded all 11 image URLs into `~/.cache/uilab-wave-w13/review/imgs/` and viewed each of the 6 distinct images (identical files counted once): canvas, proposal, components-proposal, yaml, docs, help.
- **Cleanup:** I deleted the Node crawl script. No Python was written, and no repository was touched.

Recorded by the coordinator: the reviewer returned "PASS" on its first line; the store's verdict line "approve" was added above it, and the empty findings block below. The card links to /docs/concepts/the-document, which links ESS's reference; the plain ESS link is the footer's.

```findings
[]
```
