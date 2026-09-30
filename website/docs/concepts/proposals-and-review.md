---
title: Proposals and review
sidebar_position: 3
description: The agent proposes one checked change at the selected node; you accept, reject or undo it.
---

# Proposals and review

A **proposal** is the agent's answer to one instruction: one change at the selected node. It is the
only thing the agent can produce, and it changes nothing until you accept it.

## The loop

1. **Select** a node in the tree or on the canvas.
2. **Say** what you want there — hold `Space` and speak, or type and press `Enter`.
3. **Review** the proposal: the diff in the sidebar, the change outlined on the canvas and marked in
   the tree.
4. **Accept** with `Enter`, **reject** with `Esc`. Undo an accepted one with `Ctrl`+`Z`.

![A proposal: INSERT at page:overview, the diff, and the new section previewed in green](/img/screens/proposal.png)

## What a proposal can be

| Operation | Does |
|---|---|
| `insert` | adds a new child under the selected node |
| `replace` | replaces the selected node: changes its columns, title, fields or actions |
| `remove` | removes the selected node |
| `batch` | several of those, checked together, when one instruction touches more than one node — a drawer plus the row action that opens it |

## Checked before you see it

A proposal that fails a document check is refused and never shown: names resolve, the menu lists
every page, `opens` names an overlay that exists, columns name fields the view has. When a check
refuses the agent's answer, the refusal is fed back and the agent tries once more; a second refusal
is reported instead of a proposal.

The proposal card lists only the findings the proposal **brings**: those the document did not have
before, and what a `replace` would drop (`replace_drops`). The document's own findings stay in the
sidebar.

## States

A proposal is `Proposed` until you decide. Accepting moves it to `Accepted` and writes the
document; rejecting moves it to `Rejected`. An accepted proposal can be undone, which moves it to
`Undone` and writes the document back. These states and the commands that move between them are
specified in ESS as the `uilab.session` domain: the operator holds accept, reject and undo; the
agent holds only "propose a patch".

## Auto-apply

`uilab serve --auto-apply` applies every proposal as soon as it passes the checks. It is off by
default. Use it for scripted runs, not for review.

## Several people, one session

Every browser connected to a server shares one session: the same document, the same selection, the
same proposal. The presence strip at the top of the sidebar shows who is connected, humans and
agents in different colours. A second person can review what the first person asked for.

`uilab op` operates a running server from a shell as a named operator, which is how scripts and
coding agents take part:

```console
uilab op --as Assistant join
uilab op --as Assistant select page:overview
uilab op --as Assistant say --review \
  "add a table of overdue loans with title, member and due date"
uilab op --as Assistant accept
uilab op --as Assistant state
```

## The journal

Each `uilab serve` run keeps a journal directory with `events.jsonl` — what each operator did, what
speech heard, what the agent proposed or why it was refused, and how long each step took — and
every utterance as a WAV file, so a failure can be read and a transcription replayed afterwards.
The default location is `~/.cache/uilab/sessions`; `--journal` moves it.
