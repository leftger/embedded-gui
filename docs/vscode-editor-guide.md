# VSCode Workflow for embedded-gui KDL

Community feedback from the LVGL thread: UI editors are useful, but a
standalone app is not always the ideal developer experience — a VSCode-style
in-editor workflow is often preferred for firmware teams that already live in
their IDE.

`embedded-gui` already ships:

- **Embedded GUI Studio** (`crates/embedded-gui-studio/`) — a standalone
  desktop/web editor with live KDL preview and `no_std` Rust code generation.
- **A Figma plugin** (`figma-plugin/`) — design-to-KDL for designers.
- **Build-time Rust macros** (`include_gui!` / `gui_kdl!`) — KDL compiles into
  Rust without a separate codegen step.

This guide makes VSCode a first-class editing surface too, so you do not need
to switch tools for small edits.

## 1. Install the project snippets

The repository includes `.vscode/embedded-gui.code-snippets`. Open the
command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and run:

```
Preferences: Configure User Snippets
```

Then choose **Embedded GUI** if the file was loaded from the workspace, or copy
its contents into your user snippets. The `eg-` prefixes provide completions:

| Prefix | Expands to |
|--------|------------|
| `eg-screen` | A complete KDL `screen` |
| `eg-grid` | A KDL `grid` block |
| `eg-button` | A `button` widget |
| `eg-toggle` | A `toggle` widget |
| `eg-scale` | A `scale` widget |

## 2. Edit KDL and generate Rust in VSCode

The `.kdl` files are plain text, so any KDL syntax extension works. This
repository does not yet ship a full language server; for validation and
preview open the same project in Embedded GUI Studio, or run the build-time
macro on your host target:

```bash
cargo check -p your-app --features std,macros
```

A successful check proves the KDL parsed and generated Rust without leaving
your editor.

## 3. Suggested workspace settings

Add this to `.vscode/settings.json` for teams that keep UI sources in a
`ui/` folder:

```json
{
  "files.associations": {
    "*.kdl": "kdl"
  },
  "search.exclude": {
    "**/pkg": true,
    "**/target": true
  }
}
```

If no KDL extension is installed, associate `.kdl` with plain text and rely on
snippets.

## 4. Roadmap toward a native extension

A true VSCode extension would be built on the existing crates rather than a
new editor core:

- `embedded-gui-codegen` already parses KDL and emits Rust.
- `embedded-gui-studio` already has syntax highlighting/parser validation code
  that can be reused in a language server.
- A language server protocol (LSP) crate would let VSCode, Neovim, and other
  editors share diagnostics, hover, and code actions.

Contributions toward an LSP are welcome; the current in-repo workflow is a
pragmatic first step until that exists.
