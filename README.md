# Zed Quarkdown

A [Quarkdown](https://quarkdown.com/) extension for [Zed](https://zed.dev/).

- **Syntax highlighting**: [Tree-sitter Quarkdown by Kinten Le](https://github.com/kinten108101/tree-sitter-quarkdown) (block + inline grammars, with HTML/LaTeX injection)
- **Language server**: completions, documentation on hover, diagnostics and semantic highlighting of function calls — powered by `quarkdown language-server` (auto-installed, see [below](#language-server))
- **Live preview**: compile + serve + watch the current document, with automatic reload in your browser
- **Exports**: HTML, Markdown and PDF — one task away

## Live preview

The preview runs `quarkdown c <file> --preview --watch`: Quarkdown compiles your
document, serves it on a local server (`http://127.0.0.1:8089/live`), opens it
in your browser and **reloads it automatically on every change** (scroll position
is preserved).

You can start it two ways:

1. **Task picker**: open the task modal (`task: spawn`) and choose `Quarkdown preview`.
2. **Keybinding** (recommended, mirrors VS Code's `Ctrl/Cmd+Shift+V`): add this to
   your [`keymap.json`](https://zed.dev/docs/key-bindings) (`zed: open keymap`):

   ```json
   {
     "context": "Workspace",
     "bindings": {
       "ctrl-shift-v": ["task::Spawn", { "task_name": "Quarkdown preview", "reveal_target": "center" }]
     }
   }
   ```

Once running:

- **Refresh** happens on file change — no need to rerun anything.
- **Stop** the preview: kill the task in the terminal pane, or rerun/replace it
  by spawning the task again (`task: rerun`).
- **Refresh while you type**: enable autosave so changes hit the disk (and the
  watcher) immediately — same trick as the official VS Code extension:

  ```json
  { "autosave": { "after_delay": { "milliseconds": 200 } } }
  ```

> **Requirements**: `quarkdown` must be installed and on your `PATH`
> ([install guide](https://github.com/iamgio/quarkdown#getting-started)).
> The tasks need the full CLI, which is what the preview/export features are built on.

## Exports (HTML, Markdown, PDF)

Three export tasks ship with the extension. Run them from the task picker
(`task: spawn`) while a `.qd` file is active, or bind them to keys:

| Task | Runs | Output |
| --- | --- | --- |
| `Quarkdown export HTML` | `quarkdown c <file> --render=html` | `quarkdown-output/<name>/` |
| `Quarkdown export Markdown` | `quarkdown c <file> --render=markdown` | `quarkdown-output/<name>.md` |
| `Quarkdown export PDF` | `quarkdown c <file> --pdf` | `quarkdown-output/<name>.pdf` |

```json
{
  "context": "Workspace",
  "bindings": {
    "ctrl-alt-h": ["task::Spawn", { "task_name": "Quarkdown export HTML" }],
    "ctrl-alt-m": ["task::Spawn", { "task_name": "Quarkdown export Markdown" }],
    "ctrl-alt-p": ["task::Spawn", { "task_name": "Quarkdown export PDF" }]
  }
}
```

The PDF export uses a headless browser, so it may take a few seconds; the
resulting path is printed in the task output (the terminal tab closes
automatically on success).

> **Why no right-click → "Export…"?** Zed's extension API cannot contribute
> items to context menus (neither in the file tree nor in the editor) — the
> `task: spawn` picker and keybindings are the supported entry points today.

## Language server

Quarkdown's [language server](https://github.com/iamgio/quarkdown) (Quarkdown ≥ 1.9)
provides, out of the box:

| Feature | VS Code equivalent |
| --- | --- |
| Completions for functions, parameters and values | ✅ same |
| Documentation on hover and in the completion menu | ✅ same |
| Diagnostics (unknown parameter names, invalid values, duplicates…) | ✅ same |
| Semantic highlighting of function calls (`function.name {arg}`) | ✅ same |

The binary is looked up in this order:

1. Your `PATH` (`which quarkdown`)
2. `lsp.quarkdown.binary.path` in your Zed settings (overrides everything)
3. Automatically downloaded from the [latest Quarkdown release](https://github.com/iamgio/quarkdown/releases)

So completions/hover/diagnostics work even without installing Quarkdown yourself —
only the preview/export tasks require a `PATH` installation.

Restart the language server from the command palette after upgrading Quarkdown.

### Settings

```json
{
  "lsp": {
    "quarkdown": {
      "binary": {
        "path": "/opt/quarkdown/bin/quarkdown",
        "arguments": ["language-server"],
        "env": {}
      },
      "initialization_options": {},
      "settings": {}
    }
  }
}
```

## Customizing compiler options

The tasks use Quarkdown's default output directory (`quarkdown-output`, next to
the source file) and no extra flags. To change that (e.g. `--allow network`,
a custom `--out`, or a non-default server port), copy the task into a project
[`tasks.json`](https://zed.dev/docs/tasks) (`.zed/tasks.json` via
`zed: open project tasks`):

```json
[
  {
    "label": "Quarkdown preview (custom)",
    "command": "quarkdown",
    "args": ["c", "$ZED_FILENAME", "--preview", "--watch", "--server-port", "8099", "--allow", "network"],
    "cwd": "$ZED_DIRNAME",
    "save": "current",
    "reveal": "no_focus"
  }
]
```

Worktree-local tasks are listed first in the task picker, so a customized copy
in `.zed/tasks.json` sits right above the one shipped by the extension.

## Why no preview pane inside Zed?

Unlike VS Code, Zed's extension API does not (yet) allow extensions to render
custom views or webviews inside the editor — see
[zed-industries/zed#21208](https://github.com/zed-industries/zed/issues/21208).
The browser-based live preview is the closest possible experience today: it
updates in real time and preserves your scroll position. If an in-editor pane
matters to you, consider upvoting that issue.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions)
section of the Zed docs. In short: install the `wasm32-wasip2` Rust target, then
use `zed: install dev extension` and select this directory.

## Publishing a new version

To publish a new version of this extension:

1. Bump the version number in `extension.toml` and `Cargo.toml`
2. Open a PR to the [zed-industries/extensions](https://github.com/zed-industries/extensions) repository following [these steps](https://zed.dev/docs/extensions/developing-extensions#updating-an-extension)
