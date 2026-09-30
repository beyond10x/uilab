---
title: FAQ
sidebar_position: 1
description: Short answers to the questions front-end teams ask first.
---

# FAQ

## Does uilab generate React or Vue code?

No. It edits a specification. The canvas is uilab's own renderer of that specification, and your
team implements the screens in its own stack. See
[Adopting uilab](./adopting.md#what-stays-hand-written).

## Can I use my own design system on the canvas?

Not today. The canvas renders the 14 built-in composite kinds, the nine primitives and your declared
widgets with uilab's generic look. `ui-spec/1` has no style tokens yet. Map the kinds and widgets to
your components when you implement.

## Does anything change without me accepting it?

Not with the default settings. A proposal changes nothing until you press `Enter` (or run
`uilab op accept`). The exception is `uilab serve --auto-apply`, which you have to ask for.

## What does the agent see? Does my code leave the machine?

The agent is sent the instruction, the selected node's YAML and context, the view names and the
field names of their sample rows, and the declared widgets. Not your source code, and not the
values in your fixture rows. The run has no tools. Details in
[Working with the agent](./working-with-the-agent.md#what-the-agent-sees).

## Do I need a GPU?

Only for speech at full speed. Start the server with `--no-stt` and type instructions, or with
`--stt-cpu` to transcribe on the CPU. The default build still compiles the speech crate, so the
build machine needs CMake, a C++ compiler and the Vulkan development files.

## Which model does the agent use?

A Claude model through your Claude login by default. `--model` and `--base-url` point it at
another model or Messages-compatible endpoint.

## Why did the agent refuse, or propose something odd?

A proposal must pass the document checks; when it does not, the agent gets the refusal and one more
try, and a second refusal is reported to you. Odd proposals are usually a selection problem: the
agent works at the selected node, so select the table you mean before you say "add a column". The
session journal (`events.jsonl`) records each instruction, the proposal or refusal, and timings.

## My file got reformatted and lost its comments. Why?

uilab writes the whole document back in one canonical YAML layout on every accepted change, and
comments do not survive. Commit that normalisation once, on its own, so later diffs are small.

## Can several people work in one session?

Yes. Every browser connected to the same server shares the document, the selection and the
proposal on screen. Add `?name=<you>` to the address to set your name in the presence strip.
Scripts and coding agents join with `uilab op`.

## Can I script it?

`uilab op` drives a running server from a shell: `join`, `select`, `say`, `goal`, `accept`,
`reject`, `undo`, `state`, and `eval` for running an instruction suite and writing a report. The
server also exposes `/api/document.yaml`, `/api/docs.md` and `/api/help.md`.

## Where is the specification format defined?

`ui-spec/1` belongs to [ESS](https://github.com/beyond10x/ess) and is still being settled there.
uilab reads the subset it edits; [The specification](./specification.md) walks through it.

## Is uilab ready for production use?

It is a working tool under active development; `ui-spec/1` itself is not released yet. Treat
the specification it writes as a design artefact that your team reviews, not as an input your build
depends on.
