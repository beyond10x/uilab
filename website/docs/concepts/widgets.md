---
title: Widgets
sidebar_position: 4
description: App-defined, reusable composites with typed params and a body, built and previewed on the Components tab.
---

# Widgets

A **widget** is an app-defined composite: a named, reusable piece of UI declared once under
`widgets:` at the root and used wherever a built-in composite kind can go. In front-end terms it
is the specification of a component — its props and its structure — not its implementation.

```yaml
widgets:
  member_card:
    summary: Card showing a member's name, standing badge and a link to the members page
    params:
      member:
        type: Member
        required: true
        note: the member row
    arrange: column
    body:
      - name: name
        primitive: text
        style: heading
        text: args.member.name
      - name: standing
        primitive: badge
        text: args.member.standing
      - name: members_link
        primitive: link
        text: Members
        to: {to: members}
```

A member card with the member's name, their standing as a badge and a link to the members page.
The screenshot on this page shows the agent proposing a smaller one on the Components tab.

## The parts

Widgets are part of `ess-ui/1`; its [reference](https://beyond10x.github.io/ess/docs/reference/ess-ui)
lists every field under *Widget*.

| Key | What it holds |
|---|---|
| `summary` | one line, shown in pickers and in the generated docs (required) |
| `doc` | optional longer description for authors |
| `params` | typed parameters; each has a `type` and a one-line `note`, and may be `required` and have a `default` |
| `arrange` | how the body is laid out: `column` (the default), `row` or `grid` |
| `body` | a list of named nodes in order: composites, other widget instances, or primitives |

Inside the body, `args.<param>` refers to a parameter: `text: args.member.name`.

## Primitives

A body node — or a node of a collection's item list — can be one of nine primitives:

| Primitive | What it is |
|---|---|
| `text` | a caption, heading or formatted value (`text` or `field`) |
| `badge` | a short value in a toned pill (`tone` or `tone_by`) |
| `icon` | a semantic icon with an accessible `label` |
| `button` | a button that runs one `action` |
| `link` | text that opens a page (`to`) or an address (`href`) |
| `input` | one free input bound to state (`binds`) |
| `toggle` | an on/off switch bound to state or running an action |
| `image` | an image with required `alt` text |
| `divider` | a visual separator |

## Using a widget

An instance names the widget as its `component` and supplies `args`:

```yaml
name: card
component: member_card
args: {member: row}
```

It goes wherever a composite goes: a section, an overlay, a board's widget, or a collection's item,
where the current row is `row`. ESS expands each instance at its use site — the widget's body with
the args substituted — and checks the result like built-in nodes. The `widget_expands` check holds
that the widget exists, that an instance supplies every required param with an arg of the param's
type, and that no widget contains itself. On the canvas the expanded body nodes of an instance are
marked as inherited; the file keeps only the instance.

## The Components tab

Press `4` (or click **Components**) to see every declared widget with a live preview filled from
sample rows, its summary, its params and where it is used. The tab is a workspace of its own: an
instruction given there builds widgets.

- With a widget or one of its body nodes selected, the agent works under that widget.
- Otherwise it declares a new widget at the root.
- A page left selected on the canvas does not count on this tab, and neither does a board's widget,
  which is part of a page.
- `1` returns to the UI.

![The Components tab with a new widget proposed: its preview on the left, the diff on the right](/img/screens/components-proposal.png)
