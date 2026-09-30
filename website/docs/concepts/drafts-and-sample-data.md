---
title: Drafts and sample data
sidebar_position: 6
description: Fixtures make the canvas render without a backend; draft views mark data the model does not provide yet.
---

# Drafts and sample data

The canvas renders without a backend. Every composite that shows data reads an ESS **view** by
name, and uilab fills it from sample rows instead of a running service.

## Fixtures

The document's `fixtures` index maps each view to a YAML file next to it:

```yaml
fixtures:
  dir: fixtures
  views:
    loans.All: loans.yaml
    loans.Summary: loans.yaml
    members.All: members.yaml
```

```yaml title="fixtures/loans.yaml"
views:
  loans.All:
    total: 4
    rows:
      - id: l-1
        title: The Left Hand of Darkness
        member: Robin Example
        due: 2026-10-14
        state: on_loan
      - id: l-2
        title: A Pattern Language
        member: Kim Sample
        due: 2026-10-02
        state: overdue
  loans.Summary:
    rows:
      - {on_loan: 4, overdue: 1}
```

A file holds one view (`view:` and `rows:`) or several (`views:`). The field names in these rows
are also what the agent is told each view offers, so a column it proposes names a real field.

:::note[Fixtures are not queries]
The canvas shows a fixture's rows as they are. A `params` filter such as `{state: overdue}` is part
of the specification, but the canvas does not apply it to sample rows.
:::

A view that is read but has no fixture gets a `fixture_per_view` warning.

## Draft views

Sometimes the screen needs data the model does not have yet: "a chart of loans per month" when no
such view exists. The agent is told never to invent a view name. Instead it reads a placeholder:

```yaml
reads: {view: draft.LoansPerMonth}
```

A `draft.` view marks work the data model owes the UI:

- The canvas fills it with made-up sample rows shaped by the fields the composites name — columns,
  form fields, a metric's `from`, a chart's `x` and `series` — so the screen can be reviewed. These
  rows are never data anybody should read as real.
- The `draft_read` warning lists every draft read in the sidebar and the Docs view until the model
  has the view and the read is renamed.
- The Docs view's **Data** table marks each draft as "draft: no model view yet".

That list is the hand-off to whoever owns the backend: the views the agreed screens need.
