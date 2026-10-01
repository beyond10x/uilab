---
title: Drafts and sample data
sidebar_position: 6
description: Fixtures make the canvas render without a backend; placeholder reads mark data the model does not provide yet.
---

# Drafts and sample data

The canvas renders without a backend. Every composite that shows data reads an ESS **view** by
name, and uilab fills it from sample rows instead of a running service. Both mechanisms below are
part of `ess-ui/1`; its [reference](https://beyond10x.github.io/ess/docs/reference/ess-ui)
describes them under *Reads, actions and fixtures*.

## Fixtures

The document's `fixtures` names a directory and an index file; the index maps each view to a YAML
file in that directory:

```yaml
fixtures: {dir: fixtures, index: fixtures/index.yaml}
```

```yaml title="fixtures/index.yaml"
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

## Placeholder reads

Sometimes the screen needs data the model does not have yet: "a chart of loans per month" when no
such view exists. The agent is told never to invent a view name. Instead the read names a
placeholder and the fixture file that answers it:

```yaml
- name: per_month
  component: chart
  chart: bar
  reads: {placeholder: loans.PerMonth, fixture: fixtures/loans-per-month.yaml}
  x: month
  series: [loans]
```

A placeholder read marks work the data model owes the UI:

- When the fixture file exists, the canvas shows its rows. When it does not, the canvas fills the
  read with made-up rows shaped by the fields the composites name — columns, form fields, a
  metric's `from`, a chart's `x` and `series` — marked as samples, so the screen can be reviewed.
  These rows are never data anybody should read as real.
- ESS reports every placeholder with the `unbound_placeholder` warning, listed in the sidebar and
  the Docs view, until the model has the view and the read is changed to `reads: {view: …}`.

That list is the hand-off to whoever owns the backend: the views the agreed screens need.
