---
title: The specification
sidebar_position: 2
description: ui-spec/1 read line by line from the lending-library example, and how uilab relates to ESS.
---

# The specification

This page reads `examples/library/library.ui.yaml` — the lending-library app uilab ships as its
example — from top to bottom. It is short enough to read whole, and it uses most of what a
list-and-detail back-office screen needs.

## Header

```yaml
format: ui-spec/1
app: library
title: Lending library
model: library
placement_profile: fat
```

`format` marks the file. `app` and `title` name the application. `model` names the ESS model whose
views (`loans.All`) and commands (`loans.ExtendLoan`) the document refers to. `placement_profile`
is carried through from `ui-spec/1`; uilab shows it in the Docs view.

## Sample data

```yaml
fixtures:
  dir: fixtures
  views:
    loans.All: loans.yaml
    loans.Summary: loans.yaml
    members.All: members.yaml
    staff.Me: staff.yaml
```

Each view the document reads gets rows from a fixture file, so the canvas renders without a
backend. See [Drafts and sample data](./concepts/drafts-and-sample-data.md).

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
notifications and the account menu go. Pages render into the `page_outlet`; a shell a page renders
in must have one.

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

## A dashboard page

```yaml
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      on_loan:
        component: metric
        title: Copies on loan
        reads: {view: loans.Summary}
        from: on_loan
      recent:
        component: collection
        title: Recent loans
        reads: {view: loans.All, params: {size: 5}}
        columns: [{field: title}, {field: member}, {field: due}]
```

Sections are named and appear in layout order. A `metric` shows one number, taken `from` a field of
the first row of its view. A `collection` shows rows; `params` are fixed parameters of the read.

## A list page with a drawer

```yaml
  loans:
    kind: list_page
    title: Loans
    sections:
      list:
        component: collection
        reads: {view: loans.All, paging: server}
        columns: [{field: title}, {field: member}, {field: due}, {field: state, as: tag}]
        row_actions: [{opens: edit, label: Extend}]
    overlays:
      edit:
        kind: drawer
        component: form
        title: Extend loan
        does: loans.ExtendLoan
        fields: [due]
```

`as: tag` renders a column as a tag. `row_actions` puts an **Extend** action on every row that
`opens` the overlay `edit`. The overlay is a `drawer` holding a `form` whose submit runs the ESS
command `loans.ExtendLoan` with the field `due`. The `opens_resolves` check holds that `edit` exists
on this page or its shell.

## The last page

```yaml
  members:
    kind: list_page
    title: Members
    sections:
      list:
        component: collection
        reads: {view: members.All}
        columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: tag}]
```

That is the whole document: 79 lines, three pages, five composites.

## What one instruction adds

Selecting `page:overview` and saying *"add a table of overdue loans with title, member and due
date"* produced this proposal, which was accepted as is:

```yaml
overdue:
  component: collection
  reads:
    view: loans.All
    params:
      state: overdue
  title: Overdue loans
  columns:
    - field: title
    - field: member
    - field: due
```

The agent reused the existing view `loans.All` with a fixed `state` parameter — its rows carry a
`state` field — rather than inventing a new view.

## How uilab relates to ESS

[ESS](https://github.com/beyond10x/ess) — executable system specifications — is the beyond10x
toolchain for describing systems as typed, validated data and deriving contracts, code and
conformance suites from them. Its [documentation](https://beyond10x.github.io/ess/) and
[crates](https://github.com/beyond10x/ess/tree/main/crates) live in its own repository. uilab
touches it in three places:

| Where | What |
|---|---|
| The document | `ui-spec/1` is ESS's UI specification format. A document's views and commands are names in an ESS model. The format is still being settled in ESS and is not part of a released ESS yet; uilab reads the subset it edits with its own reader, `crates/uilab-doc`. |
| uilab itself | uilab's session — documents, selection, proposals and who may accept them — and its browser↔server messages are specified in ESS in the repository's `ess/` directory. The Rust and TypeScript types and the conformance suite are generated from that specification, and the build fails when they drift. |
| Drafts | a `draft.` view is a request to the ESS model: the UI needs this data, and the model does not have it yet. |

You do not need ESS installed to run uilab over a document. You need it to change uilab's own
specification.
