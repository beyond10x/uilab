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
        to: members
```

This declaration is what the agent proposed for *"a member card widget with the member's name,
their standing as a badge and a link to the members page"*, given on the Components tab.

## The parts

| Key | What it holds |
|---|---|
| `summary` | one line, shown in pickers and in the generated docs |
| `doc` | optional longer description for authors |
| `params` | typed parameters; each has a `type`, and may be `required`, have a `default` and a `note` |
| `arrange` | how the body is laid out: `column` (the default), `row` or `grid` |
| `body` | named nodes in order: composites, other widget instances, or primitives |

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
component: member_card
args: {member: row}
```

It goes wherever a composite goes: a section, an overlay, a board's widget, or a collection's item,
where the current row is `row`. The checks hold that an instance supplies every required param and
no undeclared one (`widget_args`), that a widget is never named like a built-in kind
(`widget_named_like_builtin`) and that no widget contains itself (`widget_recursion`).

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
