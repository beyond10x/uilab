---
slug: /
title: Introduction
sidebar_position: 1
description: What uilab is, who it is for, and what it does and does not do.
---

# Introduction

uilab is a browser workbench for writing a UI specification together with an agent. It opens one
`ess-ui/1` document — a YAML file that describes an application's shell, menu, pages, sections and
overlays — and shows it two ways at once: as a rendered canvas on the left and as a tree on the
right. You select a node, say or type what you want there, and the agent answers with **one
proposed change** at that node. You see the change as a diff and as a highlighted preview on the
canvas, and nothing is written to the file until you accept it. The format belongs to ESS, whose
[reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) defines it, and ESS's own checker
decides what a document may hold.

![uilab with a proposal waiting for review: the diff in the sidebar, the new section highlighted on the canvas](/img/screens/proposal.png)

## Why it exists

Most UI work starts with a conversation about *what* the screen should show: which list, which
columns, which action opens which drawer. That conversation usually ends up in a ticket, a mock-up
and a component tree that drift apart. uilab keeps the answer in one reviewable text file:

- The **specification** is the record. It is plain YAML in your repository, reviewed like code.
- The **canvas** renders it with sample data, so the conversation happens in front of a working
  screen rather than a description of one.
- The **agent** does the typing. It proposes a change at the node you point at; you decide.

## Who it is for

uilab is written for front-end teams that want to agree on screens before they build them, and for
the people who work with them — product, design, domain experts — who can point at a page and say
"also show the member here" without learning the file format.

It does not replace your component library, your design system or your framework. See
[Adopting uilab in a front-end team](./adopting.md) for where it sits next to them.

## What it does today

| Verb | What uilab does |
|---|---|
| Edit | one `ess-ui/1` document per server: shells, regions, menu, pages, sections, overlays, widgets |
| Render | a canvas of the 12 composite kinds, the header, overlays and 9 primitives, with the sections a page kind contributes marked as inherited, fed by fixture rows or made-up sample rows |
| Propose | insert, replace, remove, or a batch of those, at the selected node; checked before you see it |
| Decide | accept (Enter), reject (Esc), undo (Ctrl+Z); accepted changes are written back to the file |
| Plan | goals: a typed request the agent splits into steps (8 at most unless the goal sets another cap), proposed one at a time |
| Listen | optional speech input, transcribed on your own machine by whisper.cpp |
| Explain | a generated documentation view and a help view derived from the document and the code |

## What it does not do

- It does not generate React, Vue or any other framework code. The canvas is uilab's own
  renderer of the specification, not your application.
- It does not change your data model. Views and commands are referenced by name; when a view does
  not exist yet the agent writes a placeholder read, `reads: {placeholder, fixture}`, and the
  document says so.
- It does not act without you. The agent has no tools, cannot read your source tree and cannot
  write files. The server writes the document only when a proposal is accepted — unless you start
  it with `--auto-apply`, which you choose explicitly.

## Where to go next

- [Getting started](./getting-started.md) — run uilab over the example lending-library app.
- [Concepts](./concepts/the-document.md) — the document, nodes, proposals, widgets, goals, drafts.
- [Working with the agent](./working-with-the-agent.md) — how to phrase instructions, and the limits.
- [The specification](./specification.md) — `ess-ui/1`, read line by line.
- [FAQ](./faq.md)
