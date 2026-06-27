# Ask ChatGPT (Rust Version) 🦀

`ask` is a powerful, lightweight command-line tool written in **Rust** that automates ChatGPT directly in your real Chrome browser. It uses the **Model Context Protocol (MCP)** and **Chrome DevTools Protocol (CDP)** via the embedded `doggy8088/mcp-cli` Rust library dependency and `chrome-devtools-mcp` to control Chrome, input prompts, click submit, and **stream the response back to your terminal in real-time**.

Unlike typical API clients, `ask` operates inside a real, headful Chrome browser with a **persistent user profile**. This means:
- You log in manually **once** (`ask login`).
- You can solve CAPTCHAs, bypass Cloudflare, and access GPT-4o or your custom GPTs exactly like a human would.
- Your session cookies, login state, and chat history are saved persistently.

---

## 🌟 Key Features

- **🦀 100% Rust Core**: Extremely fast, lightweight, and compile-once, run-anywhere binary.
- **🌐 Real Browser Automation**: Directly interacts with Chrome on port `9223` (isolated debug profile).
- **🔒 Persistent Login**: Uses a dedicated local profile directory (`~/.config/ask-chatgpt/chrome-profile`) so you never lose your login state.
- **⚡ Real-time Streaming**: Streams ChatGPT's response character-by-character as it's generated, with a beautiful live-typing effect.
- **🌀 TUI Thinking Animation**: Displays a gorgeous, fluid rotating braille spinner (`⠋ 正在思考中 🧠...`) while waiting for ChatGPT to think and reply, which clears automatically the instant the streaming response starts.
- **🧠 Intelligent Tab Management**: Reuses existing ChatGPT tabs if open, focuses them, or opens new ones, avoiding tab clutter.
- **🖥️ Pipe & Stdin Support**: Supports piping prompts via `stdin` (e.g. `cat report.txt | ask "summarize this"`).
- **📎 Image & File Attachments**: Attach local images with `--image` or documents (PDF, Word, Excel, plain text, Markdown, JSON, etc.) with `--file`; each flag can be specified multiple times.
- **🔀 Model Switching**: Use `--model` to switch the ChatGPT model or thinking level (e.g. `GPT-5.4`, `o3`, `即時`) before the prompt is sent.
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

The compiled binary will be located at `target/release/ask-chatgpt`.

---

## 📖 Usage Guide

### 1. First Time Setup: Login to ChatGPT

Before sending prompts, you need to log in to your ChatGPT account. Run:

```bash
ask login
```

- This will automatically launch Google Chrome with a dedicated, persistent debug profile.
- Log in manually to `https://chatgpt.com/` using your account (Google, Apple, Email, etc.).
- Once logged in, return to your terminal and press **`[Enter]`**.
- The tool will verify your login status and save your profile. You only need to do this **once**!

### 2. Send Prompts Directly

Simply pass your prompt as an argument:

```bash
ask "What is the difference between a struct and a tuple in Rust?"
```

- Chrome will open or focus on your ChatGPT tab.
- The prompt will be typed out and submitted.
- The AI's response will **stream live inside your terminal**!

### 3. Open a Brand New Session (`--new`)

By default, `ask` will reuse any existing open ChatGPT tab to avoid cluttering your browser with too many tabs.

If you want to start a **completely fresh conversation session** (equivalent to clicking "New Chat" in the sidebar), use the `--new` flag:

```bash
ask "誰是保哥？" --new
```

- This will open a **brand new ChatGPT tab**.
- It will **automatically close all previous ChatGPT tabs** to keep your workspace clean and organized.

### 4. Headless Mode (Default: True)

By default, standard queries run Chrome in **headless mode** (`--headless=true`) so that the browser operates silently in the background without stealing your focus or popping up windows.

If you want to watch Chrome work in real-time or need to manually check what's happening on the page, you can run in **headful mode** by setting `--headless=false`:

```bash
ask "誰是保哥？" --headless=false
```

*Note: Subcommands like `ask login` and `ask open` always override the default and run in **headful mode** so you can interact with the UI.*

### 5. Verbose Mode (`--verbose`)

By default, `ask` runs in a **quiet, clean mode** that hides all background browser-control logs (such as "Checking open Chrome tabs...", "Typing prompt...", etc.) and only displays the final streamed markdown answer. However, it still plays a beautiful, animated rotating spinner in your terminal while waiting for ChatGPT to generate a response.

If you want to see detailed step-by-step status logs of what `ask` is doing behind the scenes, add the `--verbose` flag:

```bash
ask "誰是保哥？" --verbose
```

This will print every stage of the browser automation:
- Checking open Chrome tabs...
- Focusing input field...
- Typing your prompt...
- Submitting...
- Waiting for ChatGPT response...

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

Or read files:

```bash
cat src/main.rs | ask "Are there any memory leaks in this Rust code?"
```

### 8. Attaching Images or Files

Instead of piping file contents into the prompt, you can upload local files as attachments directly to ChatGPT.

#### Attach images

Use `--image` (repeatable) to attach one or more local images:

```bash
ask "Describe this image." --image screenshot.png
ask "Compare these two images." --image v1.png --image v2.png
```

Supported formats include PNG, JPEG, GIF, WebP, SVG, BMP, and more.

#### Attach documents

Use `--file` (repeatable) to attach documents such as PDF, Word, Excel, PowerPoint, plain text, Markdown, CSV, JSON, or source code:

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

Use `--model` to automatically switch the ChatGPT model or thinking level before the prompt is sent. Matching is case- and punctuation-insensitive (`-`, `.`, spaces, etc. are ignored).

```bash
ask "Introduce Rust in a few sentences." --model GPT-5.4
ask "Prove this math problem." --model o3
ask "Quickly translate this." --model 即時
```

Available model names (depending on your account entitlements):

- **Models**: `GPT-5.5`, `GPT-5.4`, `GPT-5.3`, `o3`
- **Thinking levels**: `智慧`, `即時`, `中等`, `高`, `超高`, `專業`

> If the requested name is not found in the menu, `ask` reports `Model switch failed: error: model not found in menu` and aborts without submitting the prompt.

### 10. Just Open ChatGPT

To quickly launch the browser and open ChatGPT without sending any query:

```bash
ask open
```

---

## ⚙️ How It Works (Under the Hood)

1. **Browser Initialization**: `ask` checks if Chrome is listening on debugging port `9223`. If not, it spawns Google Chrome as a background process with a custom profile directory (`~/.config/ask-chatgpt/chrome-profile`).
2. **MCP Bridge Config**: On startup, it automatically writes a custom `mcp_servers.json` to `~/.config/ask-chatgpt/mcp_servers.json`, configuring the Chrome DevTools MCP server by default with `chrome-devtools-mcp@latest` and `--browser-url=http://127.0.0.1:9223`.
3. **Client Call**: `ask` calls the embedded `doggy8088/mcp-cli` Rust library dependency, invoking `list_pages`, `select_page`, `type_text`, and `evaluate_script` tools to automate the DOM without relying on an external `mcp-cli` executable.
4. **State Polling**: During generation, a lightweight JavaScript engine checks ChatGPT's send/stop button states and extracts response element inner-text, streaming the delta to `stdout`.

---

## 📄 License

MIT License. Feel free to use, modify, and distribute.
