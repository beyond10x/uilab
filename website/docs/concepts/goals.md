---
title: Goals
sidebar_position: 5
description: A typed request the agent plans into steps and proposes one at a time, each waiting for your decision.
---

# Goals

An instruction is one change. A **goal** is a request that takes several: "build out the members
page with a details drawer and a list of their current loans".

## Giving a goal

1. Select the node the goal is about. The agent plans from there.
2. Switch the toggle under the text field from **instruction** to **goal**.
3. Type the goal and press `Enter`.

A goal is always typed. A spoken instruction is always a single instruction.

## What happens

1. **Plan.** The agent breaks the goal into ordered steps — at most 8. Each step is an
   instruction another run can carry out on its own, at a target node that exists or that an
   earlier step creates. The plan is checked before it runs; a plan that breaks a rule is sent back
   once, like a refused proposal.
2. **Propose, one step at a time.** Each step becomes an ordinary proposal at the node it names.
   It waits for your accept or reject like any other proposal. Then the next step follows, on the
   document the earlier steps left.
3. **Watch.** The goal panel in the sidebar lists the steps and where each stands.
4. **Stop** ends the run at any point.

While a goal runs, no other instruction or goal is taken.

When the text is not a request to change the UI — a greeting, a question about what uilab can do —
the planner declines with a one-line reason instead of inventing work.

## Goals on the Components tab

A goal given on the Components tab is planned as widget work: declaring a widget, adding body
nodes, and using the widget where the goal says.

## From a shell

```console
uilab op --as Assistant goal --target page:members --max-steps 4 \
  "add a details drawer for a member, opened from each row"
```

`uilab op goal` waits until the goal is done, stopped or failed (up to 15 minutes) and prints each
step with its status. `--auto` applies each step's proposal at once instead of waiting.
