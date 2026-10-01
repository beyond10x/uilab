---
title: The document
sidebar_position: 1
description: One ess-ui/1 YAML file describes the application; uilab reads it, renders it and writes accepted changes back.
---

# The document

Everything uilab shows comes from one file: an `ess-ui/1` document. It describes an application's
structure — frames, menu, pages and what each page shows — and names the data each part reads. It
does not contain component code, styling or business logic. The format is ESS's; its
[reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) defines every construct.

```yaml
format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
fixtures: {dir: fixtures, index: fixtures/index.yaml}
shells:
  app:
    regions:
      nav: {kind: navigation}
      main: {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: loans
  sections:
    - name: circulation
      label: Circulation
      pages: [loans]
pages:
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}, {field: member}, {field: due}]
```

## The parts

| Key | What it holds |
|---|---|
| `format` | always `ess-ui/1`; ESS's loader refuses anything else |
| `app`, `title` | the application's id and display name |
| `model` | the name of the ESS model whose views and commands the document refers to |
| `placement_profile` | where UI state lives: `thin`, `fat` or `hybrid` |
| `fixtures` | `{dir, index}`: the directory of sample-row files and the index file that maps each view to one, so the canvas renders without a backend |
| `shells` | application frames, each made of regions: `navigation`, `page_outlet`, `overlay_outlet`, `notifications`, `assistant`, `account_menu` |
| `navigation` | the home page, the menu sections and their pages, and `hidden` pages reached only by link |
| `pages` | routes; each has a `kind`, a `title`, `sections` — a list of named nodes in page order — and named `overlays` |
| `page_kinds` | app-defined page templates, each extending a built-in kind |
| `widgets` | app-defined, reusable composites (see [Widgets](./widgets.md)) |

Lists whose order matters — sections, menu sections, columns, a widget's body — are lists of nodes
that each carry a `name`; maps are used only where order means nothing, such as `pages` and
`overlays`.

## Page kinds and inherited sections

A page's `kind` is a template that contributes sections, state, a header and overlays. A
`list_page` contributes a `filters` section (a `filter_bar` on the page's search) and a `list`
section (a `collection`); the Loans page above writes its own `list`, which is merged over the
kind's by name, and inherits `filters`. ESS expands the document before anything is shown, so the
canvas shows what ESS renders: the sections a page kind contributes and the bodies of widget
instances are included and marked as inherited.

uilab writes back only what you wrote. Nothing a page kind or a widget contributes is saved to the
file.

## Read, render, write back

uilab reads the document when the server starts, renders it on the canvas and keeps it in memory
as the session's state. Each accepted proposal is written straight back to the file, and so is each
undo. There is no separate project format and no database: the file on disk is the result.

:::note[The first write normalises the file]
uilab writes the whole document back in one canonical YAML layout: block style instead of inline
`{…}` and `[…]`. Comments are not preserved. Commit that normalisation once, on its own, so later
diffs show only the changes you accepted.
:::

## Checks

ESS's checker, `ess-ui-check`, decides what a document may hold. Every document is checked
continuously, and every proposal is checked before you see it: a proposal whose result has an ESS
error the document did not already have is refused. Warnings are listed in the sidebar and in the
Docs view. Every finding names the node it is about. Among the checks:

| Check | Holds that |
|---|---|
| `document_loads` | the document has the shape `ess-ui/1` defines: required fields present, no field a construct does not declare (a `title` on a `collection`, `reads` on a `filter_bar`) |
| `names_unique` | sibling nodes have different names |
| `nav_resolves` | the home page, every menu page and every hidden page exist |
| `page_reachable` | every page is in the menu or hidden |
| `opens_resolves` | `opens` names an overlay of the page, its kind or its shell |
| `section_refs` | a header's `total` and `filters`, a `depends_on` and a layout name sections of the page |
| `widget_expands` | a `component` that is not a built-in kind names a declared widget, the instance supplies every required param, and no widget contains itself |
| `fixture_per_view` (warning) | a view that is read has a fixture |
| `unbound_placeholder` (warning) | a read is a placeholder not yet bound to a model view |

uilab keeps a check of its own only where ESS has none. Run the same checks from a shell with
`ess ui check --path <file>`.

## The generated Docs view

Press `3` to see documentation generated from the document: navigation, each page with its
sections and the views they read, the component catalogue, widgets, data views and which
composites read them, commands and the overlays they open, and the current findings. Nothing in it
is written by hand, so it is always current.

![The Docs view, generated from the document](/img/screens/docs.png)

## Where `ess-ui/1` comes from

`ess-ui/1` is the UI specification format of [ESS](https://github.com/beyond10x/ess), the
beyond10x toolchain for executable system specifications, released with ess 0.47.0. uilab follows
ess 0.48.0. ESS's [reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) is generated
from the format's schema. A construct uilab needs and the format lacks is requested in ESS and
waits for an ESS release; uilab never adds it to the document on its own. See
[The specification](../specification.md) for the full walkthrough and how uilab relates to ESS.
