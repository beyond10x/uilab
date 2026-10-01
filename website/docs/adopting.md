---
title: Adopting uilab in a front-end team
sidebar_position: 3
description: Where uilab sits next to a component library, a design system and code review, and what stays hand-written.
---

# Adopting uilab in a front-end team

uilab is a small addition to a front-end workflow, not a replacement for one. It produces a
reviewed YAML file that says what each screen contains. Everything that turns that into your
product stays where it is today.

## What changes and what does not

| Stays as it is | uilab adds |
|---|---|
| your framework, router and state management | a specification file per application, in the repository |
| your component library and design system | a shared vocabulary: composite kinds, widgets and their params |
| your styling, tokens, accessibility and interaction details | agreement on *which* data, fields and actions each screen shows |
| your API client and backend | a list of the views and commands the screens need, with placeholder reads marking the missing ones |
| code review, CI and releases | a diff of the specification in the same pull request |

The canvas is not your application. It renders the specification with uilab's own generic
components so that people can agree on content and structure. Nothing it draws is meant to ship.

## A workflow that fits

1. **Keep the document in the repository**, next to the code it describes. Commit the first
   normalising write on its own (see [The document](./concepts/the-document.md#read-render-write-back)).
2. **Shape a screen in a session.** A developer runs `uilab serve` over a branch checkout; product,
   design or a domain expert joins in the browser, points and talks. Each accepted change is written
   to the file.
3. **Review the diff like code.** The pull request shows exactly what changed in the screen's
   specification: a new section, a renamed column, a new drawer. Reviewers who were not in the
   session can read it without running anything.
4. **Implement by hand.** The team builds the screen in its own stack from the reviewed
   specification, using its own components.
5. **Hand the data list to the backend.** The Docs view's **Data** table lists every view the
   screens read, and the `unbound_placeholder` warnings list the placeholders that no model view
   answers yet.

## Mapping to your component library

The specification is an `ess-ui/1` document, ESS's renderer-neutral UI format; its
[reference](https://beyond10x.github.io/ess/docs/reference/ess-ui) is the full vocabulary. Its
composite kinds are deliberately generic: `collection`, `record`, `form`, `metric`, `chart` and so
on. Most teams already have a component for each. Write the mapping down once — which
of your components renders a `collection` with `as: tag` columns, which renders a `drawer` — and
the specification becomes a checklist for implementation.

Where your library has something more specific — a member card, a loan status line — declare it as
a [widget](./concepts/widgets.md) with the same name and props. Widgets are the specification's
equivalent of your components: typed params, a body, a summary. The Components tab previews them
and the agent reuses the ones you declare.

## What stays hand-written

- **All production code.** uilab generates no framework code.
- **Look and feel.** The specification has no styles, spacing or tokens.
- **Behaviour beyond structure.** Validation messages, optimistic updates, loading and error states,
  keyboard handling, animation.
- **The data model.** Views and commands are named, not defined, in the document. They belong to the
  backend's own specification.

## Trust and safety

For a team deciding whether to let an agent near its repository:

- The agent proposes; a person accepts. With the default settings nothing is written without an
  explicit accept, and every accepted change can be undone and is visible in `git diff`.
- The agent run has no tools: it cannot read your source tree, run commands or reach the network
  beyond the model call.
- What leaves the machine is the model request: the instruction, the selected node's YAML and
  context, and view **field names**. Fixture row values are not sent. Speech is transcribed locally.
- Every session keeps a local journal of what was asked, proposed, refused, accepted and how long it
  took.

## A cautious first step

Try it where the cost of being wrong is zero: take one existing internal screen, write its
specification (or ask the agent to, one section at a time), and compare the file with what the team
would have written in a ticket. If the file is clearer, keep it next to the code. If it is not,
delete it; nothing else depended on it.
