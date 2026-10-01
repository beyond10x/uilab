---
title: Getting started
sidebar_position: 2
description: Build uilab, open the lending-library example, and make your first reviewed change. Speech is optional.
---

# Getting started

This page takes you from a clone to a first accepted change in the example lending-library app.
Speech input is optional; everything works with typed instructions.

## What you need

| Requirement | Needed for | Notes |
|---|---|---|
| Rust (stable) and [`task`](https://taskfile.dev) | building and running the server | |
| Node 22 with pnpm | building the browser app | the server serves the built files |
| CMake, a C++ compiler and the Vulkan development files | building the speech crate | compiled by the default build even when you run without speech |
| A Claude login | the agent | read from `~/.claude/.credentials.json`; `--model` and `--base-url` point it elsewhere |
| A GPU and the speech model (about 1.6 GB) | speech only | `task model` downloads it; `--stt-cpu` runs speech on the CPU |

uilab runs on your machine. It listens on `127.0.0.1` by default and has no accounts or hosted
component.

## Run the example

```console
git clone https://github.com/beyond10x/uilab
cd uilab
# build the browser app (widget/dist)
task widget
cargo run --release -p uilab-app -- serve \
  --doc examples/library/library.ui.yaml --no-stt
```

Open `http://127.0.0.1:8740`. With speech:

```console
# download the whisper model, 1.6 GB
task model
# build the app and serve examples/library
task run
# or serve your own document
task run -- path/to/your.ui.yaml
```

:::tip[Work on a copy]
Accepted changes are written back to the file you pass with `--doc`. Point it at a copy, or at a
file under version control, while you try things.
:::

## The screen

![The canvas on the left renders the document; the sidebar holds the talk button, the instruction field and the tree](/img/screens/canvas.png)

- **Left: the canvas.** The document rendered with sample rows. Click anything to select it. A
  section the page's kind contributes, such as the Loans page's `filters`, is drawn with a dashed
  outline and marked **inherited** in the tree.
- **Top: four views.** `1` UI, `2` YAML, `3` Docs, `4` Components. **Export YAML** downloads the
  current document.
- **Structure or preview.** On the UI tab, `p` (or the **Preview** button) switches the canvas
  between structure, with each node labelled by name, kind and view, and a preview drawn as the
  app would look. Selection and proposal marks work in both.
- **Right: the sidebar.** The talk button (or hold `Space`), the instruction field with its
  **instruction / goal** toggle, the document tree, the selected path, and the activity feed.
- **Top right: who is here.** Every browser and every scripted operator connected to the server.
  Add `?name=Robin` to the address to choose the name you appear under.

## Your first change

1. Click **Overview** in the tree (or its title on the canvas). The selected path reads
   `page:overview`.
2. Type into the instruction field and press `Enter`:

   ```text
   add a table of overdue loans with title, member and due date
   ```

3. The agent thinks for a few seconds, then shows its proposal: an `INSERT` card with the diff in
   the sidebar, the new section outlined in green on the canvas, and the new node in the tree.

   ![A proposal waiting for review](/img/screens/proposal.png)

4. Press `Enter` to accept, or `Esc` to reject. After accepting,
   `Ctrl`+`Z` undoes it.
5. Press `2` to see the YAML. The new section is there, and so is the file on disk.

![The YAML view, with the selected node highlighted](/img/screens/yaml.png)

## Keys

| Key | Does |
|---|---|
| `Space` (hold) | speak an instruction |
| `Enter` | accept the proposal |
| `Esc` | reject the proposal, or close an overlay or the help |
| `Ctrl`+`Z` | undo the last accepted change |
| `?` | open the help |
| `1` `2` `3` `4` | UI, YAML, Docs, Components |
| `p` | on the UI tab: switch the canvas between structure (labels) and preview (the app) |

While a text field has focus, none of these keys acts on the workbench, `Esc` included: they go to
the field, and `Enter` in the instruction field sends the instruction. Click the canvas or the tree
to leave the field first. Two exceptions: open help closes on `Esc` or `?` wherever the focus is,
and while you edit your name at the top right, `Enter` keeps it and `Esc` cancels the edit.

## Server options

`uilab serve --help` lists them all. The ones you will reach for:

| Option | Default | What it does |
|---|---|---|
| `--doc <file>` | (required) | the document to edit; accepted changes are written back to it |
| `--listen <addr>` | `127.0.0.1:8740` | where the server listens |
| `--no-stt` | off | no speech; instructions are typed |
| `--stt-cpu` | off | run speech on the CPU |
| `--language <code>` | detected | the spoken language, for example `en` or `de` |
| `--auto-apply` | off | apply every proposal at once instead of waiting for accept or reject |
| `--journal <dir>` | `~/.cache/uilab/sessions` | where each run keeps `events.jsonl` and one WAV per utterance |
| `--model`, `--base-url` | built in | the model and the Messages endpoint the agent uses |

## Next

- Read [the document](./concepts/the-document.md) to understand what you just edited.
- Read [working with the agent](./working-with-the-agent.md) before you try it on a real screen.
