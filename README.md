# uilab

Build a UI by talking to it. uilab opens an `ess-ui/1` document in the browser: the canvas shows the
application it describes, the sidebar shows its tree. Select a node, hold Space and say what you
want there ("add a table of overdue loans with title, member and due date"). Speech is transcribed
on your own GPU, an agent proposes one change at that node, and you see it as a diff and a preview
before you accept it with Enter. Accepted changes are written back to the file.

## Run

Requires Rust, `task`, Node 22 with pnpm, CMake, a C++ compiler and the Vulkan development files
(the default build compiles whisper.cpp for Vulkan), a GPU with a Vulkan driver for speech at full
speed, and a Claude login (the agent uses the credentials in `~/.claude/.credentials.json`). For
CUDA instead, build with `--no-default-features --features cuda`.

```console
task model                                   # downloads the speech model, about 1.6 GB
task run                                     # serves examples/library on http://127.0.0.1:8740
task run -- path/to/your.ui.yaml             # or your own document
```

Without a GPU or model: `cargo run -p uilab-app -- serve --doc <file> --no-stt` and type the
instructions instead.

## How it fits together

- The **document** is `ess-ui/1`, the UI format ESS publishes
  ([reference](https://beyond10x.github.io/ess/docs/reference/ess-ui)): shells with regions, a
  menu, pages made of named sections, overlays, app-defined widgets, and 12 composite kinds
  (collection, form, record, metric, chart, board, …). ESS's own loader and checker decide what a
  document may hold. Sample rows come from fixture files, so the canvas renders without a backend.
- The **session** is specified in ESS (`ess/`): a proposal is made by the agent and decided by you;
  nothing changes the file until you accept.
- **Speech** is whisper.cpp (`large-v3-turbo`) on the GPU, prompted with the names valid at the
  selected node.
- The **agent** is one run of the beyond10x harness loop whose only output is a patch matching the
  JSON Schema of what the selected node can hold.

Agent-facing detail is in [AGENTS.md](AGENTS.md).
The documentation site is in [website/](website/), built for `https://beyond10x.github.io/uilab/`.
