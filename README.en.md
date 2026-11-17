# Ask Bridge 🦀

`ask-bridge` is a powerful, lightweight command-line tool written in **Rust** that automates ChatGPT, Gemini or Claude directly in your real Chrome browser. It uses the **Model Context Protocol (MCP)** and **Chrome DevTools Protocol (CDP)** via the embedded `doggy8088/mcp-cli` Rust library dependency and `chrome-devtools-mcp` to control Chrome, input prompts, click submit, and print the response back to your terminal. ChatGPT is the default when no global provider is configured; use `--provider gemini`, `--provider claude`, or the global config file to switch providers.

## Design Intent

The core purpose of `ask-bridge` is not to replace ChatGPT, Gemini, Claude, or any Coding Agent. It is to bridge them together. During software development, many AI-assisted tasks are exploratory: researching background information, summarizing documents, comparing options, digesting error messages, analyzing code snippets, drafting text, or clarifying uncertain technical questions. These tasks do not always need to be handled directly by the primary Coding Agent, and they do not always justify using the same agent budget that should be reserved for code editing, testing, refactoring, and integration work.

With `ask-bridge`, a Coding Agent can delegate low-risk, exploratory, and research-oriented tasks to the ChatGPT, Gemini, or Claude websites, then bring the website response back into the terminal or the next step of the local workflow. Because ChatGPT, Gemini, and Claude website usage quotas are typically separate from Coding Agent execution quotas, `ask-bridge` gives developers a more flexible way to allocate AI resources: the primary agent can focus on understanding the repository, modifying code, running tests, and integrating results, while website-based AI handles background research, text processing, and candidate solution generation.

In other words, `ask-bridge` is an external research bridge for AI Agents. It turns the manual workflow of switching to a browser, pasting a prompt, waiting for a response, and copying the result back into a command-line-driven automation capability. This lets an agent request help from ChatGPT, Gemini, or Claude without leaving the local development workflow, then use the response as supporting context for its own judgment.

This tool is especially useful for:

- Sending large documents, error messages, or code snippets to a website-based AI for summarization, comparison, or first-pass analysis.
- Letting a Coding Agent outsource background research, alternative analysis, or checklist generation before implementation.
- Moving AI tasks that do not directly modify project files out of the primary agent workflow.
- Reusing existing ChatGPT, Gemini, or Claude web accounts for interactive website features outside an API workflow.

`ask-bridge` does not guarantee that provider output is correct, and it should not replace local tests, official documentation checks, or human review. Its role is to reduce the operational cost of exploratory AI work and let the primary Coding Agent obtain external AI assistance with less friction.

With the default browser backend, `ask-bridge` operates inside a real Chrome browser with a **persistent user profile**. This means:
- You log in manually **once** (`ask-bridge login`).
- You can solve CAPTCHAs, pass provider-side browser checks, and access the selected provider's web features like a normal user.
- Your session cookies, login state, and chat history are saved persistently.

---

## 🌟 Key Features

- **🦀 100% Rust Core**: Extremely fast, lightweight, and compile-once, run-anywhere binary.
- **Multi-provider support**: Choose ChatGPT, Gemini, or Claude with `--provider chatgpt|gemini|claude`.
- **Global provider config**: Set the default provider in `~/.config/ask-bridge/config.json`; CLI `--provider` overrides the config file.
- **🌐 Real Browser Automation**: Directly interacts with Chrome on port `9223` (isolated debug profile).
- **🔒 Persistent Login**: Uses a dedicated local profile directory (`~/.config/ask-bridge/chrome-profile`) so you never lose your login state.
- **Response Output**: Prints the selected provider's response back to your terminal.
- **🌀 TUI Thinking Animation**: Displays a rotating spinner while waiting for the provider to reply, then clears it once output starts.
- **🧠 Intelligent Tab Management**: Reuses existing provider tabs if open, focuses them, or opens new ones, avoiding tab clutter.
- **🖥️ Pipe & Stdin Support**: Supports piping prompts via `stdin` (e.g. `cat report.txt | ask-bridge "summarize this"`).
- **📎 Image & File Attachments**: Attach local images with `--image` (supported on ChatGPT and Claude), or documents (PDF, Word, Excel, plain text, Markdown, JSON, etc.) with `--file`; Gemini currently supports `--file` and rejects `--image`.
- **🔀 Model and Reasoning Selection**: Use `--model` for provider models and `--reasoning` for ChatGPT reasoning effort or Gemini Extended Thinking.
- **Resume Conversations**: Use `--session`, `--session-id`, or `--session-url` to continue an existing provider conversation with a new terminal prompt.
- **Optional SIWC backend**: Use ChatGPT plan usage through OAuth and the public Responses API without an API key. Sign-in opens your system browser; subsequent questions do not launch Chrome. See the [SIWC quick start](#14-use-your-chatgpt-plan-through-siwc).
- **Response Timeout**: Use `--timeout <seconds>` to control how long to wait for a provider response, defaulting to `300` seconds.
- **🔍 Quiet by Default & Verbose Mode**: Quiet and clean output by default (displaying only the generated response), with an optional `--verbose` flag to display full browser state logs if needed.
- **Version Info**: Use `-v` or `--version` to print the current version number.

---

## 🛠️ Prerequisites

The default browser backend requires the following. For SIWC-specific differences, see the [SIWC quick start](#14-use-your-chatgpt-plan-through-siwc):

1. **Node.js 20.19.0 LTS or newer LTS**, with both `node` and `npx` available in the current shell's `PATH`. `ask-bridge` starts `chrome-devtools-mcp@latest` through `npx`; older Node.js versions, such as `v20.11.0`, can cause the MCP server to exit during `initialize`.
2. **Google Chrome** installed (normally located at `/Applications/Google Chrome.app` on macOS). `make install` installs it with Homebrew when it is missing and Homebrew is available.

Check the Node.js version visible to your current shell:

```bash
node -v
npx -v
```

| Platform | Notes |
| --- | --- |
| macOS | Install a Node.js LTS release with Homebrew or nvm. If you use nvm, make sure the same shell that runs `ask-bridge` has loaded nvm and that `node -v` reports `v20.19.0` or newer. Chrome is detected at `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` by default. |
| Windows | Install a Node.js LTS release with the official installer, winget, or nvm-windows. Reopen PowerShell after installation, then verify that `node -v` and `npx -v` work. Chrome is detected from the standard `Program Files`, `Program Files (x86)`, and `%LOCALAPPDATA%` install paths. |
| Linux | Distribution package repositories may provide an older Node.js version. Prefer NodeSource, nvm, or an official Node.js LTS source. Install Google Chrome Stable and make sure `google-chrome` or `google-chrome-stable` is available in `PATH`; Snap, Flatpak, or Chromium installs may not match the default detection logic. |

You do **not** need a global `mcp-cli` executable. The Rust binary uses `mcp-cli` as a Cargo dependency from `https://github.com/doggy8088/mcp-cli`.

---

## 🚀 Installation & Build

### 1. Quick Installation (Recommended)

If you only want to use the pre-compiled Release version (without installing the Rust toolchain), you can run one of the following one-liner installation scripts. They will automatically verify the Node.js requirement, download the appropriate binary for your system architecture, and place it in your `~/.local/bin/` folder.

#### macOS / Linux
Open your terminal and run:
```bash
curl -fsSL https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.sh | bash
```

#### Windows
Open PowerShell (recommended to Run as Administrator) and run:
```powershell
irm https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.ps1 | iex
```

> [!NOTE]
> Make sure the installation path (`~/.local/bin` for macOS/Linux, and `$HOME\.local\bin` for Windows) is added to your system's `PATH` environment variable.
> The formal CLI command is `ask-bridge`; the installer also provides `ask` as a backward-compatible alias. The examples below use `ask-bridge`.
> The Windows installer verifies that the download is a PE executable, runs a
> `--version` smoke test, and puts the official install directory first in User
> PATH. Run `where.exe ask-bridge` and confirm that the first expanded path ends
> with `\.local\bin\ask-bridge.exe`.

### 2. Build & Install from Source (For Developers)

Clone or navigate to the project directory and build/install with Make:

```bash
make install
```

This verifies the Node.js environment, installs the required Chrome browser if needed, builds the optimized binary, links the formal command to `~/.local/bin/ask-bridge`, and also creates the `ask` alias.

### 3. Build Only

If you only want to build without installing:

```bash
cargo build --release
```

The compiled binary will be located at `target/release/ask-bridge`.

### 4. Install the Agent Skill

This repository provides an `ask-bridge` Agent Skill so Skills-compatible Coding Agents can use `ask-bridge` to delegate exploratory research, summarization, document analysis, or option comparison tasks to the ChatGPT, Gemini, or Claude websites.

Install it with `npx skills`; you do not need to copy the `skills/` directory manually:

```bash
npx skills add doggy8088/ask-bridge --skill ask-bridge
```

To install it globally for Codex, specify the agent and global scope:

```bash
npx skills add doggy8088/ask-bridge --skill ask-bridge --agent codex --global
```

---

## 📖 Usage Guide

### 1. First Time Setup: Login to a provider

These instructions sign into the browser backend. For SIWC OAuth authorization, follow the [SIWC quick start](#14-use-your-chatgpt-plan-through-siwc).

Before sending prompts, you need to log in to the selected provider. ChatGPT is the default when no global provider is configured:

```bash
ask-bridge login
```

For Gemini or Claude:

```bash
ask-bridge --provider gemini login
ask-bridge --provider claude login
```

- This will automatically launch Google Chrome with a dedicated, persistent debug profile.
- Log in manually to the selected provider page, such as `https://chatgpt.com/`, `https://gemini.google.com/app`, or `https://claude.ai/new`.
- The tool now checks login status every second automatically, so you can stay on the browser and it will return immediately after login is detected.
- If login is not detected within `--timeout` seconds (default: 300), it will warn you to verify the result and retry.

#### Global Provider Config

To use Gemini or Claude by default when `--provider` is not specified:

```bash
ask-bridge config --provider gemini
ask-bridge config --provider claude
```

To switch the default back to ChatGPT:

```bash
ask-bridge config --provider chatgpt
```

You can check the current setting with:

```bash
ask-bridge config
```

`--provider` has higher priority than the global config file, so this command temporarily uses ChatGPT:

```bash
ask-bridge --provider chatgpt "Summarize this text."
```

### 2. Send Prompts Directly

Simply pass your prompt as an argument:

```bash
ask-bridge "What is the difference between a struct and a tuple in Rust?"
ask-bridge --provider gemini "What is the difference between a struct and a tuple in Rust?"
ask-bridge --provider claude "What is the difference between a struct and a tuple in Rust?"
```

- Chrome will open or focus on your selected provider tab.
- The prompt will be typed out and submitted.
- The selected provider's response will be printed in your terminal.

### 3. Open a Brand New Session (`--new`)

By default, `ask-bridge` will reuse any existing open tab for the selected provider to avoid cluttering your browser with too many tabs.

If you want to start a **completely fresh conversation session** (equivalent to clicking "New Chat" in the sidebar), use the `--new` flag:

```bash
ask-bridge "誰是保哥？" --new
```

- This will open a **brand new selected-provider tab**.
- All provider and non-provider tabs that existed before the command will be preserved.
- If the new tab cannot be identified unambiguously, the command stops instead of
  reusing or closing an existing tab.

### 4. Resume an Existing Conversation

Pass either a provider conversation ID or a full conversation URL to continue its
existing web context:

```bash
ask-bridge --provider chatgpt --session-id "conversation-uuid" "Continue the previous plan."
ask-bridge --session-url "https://chatgpt.com/c/conversation-uuid" "Produce the next steps."
ask-bridge --provider gemini --session "conversation-id" "Continue the analysis."
```

`--session`, `--session-id`, and `--session-url` are aliases for the same option.
An ID uses the selected or configured provider to build the conversation URL. A
full URL identifies its provider automatically. The command stops before opening
Chrome when an explicitly selected provider conflicts with the URL, the URL does
not belong to a supported provider, or `--new` is also supplied. Existing tabs
are preserved.

### 5. Headless Mode (Default: True)

By default, standard queries run Chrome in **headless mode** (`--headless=true`) so that the browser operates silently in the background without stealing your focus or popping up windows.

If you want to watch Chrome work in real-time or need to manually check what's happening on the page, you can run in **headful mode** by setting `--headless=false`:

```bash
ask-bridge "誰是保哥？" --headless=false
```

*Note: `ask-bridge login` always overrides the default and runs in **headful mode** so you can interact with the UI. For other subcommands, pass `--headless=false` when you want a visible browser.*

### 6. Verbose Mode (`--verbose`)

By default, `ask-bridge` runs in a **quiet, clean mode** that hides all background browser-control logs (such as "Checking open Chrome tabs...", "Typing prompt...", etc.) and only displays the final markdown answer. However, it still plays an animated rotating spinner in your terminal while waiting for the provider to generate a response.

If you want to see detailed step-by-step status logs of what `ask-bridge` is doing behind the scenes, add the `--verbose` flag:

```bash
ask-bridge "誰是保哥？" --verbose
```

This will print every stage of the browser automation:
- Checking open Chrome tabs...
- Focusing input field...
- Typing your prompt...
- Submitting...
- Waiting for provider response...

### 7. Response Timeout (`--timeout`)

The default wait time is 300 seconds. If a provider takes longer, increase (or decrease) the timeout:

```bash
ask-bridge "Please summarize this report" --timeout 600
```

You can also set a shorter timeout:

```bash
ask-bridge "Short answer" --timeout 60
```

If the response does not complete within the configured seconds, `ask-bridge` will stop waiting and print a timeout warning.

### 8. Version Info

Use `-v` or `--version` to print the current version number:

```bash
ask-bridge -v
```

### 9. Piping & Stdin Support

You can pipe text or files directly into `ask-bridge`:

```bash
echo "Explain quantum computing in one sentence" | ask-bridge
```

When you also pass a prompt argument, `ask-bridge` uses the prompt first and then appends stdin content after two newlines:

```bash
cat /Users/will/.copilot/session-state/46cc0f1c-79fd-4622-9548-a0b7fa3794be/research/does-cursor-support-byok.md | ask-bridge 'What is this?'
```

Or read files:

```bash
cat src/main.rs | ask-bridge "Are there any memory leaks in this Rust code?"
```

### 10. Attaching Images or Files

Instead of piping file contents into the prompt, you can upload local files as attachments directly to the selected provider.

#### Attach images

Use `--image` (repeatable) to attach one or more local images. This currently supports ChatGPT and Claude; Gemini image input is not enabled and exits with an explicit error when used with `--provider gemini`.

```bash
ask-bridge "Describe this image." --image screenshot.png
ask-bridge "Compare these two images." --image v1.png --image v2.png
ask-bridge --provider claude "Describe this image." --image screenshot.png
```

Supported formats include PNG, JPEG, GIF, WebP, SVG, BMP, and more.

#### Attach documents

Use `--file` (repeatable) to attach documents such as PDF, Word, Excel, PowerPoint, plain text, Markdown, CSV, JSON, or source code. This flow supports ChatGPT, Gemini, and Claude.

```bash
ask-bridge "Summarize this PDF." --file report.pdf
ask-bridge "How many rows are in this CSV?" --file data.csv
ask-bridge "Check this code for issues." --file src/main.rs
```

You can attach images and documents at the same time:

```bash
ask-bridge "Compare this design image against the spec document and list inconsistencies." --image design.png --file spec.docx
```

### 11. Switch Models and Reasoning

Use `--model` to switch the provider model before the prompt is sent. Use `--reasoning` separately for provider-specific reasoning modes. ChatGPT accepts both in one invocation; Gemini Extended Thinking is compatible only with Pro models.

```bash
ask-bridge "Prove this math problem." --model "GPT-5.6 Sol" --reasoning high
ask-bridge "Quickly translate this." --reasoning instant
ask-bridge --provider gemini "Introduce Rust in a few sentences." --model "3.6 Flash"
ask-bridge --provider gemini "Prove this math problem." --model "3.1 Pro" --reasoning extended
ask-bridge --provider claude "Introduce Rust in a few sentences." --model Sonnet
```

Argument rules:

- **ChatGPT**: `--reasoning` accepts `instant`, `medium`, `high`, and `xhigh`, plus the corresponding aliases `即時`, `中`, `中等`, `高`, and `極高`. The redesigned ChatGPT reasoning slider no longer offers Auto, so `auto` only works where that option still exists. ask-bridge always uses ChatGPT's Chat mode and switches the composer back from Work mode before sending.
- **Gemini**: `--reasoning extended` selects Extended Thinking. Omit `--model` or combine it with an available Pro model.
- **Claude**: `--reasoning` is unsupported. Existing Sonnet, Opus, and Haiku `--model` selection is unchanged.

Model matching uses only each menu item's primary label and ignores subtitles and badges. It remains case- and punctuation-insensitive. The tool never maps an obsolete model version to a different version. If the primary label is unavailable, the error lists the provider options currently found and aborts before sending the prompt.

Legacy forms such as `--model 高` and Gemini `--model 延伸思考` remain temporarily supported with a deprecation warning. Use `--reasoning` instead.

### 12. Just Open a Provider

To quickly launch the browser and open the selected provider without sending any query:

```bash
ask-bridge open
ask-bridge --provider gemini open
ask-bridge --provider claude open
```

### 13. Close the Browser Instance

To close the Chrome debug profile instance managed by `ask-bridge`:

```bash
ask-bridge close
```

`close` only shuts down the `ask-bridge` Chrome instance that uses `~/.config/ask-bridge/chrome-profile` and listens on debug port `9223`. If that port is occupied by a non-`ask-bridge` Chrome process, it reports an error instead of closing it.

* * *

### 14. Use Your ChatGPT Plan Through SIWC

**SIWC is opt-in. Browser/CDP remains the default unless you change the configuration.** Sign in with ChatGPT authorizes access to the public Responses API using your ChatGPT plan, without an API key. Use it for single-turn text questions, summaries, and comparisons. Choose the browser backend when you need existing web conversations, attachments, or image generation.

#### Providers, backends, and usage

| Setting | Meaning |
| --- | --- |
| `--provider chatgpt\|gemini\|claude` | Selects the AI service. SIWC currently supports ChatGPT only. |
| `--backend browser\|siwc` | Selects the backend for this invocation and overrides the saved preference. |
| `config --chatgpt-backend siwc` | Saves the default ChatGPT backend. `--chatgpt-backend` is not an alias for `--backend`; it only takes effect with `config`. |
| Usage source | ChatGPT Work/Codex plan usage or credits, rather than API-key billing. |
| Sign-in state | SIWC OAuth credentials are separate from browser cookies. Signing into the website does not authorize SIWC. |

**SIWC does not provide free or unlimited tokens, or guarantee lower total token usage or cost.** If your primary Coding Agent shares the same plan allowance, both may consume that allowance. Eligibility, models, quotas, and credits depend on the account and workspace; a plan name alone does not guarantee access.

#### Quick start: sign in and ask a question

Install `ask-bridge` using the installation instructions above, then run:

```bash
# 1. Sign in and authorize ChatGPT plan usage in your system browser
ask-bridge --provider chatgpt --backend siwc login

# 2. Wait for the terminal to confirm sign-in, then ask a question
ask-bridge --provider chatgpt --backend siwc "Explain three situations where this tool is useful"
```

The first sign-in performs dynamic registration. OAuth authorization opens your system browser; subsequent questions use HTTP streaming and **do not launch Chrome or use CDP/MCP**. SIWC inference itself does not require Node.js/npx. The current installation scripts still check browser-backend dependencies, so follow the installation instructions above.

Use an account authorized for ChatGPT plan usage, grant that access on the authorization page, and wait for the browser to return to the local callback. After sign-in, the tool attempts to list up to 10 available model names and slugs. If the browser does not open automatically, use the authorization URL printed in the terminal. The login callback has a fixed 300-second wait; the question's `--timeout` option does not change it.

#### Text input, models, timeouts, and output files

```bash
# Supply a plain-text document as context
cat report.md | ask-bridge --provider chatgpt --backend siwc "Summarize the key points and open questions"

# Read the entire prompt from stdin
ask-bridge --provider chatgpt --backend siwc < prompt.txt

# Set the overall request timeout and save the completed answer
ask-bridge --provider chatgpt --backend siwc --timeout 120 -o answer.md "Compare two technical approaches"

# Replace MODEL_SLUG with an actual slug shown after sign-in
ask-bridge --provider chatgpt --backend siwc --model "MODEL_SLUG" "Compare two technical approaches"
```

| Option/input | SIWC behavior |
| --- | --- |
| Prompt plus stdin | Sends the prompt, two newline characters, and stdin together. Other files are not read automatically. |
| `--model` | Validated against the account's API model catalog on each request. Accepts a slug or full display name, with case-insensitive name matching. Defaults to the first catalog entry, independently of the web model selector. An unavailable model lists available slugs and aborts without selecting a substitute. |
| `--timeout` | Defaults to 300 seconds and covers token refresh, the model catalog, and inference streaming. Preflight HTTP requests also have a 30-second limit, so they may fail before the overall deadline. |
| `-o` / `--output` | Text still streams to the terminal. The file is written only after the successful `response.completed` event. A successful request overwrites an existing file; its parent directory must already exist. |

**Partial terminal output is not proof of success.** A usage error, disconnection, or timeout can occur after streaming starts. The process then exits with a nonzero code and does not perform the `-o` write, leaving any existing output file untouched by that inference failure. Shell redirection with `>` can leave partial output; use `-o` and check the exit code when you need a completed answer.

#### How do I pass my current working context?

**Pipe the relevant text through stdin and describe the task in the prompt.** SIWC does not automatically read your working directory, Git state, IDE selection, or Coding Agent conversation. It receives only what you supply. Passing a file's text through stdin is different from uploading it with `--file`.

These examples use Bash/zsh. Label multiple sources so the model can distinguish them:

```bash
# Compare two documents with explicit source labels
{
  printf '\n--- a.md ---\n'
  cat a.md
  printf '\n--- b.md ---\n'
  cat b.md
} | ask-bridge --provider chatgpt --backend siwc "Compare these documents and identify contradictions"

# Review unstaged changes; use git diff --cached for staged changes
# git diff does not include untracked files
git diff -- src/main.rs | ask-bridge --provider chatgpt --backend siwc "Review this diff for logic risks"

# Capture test output before asking for analysis
cargo test > test-output.txt 2>&1
cat test-output.txt | ask-bridge --provider chatgpt --backend siwc "Analyze the failures and suggest fixes"
```

In Windows PowerShell, use `Get-Content -Raw` to supply text:

```powershell
Get-Content -Raw -Encoding utf8 report.md | ask-bridge --provider chatgpt --backend siwc "Summarize the key points"
```

Each invocation is an independent question. The tool does not automatically save or resend earlier context, and `--session` cannot resume a web conversation. To continue a discussion, select the previous question, answer, and necessary background yourself and include them in the next input:

```bash
# Continue only if the first request succeeds, avoiding a stale summary file
if cat report.md | ask-bridge --provider chatgpt --backend siwc -o summary.md "Summarize the report and list risks"; then
  {
    printf '\n--- Previous question ---\nSummarize the report and list risks.\n'
    printf '\n--- Previous answer ---\n'
    cat summary.md
    printf '\n--- Relevant source ---\n'
    cat report.md
  } | ask-bridge --provider chatgpt --backend siwc -o follow-up.md "Use the supplied summary and source to propose three improvements"
fi
```

This manually supplies text context; it is not built-in multi-turn conversation support. Include relevant excerpts and remove passwords, tokens, personal information, or other material you should not send. Repeatedly including all history increases input tokens, latency, and plan usage, and may exceed the model's context limit. Do not pipe PDF, Word, or image binary data as text; extract text first or use the browser backend's supported attachments.

#### Save a default or switch back to the browser

```bash
# Change only the ChatGPT backend preference, preserving other settings
ask-bridge config --chatgpt-backend siwc
ask-bridge config

# The backend flag is now optional; keep ChatGPT explicit
ask-bridge --provider chatgpt "Summarize the risks in this design"

# Use the browser for this invocation without changing the saved preference
# Sign into the website first if necessary
ask-bridge --provider chatgpt --backend browser login
ask-bridge --provider chatgpt --backend browser "Answer this using the web interface"

# Restore the default ChatGPT browser backend
ask-bridge config --chatgpt-backend browser
```

Precedence is `--backend` > `chatgptBackend` in `config.json` > `browser`. Saving SIWC does not change the default provider. If yours is Gemini or Claude, specify `--provider chatgpt` to use SIWC. Gemini/Claude ignore the saved ChatGPT backend preference, but explicitly combining either with `--backend siwc` is an error.

#### Browser and SIWC feature matrix

| Feature | Browser backend | SIWC backend |
| --- | --- | --- |
| Services | ChatGPT, Gemini, Claude | ChatGPT only |
| Sign-in and transport | Website login, Chrome/CDP/MCP | OAuth; HTTP/SSE for questions |
| Single-turn text, stdin, `-o` | Supported | Supported |
| Model selection | Web model menu | Account API model catalog |
| `--session` / `--session-id` / `--session-url` | Supported | Rejected; cannot read `chatgpt.com/c/...` |
| `--new` / `--temporary` | Supported | Rejected; each request is independent, with no web temporary-chat mode |
| `--image` / `--file` | Provider-dependent | Not implemented in this tool; rejected |
| Image generation and `--image-output` / `-i` | Website-dependent | Unsupported; image-output options are rejected |
| `--reasoning` | Provider-dependent | Not implemented; rejected |
| `batch` | Supported | Rejected; explicitly use `--backend browser` if SIWC is your default |
| Browser commands such as `open` / `get` / `close` | Supported | Rejected; explicitly use `--backend browser` |
| Usage source | Website product/account limits | ChatGPT Work/Codex plan usage or credits |

There is no `--download-images` CLI flag. `downloadImages` is a browser batch-job JSON field; SIWC rejects `batch`, so it cannot enable downloads through that field.

The tool's SIWC MVP supports single-turn text inference, without file search, Code Interpreter, image generation, or hosted MCP/connectors. Missing attachment support is a limitation of this tool, not a claim that the upstream SIWC API rejects all image or document input. This HTTP path uses `store: false` and `stream: true`, and does not use `previous_response_id` to continue a conversation. `store: false` is not a guarantee of zero service-side data retention. See OpenAI's [models and inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference) and [Preview limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations) for API capabilities.

#### Reauthorize, sign out, and manage credentials

```bash
# Reauthorize after expiration, revocation, or a refresh failure
ask-bridge --provider chatgpt --backend siwc login

# Clear locally stored SIWC accounts and tokens
ask-bridge --provider chatgpt --backend siwc logout
```

Credentials are saved atomically in `~/.config/ask-bridge/siwc-credentials.json`, with `0600` permissions on Unix. That Unix permission notation does not apply to Windows. The tool refreshes expiring access tokens and saves rotated refresh tokens. Do not share the credential file, commit it to Git, or paste it into an issue.

`logout` clears local accounts and tokens while preserving the stable host ID, so the file may remain. It does not call a server-side revocation API, sign you out of the ChatGPT website, or reset your default backend to `browser`. Sign in again before the next SIWC request.

#### Troubleshooting

| Symptom | Action |
| --- | --- |
| `not signed in`, 401, or refresh failure | Run the full SIWC `login` command above. Website login cannot replace OAuth authorization. |
| Authorization declined or plan-usage permission missing | Check the account, workspace, and plan-usage authorization, then sign in again. A subscription name alone does not guarantee permission. |
| Unavailable model or empty catalog | Use a slug listed in the error. For an empty catalog, check account permissions and service availability. |
| 429, `subscription_sharing_usage_limit_exceeded`, or `subscription_sharing_usage_unavailable` | Check plan usage, credits, or account availability and follow the error. These errors may arrive after streaming starts. |
| Timeout, interrupted stream, or no completion event | The request failed; displayed text may be incomplete. Check connectivity before deciding whether to submit again. |
| Incompatible option | Remove it or explicitly select `--backend browser` and sign into the website. |
| `UnknownIssuer` | Ask your administrator to verify the required CA is installed in the system trust store. Do not disable TLS certificate verification. |

**Failures never automatically switch to Browser/CDP or API-key billing.** HTTPS uses rustls with built-in CAs and the operating system trust store. Connection errors include their underlying causes without request URLs, headers, or bodies. In TLS-inspection environments such as corporate proxies or antivirus HTTPS scanning, a trusted interceptor can see connection contents, including token exchange and refresh. Use SIWC only when you trust that environment.

#### Verification scope and references

According to the [issue #40 closure record](https://github.com/doggy8088/ask-bridge/issues/40#issuecomment-6956849874), as of 2026-11-17:

- OAuth sign-in and a single text conversation were verified live on Windows.
- Refresh-token rotation and expired-session reauthorization, revocation effects after logout, exhausted-quota errors, and network-interruption recovery were not individually verified live.
- No performance or usage-cost benchmarks were available, so this is not evidence of guaranteed latency or token savings.

This records verification at issue closure, not coverage of every account, platform, or failure case. Consult `ask-bridge --help` for the current CLI.

* * *

## ⚙️ How It Works (Under the Hood)

The steps below describe the Browser/CDP backend. After OAuth authorization, SIWC instead fetches the account model catalog and calls Responses through HTTP/SSE, bypassing these browser steps.

1. **Browser Initialization**: `ask-bridge` checks if Chrome is listening on debugging port `9223`. If not, it spawns Google Chrome as a background process with a custom profile directory (`~/.config/ask-bridge/chrome-profile`).
2. **MCP Bridge Config**: On startup, it automatically writes a custom `mcp_servers.json` to `~/.config/ask-bridge/mcp_servers.json`, configuring the Chrome DevTools MCP server by default with `chrome-devtools-mcp@latest` and `--browser-url=http://127.0.0.1:9223`.
3. **Client Call**: `ask-bridge` calls the embedded `doggy8088/mcp-cli` Rust library dependency, invoking `list_pages`, `select_page`, `type_text`, and `evaluate_script` tools to automate the DOM without relying on an external `mcp-cli` executable.
4. **State Polling**: During generation, a lightweight JavaScript engine checks the provider's send/stop button states and extracts response element inner-text for terminal output.

---

## 📄 License

MIT License. Feel free to use, modify, and distribute.
