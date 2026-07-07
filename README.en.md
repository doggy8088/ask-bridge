# Ask Bridge 🦀

`ask` is a powerful, lightweight command-line tool written in **Rust** that automates ChatGPT or Gemini directly in your real Chrome browser. It uses the **Model Context Protocol (MCP)** and **Chrome DevTools Protocol (CDP)** via the embedded `doggy8088/mcp-cli` Rust library dependency and `chrome-devtools-mcp` to control Chrome, input prompts, click submit, and print the response back to your terminal. ChatGPT is the default provider; use `--provider gemini` to switch to Gemini.

Unlike typical API clients, `ask` operates inside a real Chrome browser with a **persistent user profile**. This means:
- You log in manually **once** (`ask login`).
- You can solve CAPTCHAs, pass provider-side browser checks, and access the selected provider's web features like a normal user.
- Your session cookies, login state, and chat history are saved persistently.

---

## 🌟 Key Features

- **🦀 100% Rust Core**: Extremely fast, lightweight, and compile-once, run-anywhere binary.
- **Multi-provider support**: Choose ChatGPT or Gemini with `--provider chatgpt|gemini`.
- **🌐 Real Browser Automation**: Directly interacts with Chrome on port `9223` (isolated debug profile).
- **🔒 Persistent Login**: Uses a dedicated local profile directory (`~/.config/ask-bridge/chrome-profile`) so you never lose your login state.
- **Response Output**: Prints the selected provider's response back to your terminal.
- **🌀 TUI Thinking Animation**: Displays a rotating spinner while waiting for the provider to reply, then clears it once output starts.
- **🧠 Intelligent Tab Management**: Reuses existing provider tabs if open, focuses them, or opens new ones, avoiding tab clutter.
- **🖥️ Pipe & Stdin Support**: Supports piping prompts via `stdin` (e.g. `cat report.txt | ask "summarize this"`).
- **📎 Image & File Attachments**: Attach local images to ChatGPT with `--image`, or documents (PDF, Word, Excel, plain text, Markdown, JSON, etc.) with `--file`; Gemini currently supports `--file` and rejects `--image`.
- **🔀 Model Switching**: Use `--model` to switch the provider model before the prompt is sent, such as ChatGPT `GPT-5.4` or Gemini `3.5 Flash`.
- **🔍 Quiet by Default & Verbose Mode**: Quiet and clean output by default (displaying only the generated response), with an optional `--verbose` flag to display full browser state logs if needed.
- **Version Info**: Use `-v` or `--version` to print the current version number.

---

## 🛠️ Prerequisites

To run this tool, you need:

1. **`node`/`npx`** installed to automatically launch the `chrome-devtools-mcp` server.
2. **Google Chrome** installed (normally located at `/Applications/Google Chrome.app` on macOS). `make install` installs it with Homebrew when it is missing and Homebrew is available.

You do **not** need a global `mcp-cli` executable. The Rust binary uses `mcp-cli` as a Cargo dependency from `https://github.com/doggy8088/mcp-cli`.

---

## 🚀 Installation & Build

### 1. Build the Rust Project

Clone or navigate to the project directory and install with Make:

```bash
make install
```

This installs the required Chrome browser when needed, builds the optimized binary, and links it to `~/.local/bin/ask`.

### 2. Add to PATH

If you only want to build without installing:

```bash
cargo build --release
```

The compiled binary will be located at `target/release/ask-bridge`.

---

## 📖 Usage Guide

### 1. First Time Setup: Login to a provider

Before sending prompts, you need to log in to the selected provider. ChatGPT is the default:

```bash
ask login
```

For Gemini:

```bash
ask --provider gemini login
```

- This will automatically launch Google Chrome with a dedicated, persistent debug profile.
- Log in manually to the selected provider page, such as `https://chatgpt.com/` or `https://gemini.google.com/app`.
- Once logged in, return to your terminal and press **`[Enter]`**.
- The tool will verify your login status and save your profile. You only need to do this **once**!

### 2. Send Prompts Directly

Simply pass your prompt as an argument:

```bash
ask "What is the difference between a struct and a tuple in Rust?"
ask --provider gemini "What is the difference between a struct and a tuple in Rust?"
```

- Chrome will open or focus on your selected provider tab.
- The prompt will be typed out and submitted.
- The selected provider's response will be printed in your terminal.

### 3. Open a Brand New Session (`--new`)

By default, `ask` will reuse any existing open tab for the selected provider to avoid cluttering your browser with too many tabs.

If you want to start a **completely fresh conversation session** (equivalent to clicking "New Chat" in the sidebar), use the `--new` flag:

```bash
ask "誰是保哥？" --new
```

- This will open a **brand new selected-provider tab**.
- It will **automatically close previous tabs for the same provider** to keep your workspace clean and organized.

### 4. Headless Mode (Default: True)

By default, standard queries run Chrome in **headless mode** (`--headless=true`) so that the browser operates silently in the background without stealing your focus or popping up windows.

If you want to watch Chrome work in real-time or need to manually check what's happening on the page, you can run in **headful mode** by setting `--headless=false`:

```bash
ask "誰是保哥？" --headless=false
```

*Note: `ask login` always overrides the default and runs in **headful mode** so you can interact with the UI. For other subcommands, pass `--headless=false` when you want a visible browser.*

### 5. Verbose Mode (`--verbose`)

By default, `ask` runs in a **quiet, clean mode** that hides all background browser-control logs (such as "Checking open Chrome tabs...", "Typing prompt...", etc.) and only displays the final markdown answer. However, it still plays an animated rotating spinner in your terminal while waiting for the provider to generate a response.

If you want to see detailed step-by-step status logs of what `ask` is doing behind the scenes, add the `--verbose` flag:

```bash
ask "誰是保哥？" --verbose
```

This will print every stage of the browser automation:
- Checking open Chrome tabs...
- Focusing input field...
- Typing your prompt...
- Submitting...
- Waiting for provider response...

### 6. Version Info

Use `-v` or `--version` to print the current version number:

```bash
ask -v
```

### 7. Piping & Stdin Support

You can pipe text or files directly into `ask`:

```bash
echo "Explain quantum computing in one sentence" | ask
```

When you also pass a prompt argument, `ask` uses the prompt first and then appends stdin content after two newlines:

```bash
cat /Users/will/.copilot/session-state/46cc0f1c-79fd-4622-9548-a0b7fa3794be/research/does-cursor-support-byok.md | ask 'What is this?'
```

Or read files:

```bash
cat src/main.rs | ask "Are there any memory leaks in this Rust code?"
```

### 8. Attaching Images or Files

Instead of piping file contents into the prompt, you can upload local files as attachments directly to the selected provider.

#### Attach images

Use `--image` (repeatable) to attach one or more local images. This currently supports ChatGPT; Gemini image input is not enabled and exits with an explicit error when used with `--provider gemini`.

```bash
ask "Describe this image." --image screenshot.png
ask "Compare these two images." --image v1.png --image v2.png
```

Supported formats include PNG, JPEG, GIF, WebP, SVG, BMP, and more.

#### Attach documents

Use `--file` (repeatable) to attach documents such as PDF, Word, Excel, PowerPoint, plain text, Markdown, CSV, JSON, or source code. This flow supports both ChatGPT and Gemini.

```bash
ask "Summarize this PDF." --file report.pdf
ask "How many rows are in this CSV?" --file data.csv
ask "Check this code for issues." --file src/main.rs
```

You can attach images and documents at the same time:

```bash
ask "Compare this design image against the spec document and list inconsistencies." --image design.png --file spec.docx
```

### 9. Switch Model

Use `--model` to automatically switch the provider model before the prompt is sent. Matching is case- and punctuation-insensitive (`-`, `.`, spaces, etc. are ignored).

```bash
ask "Introduce Rust in a few sentences." --model GPT-5.4
ask "Prove this math problem." --model o3
ask "Quickly translate this." --model 即時
ask --provider gemini "Introduce Rust in a few sentences." --model "3.5 Flash"
ask --provider gemini "Introduce Rust in a few sentences." --model "3.1 Pro"
```

Available model names (depending on your account entitlements and provider UI):

- **ChatGPT models**: `GPT-5.5`, `GPT-5.4`, `GPT-5.3`, `o3`
- **ChatGPT thinking levels**: `智慧`, `即時`, `中等`, `高`, `超高`, `專業`
- **Gemini modes**: `3.5 Flash`, `3.1 Flash-Lite`, `3.1 Pro`

> If the requested name is not found in the menu, `ask` reports `Model switch failed: error: model not found in menu` and aborts without submitting the prompt.

### 10. Just Open a Provider

To quickly launch the browser and open the selected provider without sending any query:

```bash
ask open
ask --provider gemini open
```

### 11. Close the Browser Instance

To close the Chrome debug profile instance managed by `ask`:

```bash
ask close
```

`close` only shuts down the `ask` Chrome instance that uses `~/.config/ask-bridge/chrome-profile` and listens on debug port `9223`. If that port is occupied by a non-`ask` Chrome process, it reports an error instead of closing it.

---

## ⚙️ How It Works (Under the Hood)

1. **Browser Initialization**: `ask` checks if Chrome is listening on debugging port `9223`. If not, it spawns Google Chrome as a background process with a custom profile directory (`~/.config/ask-bridge/chrome-profile`).
2. **MCP Bridge Config**: On startup, it automatically writes a custom `mcp_servers.json` to `~/.config/ask-bridge/mcp_servers.json`, configuring the Chrome DevTools MCP server by default with `chrome-devtools-mcp@latest` and `--browser-url=http://127.0.0.1:9223`.
3. **Client Call**: `ask` calls the embedded `doggy8088/mcp-cli` Rust library dependency, invoking `list_pages`, `select_page`, `type_text`, and `evaluate_script` tools to automate the DOM without relying on an external `mcp-cli` executable.
4. **State Polling**: During generation, a lightweight JavaScript engine checks the provider's send/stop button states and extracts response element inner-text for terminal output.

---

## 📄 License

MIT License. Feel free to use, modify, and distribute.
