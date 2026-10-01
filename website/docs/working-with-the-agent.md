---
title: Working with the agent
sidebar_position: 1
description: How to phrase instructions, what the agent sees, and exactly what it can and cannot change.
---

# Working with the agent

The agent is deliberately narrow. It takes one instruction at one node and returns one proposal.
Knowing what it sees and what it may answer makes it predictable.

## Good instructions

Point first, then say what you want **at that node**. Short, concrete instructions work best:

| Selected | Instruction |
|---|---|
| `page:overview` | "add a table of overdue loans with title, member and due date" |
| `page:loans/section:list` | "also show the member in this table" |
| `page:members` | "add a drawer to add a new member with name and email" |
| `page:loans/section:list` | "add an edit action on each row that opens a drawer" |
| `nav` | "add a page for reservations in the circulation menu" |
| any node | "remove this" |

- Say **what** and **which fields**, not how it should look. Layout and styling are not part of
  the specification.
- To change the selected node itself — "also show X", "rename this", "add a column" — keep it
  selected. The agent replaces that node and must keep everything you did not mention.
- When the work spans several changes, give a [goal](./concepts/goals.md) instead.

## Speech

Hold `Space` (or the talk button) and speak. Speech is transcribed on your machine by whisper.cpp
(`large-v3-turbo`), prompted with the names valid at the selected node, so page and field names
are recognised. The transcription is the instruction; it is shown with the proposal it produced.
Start the server with `--language en` (or another code) to skip language detection.

## What the agent sees

For each instruction the agent is sent:

- the instruction text;
- the selected node's path, kind and YAML, its ancestors and its existing children;
- which child layers and composite kinds are allowed there;
- the views the document reads and the **field names** of their sample rows — not the row values;
- the widgets the document declares;
- for a page or the root, the existing sections of the pages the instruction names.

It is not sent your source code, and it cannot fetch anything: the agent run has **no tools**.

## What the agent can answer

Its only way to finish is an answer that matches the JSON Schema of what the selected node can
hold: an `insert`, `replace`, `remove` or `batch`. The field shapes in that schema come from ESS's
own schema for `ess-ui/1` (see its [reference](https://beyond10x.github.io/ess/docs/reference/ess-ui)),
so the agent can only write what the format declares — no section `title`, a `note` on every widget
param. The answer is validated against that schema, then admitted against ESS's document checks. If
a check refuses it, the refusal is fed back once; a second refusal is reported to you, and nothing
changes.

## What it cannot change

| It cannot | Because |
|---|---|
| write the file | only an accepted proposal is written, by the server (unless you chose `--auto-apply`) |
| change more than it shows you | the proposal is the whole change; a `batch` lists each of its patches with its own target, and they are checked together |
| invent a data source | it must use a view the document reads, or a placeholder read (`reads: {placeholder, fixture}`) that stays flagged |
| run commands or read files | the run has no tools |
| accept its own proposal | accept, reject and undo belong to the operator in the ESS session specification |

## The help view

Press `?` for the keys and **What uilab can do**: the goals and Components tab rules, where things
can go, the composite kinds and primitives, and the example instructions above. It is generated
from the same tables the checks and the answer schema use, so it matches the build you run.

![The help view](/img/screens/help.png)

## Which model

The agent runs through the beyond10x harness loop with a Claude model and your Claude login by
default. `--model` names another model and `--base-url` another Messages-compatible endpoint.
