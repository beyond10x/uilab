---
title: The document
sidebar_position: 1
description: One ui-spec/1 YAML file describes the application; uilab reads it, renders it and writes accepted changes back.
---

# The document

Everything uilab shows comes from one file: a `ui-spec/1` document. It describes an application's
structure — frames, menu, pages and what each page shows — and names the data each part reads. It
does not contain component code, styling or business logic.

```yaml
format: ui-spec/1
app: library
title: Lending library
model: library
placement_profile: fat
fixtures:
  dir: fixtures
  views:
    loans.All: loans.yaml
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
      list:
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}, {field: member}, {field: due}]
```

## The parts

| Key | What it holds |
|---|---|
| `format` | always `ui-spec/1`; the `format_marker` check refuses anything else |
| `app`, `title` | the application's id and display name |
| `model` | the name of the ESS model whose views and commands the document refers to |
| `fixtures` | where sample rows for each view live, so the canvas renders without a backend |
| `shells` | application frames, each made of regions: `navigation`, `page_outlet`, `overlay_outlet`, `notifications`, `assistant`, `account_menu` |
| `navigation` | the home page, the menu sections and their pages, and `hidden` pages reached only by link |
| `pages` | routes; each has a `kind`, a `title`, named `sections` in layout order and named `overlays` |
| `widgets` | app-defined, reusable composites (see [Widgets](./widgets.md)) |

## Read, render, write back

uilab reads the document when the server starts, renders it on the canvas and keeps it in memory
as the session's state. Each accepted proposal is written straight back to the file, and so is each
undo. There is no separate project format and no database: the file on disk is the result.

:::note[The first write normalises the file]
uilab writes the whole document back in one canonical YAML layout: block style instead of inline
`{…}` and `[…]`, keys in its own order. Comments are not preserved. Commit that normalisation once,
on its own, so later diffs show only the changes you accepted.
:::

## Checks

Every document is checked continuously, and every proposal is checked before you see it. Errors
refuse a proposal; warnings are listed in the sidebar and in the Docs view. Among them:

| Check | Holds that |
|---|---|
| `nav_resolves` | the home page, every menu page and every hidden page exist |
| `page_reachable` | every page is in the menu or hidden |
| `nav_unique` | every page is listed once and menu section names are unique |
| `page_outlet` | a shell a page renders in has a `page_outlet` region |
| `opens_resolves` | `opens` names an overlay of the page or of its shell |
| `widget_resolves` | a `component` that is not a built-in kind names a declared widget |
| `widget_args` | a widget instance supplies every required param and nothing undeclared |
| `fixture_per_view` (warning) | a view that is read has a fixture |
| `draft_read` (warning) | a read is a `draft.` placeholder with no model binding yet |

## The generated Docs view

Press `3` to see documentation generated from the document: navigation, each page with its
sections and the views they read, the component catalogue, widgets, data views and which
composites read them, commands and the overlays they open, and the current findings. Nothing in it
is written by hand, so it is always current.

![The Docs view, generated from the document](/img/screens/docs.png)

## Where `ui-spec/1` comes from

`ui-spec/1` is the UI specification format of [ESS](https://github.com/beyond10x/ess), the
beyond10x toolchain for executable system specifications. It is still being settled there and is
not part of a released ESS yet. uilab reads the subset it edits with its own reader
(`crates/uilab-doc`) and follows the format as ESS publishes it. See
[The specification](../specification.md) for the full walkthrough and how uilab relates to ESS.
