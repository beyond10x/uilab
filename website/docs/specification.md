---
title: The specification
sidebar_position: 2
description: ess-ui/1 read line by line from the lending-library example, and how uilab relates to ESS.
---

# The specification

This page reads `examples/library/library.ui.yaml` — the lending-library app uilab ships as its
example — from top to bottom. It is short enough to read whole, and it uses most of what a
list-and-detail back-office screen needs. The format is `ess-ui/1`, which ESS defines; its
[reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) lists every construct and field.

## Header

```yaml
format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
```

`format` marks the file. `app` and `title` name the application. `model` names the ESS model whose
views (`loans.All`) and commands (`loans.ExtendLoan`) the document refers to. `placement_profile`
says where UI state lives — `fat` keeps it in the browser; uilab shows it in the Docs view.

## Sample data

```yaml
fixtures: {dir: fixtures, index: fixtures/index.yaml}
```

```yaml title="fixtures/index.yaml"
views:
  loans.All: loans.yaml
  loans.Summary: loans.yaml
  members.All: members.yaml
  staff.Me: staff.yaml
```

The document names a fixture directory and an index file; the index maps each view the document
reads to a file of sample rows, so the canvas renders without a backend. See
[Drafts and sample data](./concepts/drafts-and-sample-data.md).

## The frame

```yaml
shells:
  app:
    regions:
      nav:
        kind: navigation
        props:
          collapsible: true
      account:
        kind: account_menu
        props:
          reads: {view: staff.Me}
      main:
        kind: page_outlet
      overlay:
        kind: overlay_outlet
      notify:
        kind: notifications
```

A **shell** is an application frame. Its regions say where the menu, the current page, overlays,
notifications and the account menu go. Pages render into the `page_outlet`.

## The menu

```yaml
navigation:
  home: overview
  sections:
    - name: circulation
      label: Circulation
      icon: books
      pages: [overview, loans]
    - name: people
      label: People
      icon: members
      pages: [members]
```

`home` is the start page. Every page appears in exactly one menu section or in `hidden`; the checks
refuse a page nobody can reach and a menu entry for a page that does not exist.

## The overview page

```yaml
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      - {name: board, remove: true}
      - name: on_loan
        component: metric
        label: Copies on loan
        reads: {view: loans.Summary}
        from: on_loan
      - name: recent
        component: collection
        reads: {view: loans.All, params: {size: 5}}
        columns: [{field: title}, {field: member}, {field: due}]
```

`sections` is a list, and each section carries its `name`; the list order is the order on the page.
A `metric` shows one number, taken `from` a field of the first row of its view, with `label` as its
caption.
A `collection` shows rows; `params` are fixed parameters of the read. A section has no title of its
own in `ess-ui/1` (requested in [beyond10x/ess#281](https://github.com/beyond10x/ess/issues/281)).

The page's `kind` is a template whose sections every page of that kind gets. `dashboard_page`
contributes a `board` section, and a board must read a dashboard record, which the library model
does not have. `{name: board, remove: true}` takes the inherited `board` out of this page; the
metric and the list are the page's own.

## A list page with a drawer

```yaml
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {view: loans.All, paging: server}
        columns: [{field: title}, {field: member}, {field: due}, {field: state, as: badge}]
        row_actions: [{opens: edit, label: Extend}]
    overlays:
      edit:
        kind: drawer
        component: form
        title: Extend loan
        does: loans.ExtendLoan
        fields: [due]
```

`list_page` contributes the page state `search`, `page`, `size` and `sort`, a header showing the
title and the total of `list`, and two sections: `filters`, a `filter_bar` bound to `state.search`,
and `list`, a `collection`. The page's own `list` is merged over the kind's by name; `filters` is
inherited: the canvas draws it with a dashed outline and the tree marks it **inherited**.

`as: badge` renders a column as a badge. `row_actions` puts an **Extend** action on every row that
`opens` the overlay `edit`. The overlay is a `drawer` holding a `form` whose submit runs the ESS
command `loans.ExtendLoan` with the field `due`. The `opens_resolves` check holds that `edit` exists
on this page, its kind or its shell.

## The last page

```yaml
  members:
    kind: list_page
    title: Members
    sections:
      - name: list
        component: collection
        reads: {view: members.All}
        columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: badge}]
```

That is the whole document: three pages, four sections and one overlay written by hand, and the
sections the page kinds add.

## What one instruction adds

Selecting `page:overview` and saying *"add a table of overdue loans with title, member and due
date"* asks for one new entry in that page's `sections` list:

```yaml
- name: overdue
  component: collection
  reads:
    view: loans.All
    params:
      state: overdue
  columns:
    - field: title
    - field: member
    - field: due
```

The read reuses the existing view `loans.All` with a fixed `state` parameter — its rows carry a
`state` field — rather than inventing a new view.

## How uilab relates to ESS

[ESS](https://github.com/beyond10x/ess) — executable system specifications — is the beyond10x
toolchain for describing systems as typed, validated data and deriving contracts, code and
conformance suites from them. Its [documentation](https://beyond10x.github.io/ess/) and
[crates](https://github.com/beyond10x/ess/tree/main/crates) live in its own repository. uilab
touches it in three places:

| Where | What |
|---|---|
| The document | `ess-ui/1` is ESS's UI specification format, released with ess 0.47.0; uilab follows ess 0.48.0. ESS's own loader expands a document and its checker decides what it may hold; uilab adds node paths and patches on top and writes only what ESS reads. A document's views and commands are names in an ESS model. The [reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) is generated from the format's schema. |
| uilab itself | uilab's session — documents, selection, proposals and who may accept them — and its browser↔server messages are specified in ESS in the repository's `ess/` directory. The Rust and TypeScript types and the conformance suite are generated from that specification, and the build fails when they drift. |
| Placeholders | a read written as `reads: {placeholder, fixture}` is a request to the ESS model: the UI needs this data, and the model does not have it yet. |

You do not need ESS installed to run uilab over a document. You need it to change uilab's own
specification, and `ess ui check --path <file>` checks a document the same way uilab does.
