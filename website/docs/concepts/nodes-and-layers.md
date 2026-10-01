---
title: Nodes and layers
sidebar_position: 2
description: Every part of the document is a node with a stable path; the layer of the selected node decides what can go under it.
---

# Nodes and layers

The tree in the sidebar is the document broken into **nodes**. Every node belongs to a **layer**
and has a **path**. You select a node before you give an instruction, and the agent works at that
node.

## Paths

A path is `/`-separated `layer:name` segments, keyed by name and never by position:

```text
page:loans
page:loans/section:list
page:loans/overlay:edit
shell:app/region:nav
component:member_card
```

Because paths use names, adding a sibling never moves an existing path. The selected path is shown
under the tree, and `uilab op select <path>` selects one from a shell.

## Layers and what they can hold

The layer of the selected node decides what an instruction can add there. This table is the same one
the checks and the agent's answer schema use:

| Selected | Is | Can hold |
|---|---|---|
| root (`/`) | the document | shell, nav, page, component |
| shell | an application frame | region, overlay |
| nav | the menu | nav_section |
| page | a route | section, overlay |
| section | a region of a page | widget, item |
| component | a widget declaration | node |

A **section** is a composite on a page. A **board** section holds `widget` children, each an
ordinary composite; a **collection** or **record** holds named `item` nodes rendered for each row.
Regions and menu sections hold nothing further.

## Composite kinds

A section, an overlay, a board widget or an item holds one member of `ess-ui/1`'s composite union,
named by `component` — or an app-defined [widget](./widgets.md). The union has 12 members; `header`
and `overlay` are composites too, placed by position (a page's `header`, a page's or shell's
`overlays`) rather than by `component`. The
[reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) lists the fields of each.

| Kind | What it is for |
|---|---|
| `collection` | rows of a view as a table, cards, a list or a tree, with columns and row actions |
| `record` | one row shown as labelled fields |
| `form` | inputs bound to a command (`does`) |
| `choice` | a pick from fixed options or from a view |
| `filter_bar` | search, time window and filter inputs above a collection |
| `header` | a page's title, total and actions |
| `overlay` | a drawer, dialog, fullscreen pane or popover holding one composite |
| `confirm` | a confirmation step before a command runs |
| `metric` | one number (`from` a field of the first row) |
| `chart` | a series over time or categories (`x`, `series`) |
| `board` | a grid of widgets, each an ordinary composite |
| `graph_editor` | nodes and edges edited as a whole |
| `rich_text` | formatted text |
| `references` | what uses a record |

An overlay is a `drawer`, `dialog`, `fullscreen` or `popover` holding one composite; a row action
or a button opens it by name with `opens`. A page's `kind` is one of `list_page`, `report_page`,
`detail_page`, `settings_page`, `dashboard_page`, `editor_page`, `form_page` and `static_page`, or
a kind the document declares under `page_kinds`. A kind contributes sections to every page of that
kind — a `list_page` its `filters` and `list` — and the canvas draws the ones a page does not write
itself with a dashed outline, marked **inherited** in the tree (see [The document](./the-document.md#page-kinds-and-inherited-sections)).

## Selecting

- Click a node in the tree, or click it on the canvas.
- The selection is **shared**: every browser connected to the same server sees the same selected
  node, and the sidebar says who selected it.
- A selection by a scripted operator (`uilab op select`) shows up the same way.
