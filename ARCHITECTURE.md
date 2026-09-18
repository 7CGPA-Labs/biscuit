# System Architecture: Biscuit (formerly ClippyText)

This document outlines the technical architecture of the Biscuit application in its current implemented state.

## 1. High-Level Architecture
Biscuit is a hybrid application combining a native Rust/GTK4 backend with a statically bundled JavaScript frontend for text rendering, and a local CPU-bound AI engine powered by Candle for intelligent features.

### Core Stack
- **Backend & GUI:** Rust 2021, GTK4 (`gtk4-rs`), Libadwaita (`libadwaita-rs`).
- **Text Editor Component:** GtkSourceView 5 (`sourceview5`).
- **Rendering Frontend:** Vite, Node.js, `markdown-it`, `latex.js` (bundled at compile time).
- **Embedded Web Engine:** WebKitGTK (`webkit6`).
- **AI Engine:** Candle (`candle-core`, `candle-transformers`).
- **Build System:** Cargo (with heavy use of `build.rs` for asset downloading and frontend compilation).

---

## 2. Component Details

### 2.1. The Rust Backend & Native GUI (`src/`)
The main application is driven by a GTK4 event loop initialized via Libadwaita. 
- **`src/main.rs` & `src/ui/`:** Handles window lifecycle, theme management, menus, and user actions. Sets environment variables (like `LIBGL_ALWAYS_SOFTWARE`) to ensure stable rendering across sandboxed Linux environments.
- **`src/editor/`:** Wraps the `sourceview5::View` and `sourceview5::Buffer`. It manages keyboard input, text retrieval, and triggers debounced updates to the preview pane.

### 2.2. The WebView Rendering Engine (`webview-src/` & `src/preview.rs`)
To accurately render complex Markdown and LaTeX without reinventing the wheel in Rust, the application relies on an embedded WebKit6 WebView.
- **Frontend Build (`webview-src/`):** A Vite project utilizing `vite-plugin-singlefile`. It takes dependencies like `markdown-it` and `latex.js` and compiles them into a single, massive `index.html` file containing all CSS and JS inline.
- **Compile-time Injection:** `build.rs` executes `npm run build` during the Cargo build process. `src/preview.rs` then uses the `include_str!` macro to bake the resulting `index.html` directly into the Rust binary.
- **Runtime Execution:** The `PdfPreview` struct initializes the WebView and loads the baked HTML using `webview.load_html` with a custom `http://localhost:17539/` base URI to satisfy WebKit module CSP requirements while avoiding the need for an actual local HTTP server.
- **Latex Hardening:** Rust escapes text and passes it to the frontend via `webview.evaluate_javascript`. The frontend aggressively scrubs YAML frontmatter and `\documentclass`/`\usepackage` macros to prevent the `latex.js` body parser from throwing fatal exceptions.

### 2.3. Offline Document Export Pipeline
Biscuit leverages external, statically linked binaries to convert Markdown/LaTeX into distributable formats (`.pdf` and `.docx`).
- **Asset Fetching:** `build.rs` downloads pre-compiled releases of `pandoc`, `typst`, and `tectonic` directly from GitHub into the `OUT_DIR` during compilation.
- **Subprocess Execution:** When a user clicks "Export", `src/preview.rs` spawns a background thread using `std::process::Command` to pipe the raw text directly into the stdin of the embedded binaries.
- **UI Feedback:** A modal `gtk::Window` with a `gtk::ProgressBar` uses a glib timeout loop to monitor a `std::sync::mpsc::channel` connected to the export thread, preventing UI freezes.

### 2.4. Local AI Engine & Assistant UI (`src/ai/` & `src/clippy/`)
The application features a fully offline AI assistant that interacts directly with the user's text.
- **Model Storage:** Models are downloaded to `~/.local/share/clippytext/models/` by the `ai::downloader` module. We utilize the `SmolLM2-360M-Instruct-Q4_K_M.gguf` model for efficient local execution.
- **Inference Runtime:** `src/ai/worker.rs` and `src/ai/models.rs` utilize the `candle` machine learning framework to load and run quantized models completely on the CPU. It features custom decoding logic, repetition penalties, and few-shot prompting to parse and generate text intelligently.
- **Sprite Animation:** `src/clippy/sprite.rs` manages the visual assistant (Clippy/paperclip sprite). It overlays a `gtk::DrawingArea` onto the text editor and uses `add_tick_callback` to run a 60FPS animation loop across a sprite sheet (`gdk::Texture`).
- **Interactive Assistant:** The assistant tracks the user's mouse and transitions between dynamic states (`IDLE`, `THINKING`, `WRITING`) while background threads generate text completions or fix grammar.

---

## 3. Data Flow

1. **User Types Text:** Input is captured by `sourceview5::View` in the main GTK thread.
2. **Debounce:** A timer waits for the user to pause typing (e.g., 200ms).
3. **Render Trigger:** Rust reads the `Buffer` contents, escapes special characters, and formats a JS string: `window.renderContent('...', is_latex)`.
4. **IPC to WebKit:** `webview.evaluate_javascript()` pushes the payload to the WebView.
5. **Frontend Parsing:** Inside WebKit, `markdown-it` parses the text into DOM nodes, which are injected into the `<div id="app">`, rendering instantly for the user.
