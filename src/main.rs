use clap::{CommandFactory, Parser, Subcommand};
use mcp_cli::McpClient;
use serde_json::Value;
use std::io::{self, IsTerminal, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "ask")]
#[command(version = "0.1.0")]
#[command(about = "ChatGPT Browser Automation CLI in Rust", long_about = None)]
struct Cli {
    /// The prompt to send to ChatGPT. If empty, reads from standard input.
    prompt: Option<String>,

    /// Run Chrome in headless mode. Defaults to true.
    #[arg(long, require_equals = true, num_args = 0..=1, default_value = "true", default_missing_value = "true")]
    headless: bool,

    /// Create a brand new ChatGPT session by opening a new tab and closing old ones.
    #[arg(long, default_value_t = false)]
    new: bool,

    /// Print verbose debugging status messages.
    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    /// Write the final response in Markdown format to the specified file.
    #[arg(long, short, value_name = "FILE")]
    output: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Open Chrome browser, optionally navigate to a URL, and copy the latest response
    Open {
        /// Optional ChatGPT conversation URL to open before copying the latest response.
        url: Option<String>,
    },
    /// Retrieve the latest response from ChatGPT (defaults to headless)
    Get {
        /// Optional ChatGPT conversation URL to fetch before copying the latest response.
        url: Option<String>,
    },
    /// Open Chrome browser and wait for manual login
    Login,
}

struct Page {
    id: usize,
    url: String,
    selected: bool,
}

fn write_mcp_config(quiet_mcp: bool, headless: bool) -> Result<String, String> {
    let mut config_dir = home::home_dir().ok_or("Could not locate home directory")?;
    config_dir.push(".config/ask-chatgpt");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let log_path = config_dir.join("chrome-devtools-mcp.log").to_string_lossy().to_string();

    config_dir.push("mcp_servers.json");
    let config_path = config_dir.to_string_lossy().to_string();

    let mut chrome_devtools_server = if quiet_mcp {
        let mut sh_cmd = format!(
            "exec npx -y chrome-devtools-mcp@latest --browser-url=http://127.0.0.1:9223 --no-usage-statistics --no-performance-crux --logFile \"{}\"",
            log_path
        );
        if headless {
            sh_cmd.push_str(" --headless");
        }
        sh_cmd.push_str(" 2>/dev/null");

        serde_json::json!({
            "command": "sh",
            "args": [
                "-c",
                sh_cmd
            ]
        })
    } else {
        let mut mcp_args = vec![
            "-y".to_string(),
            "chrome-devtools-mcp@latest".to_string(),
            "--browser-url=http://127.0.0.1:9223".to_string(),
        ];
        if headless {
            mcp_args.push("--headless".to_string());
        }
        mcp_args.push("--logFile".to_string());
        mcp_args.push(log_path);

        serde_json::json!({
            "command": "npx",
            "args": mcp_args
        })
    };

    if quiet_mcp {
        chrome_devtools_server["env"] = serde_json::json!({
            "NPM_CONFIG_LOGLEVEL": "error",
            "NPM_CONFIG_PROGRESS": "false",
            "NPM_CONFIG_FUND": "false",
            "NPM_CONFIG_AUDIT": "false",
            "NPM_CONFIG_FUNDING": "0",
            "NPM_CONFIG_UPDATE_NOTIFIER": "false",
            "NO_COLOR": "1",
            "CI": "1",
            "NODE_NO_WARNINGS": "1"
        });
    }

    let config_content = serde_json::json!({
        "mcpServers": {
            "chrome-devtools": chrome_devtools_server
        }
    });

    let content_str = serde_json::to_string_pretty(&config_content).map_err(|e| e.to_string())?;

    std::fs::write(&config_path, content_str)
        .map_err(|e| format!("Failed to write mcp_servers.json: {}", e))?;

    Ok(config_path)
}

fn start_chrome_if_needed(headless: bool, verbose: bool) -> Result<(), String> {
    let mut profile_dir = home::home_dir().ok_or("Could not locate home directory")?;
    profile_dir.push(".config/ask-chatgpt/chrome-profile");
    std::fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to create chrome profile directory: {}", e))?;

    let profile_path = profile_dir.to_string_lossy().to_string();

    if TcpStream::connect("127.0.0.1:9223").is_ok() {
        if !headless || is_debug_chrome_background(&profile_path) {
            if headless {
                // Force hide any existing background Chrome PIDs asynchronously just in case they are currently visible
                let pids = debug_port_listener_pids();
                thread::spawn(move || {
                    for pid_str in pids {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            let script = format!(
                                "tell application \"System Events\" to set visible of first application process whose unix id is {} to false",
                                pid
                            );
                            let _ = Command::new("osascript")
                                .arg("-e")
                                .arg(&script)
                                .status();
                        }
                    }
                });
            }
            return Ok(());
        }

        if verbose {
            println!("Found ask Chrome on port 9223 running visibly. Restarting in background...");
        }
        stop_ask_chrome_on_debug_port(&profile_path)?;
    }

    if verbose {
        println!(
            "Chrome is not running on port 9223. Starting Chrome with remote debugging (headless: {})...",
            headless
        );
    }

    let chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
    if !std::path::Path::new(chrome_path).exists() {
        return Err("Google Chrome not found at /Applications/Google Chrome.app".to_string());
    }

    let mut cmd = Command::new(chrome_path);
    cmd.arg("--remote-debugging-port=9223")
        .arg(format!("--user-data-dir={}", profile_path))
        .arg("--no-first-run")
        .arg("--no-default-browser-check");

    if headless {
        cmd.arg("--ask-chatgpt-background")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--window-size=1440,1200")
            .arg("--window-position=-2000,-2000");
    }

    let child = cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start Google Chrome: {}", e))?;

    if headless {
        let pid = child.id();
        thread::spawn(move || {
            // Rapidly set visibility to false during startup to prevent window from flashing or drawing
            for _ in 0..40 {
                let script = format!(
                    "tell application \"System Events\" to try\nset visible of first application process whose unix id is {} to false\nend try",
                    pid
                );
                let _ = Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .status();
                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    // Wait for Chrome to listen on port 9223
    for _ in 0..50 {
        if TcpStream::connect("127.0.0.1:9223").is_ok() {
            if verbose {
                println!("Chrome started and listening on port 9223.");
            }
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    Err("Timed out waiting for Chrome to start on port 9223".to_string())
}

fn debug_port_listener_pids() -> Vec<String> {
    let output = Command::new("lsof")
        .args(["-tiTCP:9223", "-sTCP:LISTEN"])
        .output();

    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn process_command(pid: &str) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", pid, "-o", "command="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_debug_chrome_background(profile_path: &str) -> bool {
    let profile_arg = format!("--user-data-dir={}", profile_path);

    debug_port_listener_pids().iter().any(|pid| {
        process_command(pid)
            .map(|cmd| {
                cmd.contains(&profile_arg)
                    && cmd.contains("--ask-chatgpt-background")
            })
            .unwrap_or(false)
    })
}

fn stop_ask_chrome_on_debug_port(profile_path: &str) -> Result<(), String> {
    let profile_arg = format!("--user-data-dir={}", profile_path);
    let ask_pids: Vec<String> = debug_port_listener_pids()
        .into_iter()
        .filter(|pid| {
            process_command(pid)
                .map(|cmd| cmd.contains(&profile_arg))
                .unwrap_or(false)
        })
        .collect();

    if ask_pids.is_empty() {
        return Err(
            "Port 9223 is already used by a non-ask Chrome process. Stop it or use a different debugging port."
                .to_string(),
        );
    }

    for pid in ask_pids {
        let _ = Command::new("kill").arg(&pid).status();
    }

    for _ in 0..50 {
        if TcpStream::connect("127.0.0.1:9223").is_err() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    Err("Timed out waiting for existing ask Chrome to stop".to_string())
}

fn call_mcp_tool(config_path: &str, tool: &str, args: Value) -> Result<Value, String> {
    let client = McpClient::load(Some(config_path))
        .map_err(|e| format!("Failed to load MCP config: {}", e))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create async runtime for MCP call: {}", e))?;

    runtime
        .block_on(async { client.call_tool("chrome-devtools", tool, args).await })
        .map_err(|e| format!("mcp-cli library call failed: {}", e))
}

fn parse_pages(text: &str) -> Vec<Page> {
    let mut pages = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("##") {
            continue;
        }
        if let Some((id_str, rest)) = line.split_once(':') {
            let id = match id_str.trim().parse::<usize>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let rest = rest.trim();
            let (url, selected) = if rest.ends_with("[selected]") {
                let url = rest.strip_suffix("[selected]").unwrap().trim().to_string();
                (url, true)
            } else {
                (rest.to_string(), false)
            };
            pages.push(Page { id, url, selected });
        }
    }
    pages
}

fn parse_script_result(val: &Value) -> Result<Value, String> {
    let text = val
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            format!(
                "Could not extract text field from evaluate_script result: {:?}",
                val
            )
        })?;

    let start_tag = "```json";
    let end_tag = "```";

    if let Some(start_pos) = text.find(start_tag) {
        let json_start = start_pos + start_tag.len();
        if let Some(end_pos) = text[json_start..].find(end_tag) {
            let json_str = text[json_start..json_start + end_pos].trim();
            let parsed: Value = serde_json::from_str(json_str)
                .map_err(|e| format!("JSON parsing error: {}\nJSON content: {}", e, json_str))?;
            return Ok(parsed);
        }
    }

    Err(format!(
        "Could not find JSON fencing in script result:\n{}",
        text
    ))
}

fn is_glow_available() -> bool {
    Command::new("glow")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn render_markdown(markdown: &str, use_glow: bool) -> Result<(), String> {
    if markdown.is_empty() {
        return Ok(());
    }

    if use_glow {
        let glow = Command::new("glow")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();

        if let Ok(mut child) = glow {
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(e) = stdin.write_all(markdown.as_bytes()) {
                    eprintln!("Failed to send Markdown content to glow: {}", e);
                }
            }

            match child.wait() {
                Ok(status) if status.success() => {
                    return Ok(());
                }
                Ok(status) => {
                    eprintln!("glow exited with status: {}", status);
                }
                Err(e) => {
                    eprintln!("Failed to wait for glow process: {}", e);
                }
            }
        }
    }

    print!("{}", markdown);
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {}", e))?;

    Ok(())
}

fn read_clipboard() -> Result<String, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| format!("Failed to run pbpaste: {}", e))?;

    if !output.status.success() {
        return Err(format!("pbpaste exited with status: {}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn write_clipboard(content: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pbcopy: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write clipboard content: {}", e))?;
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for pbcopy: {}", e))?;

    if !status.success() {
        return Err(format!("pbcopy exited with status: {}", status));
    }

    Ok(())
}

fn click_latest_copy_button(config_path: &str) -> Result<(), String> {
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": r#"() => {
                const isVisible = (el) => {
                    if (!el || el.disabled || el.getAttribute('aria-disabled') === 'true') return false;
                    const style = window.getComputedStyle(el);
                    if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                    const rect = el.getBoundingClientRect();
                    return rect.width > 0 && rect.height > 0;
                };

                const labelOf = (el) => [
                    el.getAttribute('aria-label'),
                    el.getAttribute('title'),
                    el.getAttribute('data-testid'),
                    el.textContent
                ].filter(Boolean).join(' ');

                const isCopyButton = (el) => {
                    const label = labelOf(el);
                    return /copy|複製|复制|コピー|복사/i.test(label)
                        && !/prompt|提示詞|提示词|入力|table|表格/i.test(label);
                };
                const copyButtonScore = (el) => {
                    const label = labelOf(el);
                    if (!isCopyButton(el) || !isVisible(el)) return -1;
                    if (el.closest('pre, code, [class*="code"], [data-testid*="code"]')) return -1;
                    if (/copy-turn-action-button/i.test(label)) return 100;
                    if (/response|回應|回答|reply/i.test(label)) return 90;
                    if (el.closest('model-response, response-container, [data-message-author-role="assistant"]')) return 50;
                    return 10;
                };
                const messages = Array.from(document.querySelectorAll([
                    '[data-message-author-role="assistant"]',
                    'model-response',
                    '.model-response',
                    '[data-test-id*="response"]',
                    '[data-testid*="response"]'
                ].join(',')));
                const latest = messages[messages.length - 1];
                if (!latest) return { ok: false, reason: "No assistant message found" };

                latest.scrollIntoView({ block: 'center', inline: 'nearest' });
                for (const type of ['pointerover', 'mouseover', 'mouseenter']) {
                    latest.dispatchEvent(new MouseEvent(type, { bubbles: true, view: window }));
                }

                const scopes = [
                    latest,
                    latest.closest('article'),
                    latest.closest('[data-testid^="conversation-turn"]'),
                    latest.parentElement,
                    latest.parentElement?.parentElement
                ].filter(Boolean);

                for (const scope of scopes) {
                    const buttons = Array.from(scope.querySelectorAll('button'));
                    const candidates = buttons
                        .map((button) => ({ button, score: copyButtonScore(button) }))
                        .filter((candidate) => candidate.score >= 0)
                        .sort((a, b) => b.score - a.score);
                    if (candidates.length > 0) {
                        const button = candidates[0].button;
                        button.click();
                        return { ok: true, label: labelOf(button) };
                    }
                }

                return { ok: false, reason: "Copy response button not found" };
            }"#
        }),
    )?;

    let parsed = parse_script_result(&res)?;
    if parsed["ok"].as_bool().unwrap_or(false) {
        Ok(())
    } else {
        Err(parsed["reason"]
            .as_str()
            .unwrap_or("Failed to click copy response button")
            .to_string())
    }
}

fn wait_for_page_load(config_path: &str, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Waiting for page to load...");
    }

    for _ in 0..90 {
        let ready_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": "() => document.readyState === 'complete' || document.readyState === 'interactive'"
            }),
        );

        if let Ok(res) = ready_res {
            if let Ok(parsed) = parse_script_result(&res) {
                if parsed.as_bool().unwrap_or(false) {
                    return Ok(());
                }
            }
        }

        thread::sleep(Duration::from_millis(500));
    }

    Err("Timeout waiting for page to load".to_string())
}

fn open_url_tab(config_path: &str, url: &str, headless: bool, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Opening URL: {}", url);
    }

    let list_res = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))?;
    let text = list_res
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("Invalid list_pages response structure: {:?}", list_res))?;

    let pages = parse_pages(text);
    if pages.len() == 1
        && (pages[0].url == "about:blank"
            || pages[0].url.contains("new-tab-page")
            || pages[0].url.contains("chrome://welcome"))
    {
        call_mcp_tool(
            config_path,
            "navigate_page",
            serde_json::json!({
                "url": url
            }),
        )?;
    } else {
        call_mcp_tool(
            config_path,
            "new_page",
            serde_json::json!({
                "url": url
            }),
        )?;
    }

    for _ in 0..20 {
        let refreshed_pages_res = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))?;
        let refreshed_text = refreshed_pages_res
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                format!(
                    "Invalid refreshed list_pages response structure: {:?}",
                    refreshed_pages_res
                )
            })?;

        let refreshed_pages = parse_pages(refreshed_text);
        if let Some(page) = refreshed_pages.iter().find(|page| page.url == url) {
            call_mcp_tool(
                config_path,
                "select_page",
                serde_json::json!({
                    "pageId": page.id,
                    "bringToFront": !headless
                }),
            )?;

            for stale_page in refreshed_pages.iter().filter(|p| p.id != page.id) {
                let _ = call_mcp_tool(
                    config_path,
                    "close_page",
                    serde_json::json!({
                        "pageId": stale_page.id
                    }),
                );
            }

            return wait_for_page_load(config_path, verbose);
        }

        thread::sleep(Duration::from_millis(250));
    }

    wait_for_page_load(config_path, verbose)
}

fn copy_latest_markdown(config_path: &str) -> Result<String, String> {
    let clipboard_before = read_clipboard().unwrap_or_default();
    let sentinel = format!("__ASK_CHATGPT_COPY_PENDING_{}__", std::process::id());
    write_clipboard(&sentinel)?;

    // Click the copy button, retrying if the message or button is not found yet (due to asynchronous rendering of Single Page App)
    let mut click_err = None;
    for _ in 0..30 {
        match click_latest_copy_button(config_path) {
            Ok(_) => {
                click_err = None;
                break;
            }
            Err(e) => {
                click_err = Some(e);
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    if let Some(err) = click_err {
        // Restore clipboard before returning error
        let _ = write_clipboard(&clipboard_before);
        return Err(format!("Error copying latest response Markdown: {}", err));
    }

    let mut copied_content = None;
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(100));
        match read_clipboard() {
            Ok(content) if !content.trim().is_empty() && content != sentinel => {
                copied_content = Some(content);
                break;
            }
            _ => {}
        }
    }

    // Always restore the original clipboard
    let _ = write_clipboard(&clipboard_before);

    let content = copied_content.ok_or_else(|| {
        "Timed out waiting for clipboard content after clicking copy".to_string()
    })?;

    // Create a temporary file path
    let temp_path = std::env::temp_dir().join(format!("ask_chatgpt_{}.md", std::process::id()));

    // Write the copied content immediately to the temporary file
    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("Failed to write to temporary file: {}", e))?;

    // Read the content back from the temporary file to output to the terminal
    let verified_content = std::fs::read_to_string(&temp_path)
        .map_err(|e| format!("Failed to read from temporary file: {}", e))?;

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_path);

    Ok(verified_content)
}

fn ensure_chatgpt_tab(config_path: &str, force_new: bool, headless: bool, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Checking open Chrome tabs...");
    }
    let list_res = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))?;

    let text = list_res
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("Invalid list_pages response structure: {:?}", list_res))?;

    let pages = parse_pages(text);

    if force_new {
        let old_chatgpt_ids: Vec<usize> = pages
            .iter()
            .filter(|p| p.url.contains("chatgpt.com"))
            .map(|p| p.id)
            .collect();

        if verbose {
            println!("Opening a brand new ChatGPT session...");
        }
        call_mcp_tool(
            config_path,
            "new_page",
            serde_json::json!({
                "url": "https://chatgpt.com/"
            }),
        )?;

        for id in old_chatgpt_ids {
            if verbose {
                println!("Closing old ChatGPT tab (ID: {})...", id);
            }
            let _ = call_mcp_tool(
                config_path,
                "close_page",
                serde_json::json!({
                    "pageId": id
                }),
            );
        }

        let refreshed_pages_res = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))?;
        let refreshed_text = refreshed_pages_res
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                format!(
                    "Invalid refreshed list_pages response structure: {:?}",
                    refreshed_pages_res
                )
            })?;
        let refreshed_pages = parse_pages(refreshed_text);

        if let Some(page) = refreshed_pages
            .iter()
            .find(|p| p.url.contains("chatgpt.com"))
        {
            if verbose {
                println!("Selecting new ChatGPT tab (ID: {})...", page.id);
            }
            call_mcp_tool(
                config_path,
                "select_page",
                serde_json::json!({
                    "pageId": page.id,
                    "bringToFront": !headless
                }),
            )?;

            for stale_page in refreshed_pages.iter().filter(|p| p.id != page.id) {
                if verbose {
                    println!("Closing non-ChatGPT tab (ID: {})...", stale_page.id);
                }
                let _ = call_mcp_tool(
                    config_path,
                    "close_page",
                    serde_json::json!({
                        "pageId": stale_page.id
                    }),
                );
            }
        }
    } else {
        // Find any existing chatgpt page
        let chatgpt_page = pages.iter().find(|p| p.url.contains("chatgpt.com"));

        match chatgpt_page {
            Some(page) => {
                if page.selected {
                    if verbose {
                        println!("Found ChatGPT tab (selected).");
                    }
                } else {
                    if verbose {
                        println!("Found ChatGPT tab (ID: {}). Selecting tab...", page.id);
                    }
                    call_mcp_tool(
                        config_path,
                        "select_page",
                        serde_json::json!({
                            "pageId": page.id,
                            "bringToFront": !headless
                        }),
                    )?;
                }
            }
            None => {
                // No ChatGPT tab. If there is only one blank tab, we navigate it. Otherwise, open a new page.
                if pages.len() == 1
                    && (pages[0].url == "about:blank"
                        || pages[0].url.contains("new-tab-page")
                        || pages[0].url.contains("chrome://welcome"))
                {
                    if verbose {
                        println!("Navigating existing blank tab to ChatGPT...");
                    }
                    call_mcp_tool(
                        config_path,
                        "navigate_page",
                        serde_json::json!({
                            "url": "https://chatgpt.com/"
                        }),
                    )?;
                } else {
                    if verbose {
                        println!("Opening a new tab for ChatGPT...");
                    }
                    call_mcp_tool(
                        config_path,
                        "new_page",
                        serde_json::json!({
                            "url": "https://chatgpt.com/"
                        }),
                    )?;
                }
            }
        }
    }

    // Wait for the prompt textarea to be present (the page is loaded)
    if verbose {
        println!("Waiting for ChatGPT to load...");
    }
    for attempt in 0..90 {
        if attempt > 0 && attempt % 10 == 0 {
            if let Ok(list_res) = call_mcp_tool(config_path, "list_pages", serde_json::json!({})) {
                if let Some(text) = list_res
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|obj| obj.get("text"))
                    .and_then(|t| t.as_str())
                {
                    if let Some(page) = parse_pages(text)
                        .iter()
                        .find(|p| p.url.contains("chatgpt.com"))
                    {
                        let _ = call_mcp_tool(
                            config_path,
                            "select_page",
                            serde_json::json!({
                                "pageId": page.id,
                                "bringToFront": !headless
                            }),
                        );
                    }
                }
            }
        }

        let ready_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": "() => document.getElementById('prompt-textarea') !== null"
            }),
        );
        let ready_res = match ready_res {
            Ok(res) => res,
            Err(e) => {
                if verbose {
                    eprintln!("Warning: Failed to check ChatGPT readiness: {}", e);
                }
                thread::sleep(Duration::from_millis(500));
                continue;
            }
        };
        if let Ok(parsed) = parse_script_result(&ready_res) {
            let is_ready = parsed.as_bool().unwrap_or(false);
            if is_ready {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err("Timeout waiting for ChatGPT page to load".to_string())
}

fn check_login_status(config_path: &str) -> Result<bool, String> {
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": "() => document.querySelector('[data-testid=\"login-button\"]') === null"
        }),
    )?;

    if let Ok(parsed) = parse_script_result(&res) {
        Ok(parsed.as_bool().unwrap_or(false))
    } else {
        Ok(false)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if !cli.verbose {
        // SAFETY: Called before spawning other threads and before loading MCP config.
        unsafe {
            std::env::remove_var("MCP_DEBUG");
        }
    }
    if std::env::var("MCP_TIMEOUT").is_err() {
        // SAFETY: Called before spawning other threads and before loading MCP config.
        unsafe {
            std::env::set_var("MCP_TIMEOUT", "20");
        }
    }

    let is_terminal = io::stdout().is_terminal();
    let use_glow = is_terminal && is_glow_available();

    let is_headless = match &cli.command {
        Some(Commands::Login) => false, // Force headful only for login command so user can see it to log in
        _ => cli.headless, // Respect --headless (defaults to true) for all other commands (including Open and Get)
    };

    let config_path = match write_mcp_config(!cli.verbose, is_headless) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = start_chrome_if_needed(is_headless, cli.verbose) {
        eprintln!("Error starting Chrome: {}", e);
        std::process::exit(1);
    }

    if let Some(command) = cli.command {
        match command {
            Commands::Open { url } => {
                if let Some(url) = url {
                    if let Err(e) = open_url_tab(&config_path, &url, is_headless, cli.verbose) {
                        eprintln!("Error opening URL: {}", e);
                        std::process::exit(1);
                    }

                    match copy_latest_markdown(&config_path) {
                        Ok(markdown) => {
                            if let Some(ref output_path) = cli.output {
                                if let Err(e) = std::fs::write(output_path, &markdown) {
                                    eprintln!("Error writing output file: {}", e);
                                    std::process::exit(1);
                                }
                            }
                            if let Err(e) = render_markdown(&markdown, use_glow) {
                                eprintln!("Error rendering Markdown: {}", e);
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error copying latest response Markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    if let Err(e) = ensure_chatgpt_tab(&config_path, false, is_headless, cli.verbose) {
                        eprintln!("Error ensuring ChatGPT tab: {}", e);
                        std::process::exit(1);
                    }
                    println!("Successfully opened ChatGPT!");
                }
                return Ok(());
            }
            Commands::Get { url } => {
                if let Some(url) = url {
                    if let Err(e) = open_url_tab(&config_path, &url, is_headless, cli.verbose) {
                        eprintln!("Error opening URL: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    if let Err(e) = ensure_chatgpt_tab(&config_path, false, is_headless, cli.verbose) {
                        eprintln!("Error ensuring ChatGPT tab: {}", e);
                        std::process::exit(1);
                    }
                }

                match copy_latest_markdown(&config_path) {
                    Ok(markdown) => {
                        if let Some(ref output_path) = cli.output {
                            if let Err(e) = std::fs::write(output_path, &markdown) {
                                eprintln!("Error writing output file: {}", e);
                                std::process::exit(1);
                            }
                        }
                        if let Err(e) = render_markdown(&markdown, use_glow) {
                            eprintln!("Error rendering Markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error copying latest response Markdown: {}", e);
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }
            Commands::Login => {
                if let Err(e) = ensure_chatgpt_tab(&config_path, false, is_headless, cli.verbose) {
                    eprintln!("Error ensuring ChatGPT tab: {}", e);
                    std::process::exit(1);
                }
                println!("\n========================================================");
                println!("Please complete the login manually in the Chrome window.");
                println!("Once you have successfully logged in, press [Enter] here.");
                println!("========================================================\n");

                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;

                match check_login_status(&config_path) {
                    Ok(true) => println!(
                        "Success: Logged in successfully! You can now use the `ask` command."
                    ),
                    _ => println!(
                        "Warning: We still detected a login button on the page. You might not be fully logged in. Please verify."
                    ),
                }
                return Ok(());
            }
        }
    }

    // Read prompt from arguments or stdin
    let mut prompt = String::new();
    if let Some(p) = cli.prompt {
        prompt = p;
    } else {
        // Check if stdin is a pipe (not a tty)
        if !std::io::stdin().is_terminal() {
            io::stdin().read_to_string(&mut prompt)?;
        }
    }

    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        // No prompt and no command, print help
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        std::process::exit(0);
    }

    if let Err(e) = ensure_chatgpt_tab(&config_path, cli.new, is_headless, cli.verbose) {
        eprintln!("Error ensuring ChatGPT tab: {}", e);
        std::process::exit(1);
    }

    // Verify login
    match check_login_status(&config_path) {
        Ok(false) => {
            eprintln!("\nError: You are not logged in to ChatGPT.");
            eprintln!(
                "Please run `ask login` to log in manually first, and then run your query again.\n"
            );
            std::process::exit(1);
        }
        Err(e) if cli.verbose => {
            eprintln!(
                "Warning: Failed to verify login status: {}. Attempting to proceed...",
                e
            );
        }
        _ => {}
    }

    // Get initial number of assistant messages before submitting the prompt
    let count_res = call_mcp_tool(
        &config_path,
        "evaluate_script",
        serde_json::json!({
            "function": "() => document.querySelectorAll('[data-message-author-role=\"assistant\"]').length"
        }),
    )?;
    let initial_assistant_count = parse_script_result(&count_res)
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    if cli.verbose {
        println!("Focusing input field...");
    }
    call_mcp_tool(
        &config_path,
        "evaluate_script",
        serde_json::json!({
            "function": "() => { const el = document.getElementById('prompt-textarea'); if (el) { el.focus(); return true; } return false; }"
        }),
    )?;

    if cli.verbose {
        println!("Typing your prompt...");
    }
    call_mcp_tool(
        &config_path,
        "type_text",
        serde_json::json!({
            "text": prompt
        }),
    )?;

    thread::sleep(Duration::from_millis(500));

    if cli.verbose {
        println!("Submitting...");
    }
    call_mcp_tool(
        &config_path,
        "evaluate_script",
        serde_json::json!({
            "function": "() => { const btn = document.querySelector('[data-testid=\"send-button\"]') || document.getElementById('composer-submit-button'); if (btn) { btn.click(); return true; } return false; }"
        }),
    )?;

    if cli.verbose {
        println!("Waiting for ChatGPT response...");
    }

    let mut last_markdown = String::new();
    let mut finished = false;
    let mut wait_cycles = 0;
    let mut stable_done_checks = 0;
    let spinner_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0;

    while !finished && wait_cycles < 1200 {
        // Max 120 seconds (1200 * 100ms)
        if is_terminal {
            let frame = spinner_frames[spinner_idx % spinner_frames.len()];
            print!("\r\x1b[1;36m{}\x1b[0m 正在等待 ChatGPT 回應...", frame);
            io::stdout().flush()?;
            spinner_idx += 1;
        }

        if wait_cycles % 5 == 0 {
            let check_res = match call_mcp_tool(
                &config_path,
                "evaluate_script",
                serde_json::json!({
                    "function": format!(r#"() => {{
                    const stopButton = document.querySelector('[data-testid="stop-button"]') || 
                                       document.getElementById('composer-stop-button') ||
                                       document.querySelector('button[aria-label="Stop generating"]');

                    const isVisible = (el) => {{
                        if (!el || el.disabled || el.getAttribute('aria-disabled') === 'true') return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    }};
                    
                    const messages = document.querySelectorAll('[data-message-author-role="assistant"]');
                    const isNew = messages.length > {initial_assistant_count};
                    
                    if (isVisible(stopButton)) {{
                        return {{ status: "generating", isNew: isNew }};
                    }}
                    
                    if (isNew) {{
                        return {{ status: "done", isNew: isNew }};
                    }}
                    
                    return {{ status: "waiting", isNew: isNew }};
                }}"#, initial_assistant_count = initial_assistant_count)
                }),
            ) {
                Ok(res) => res,
                Err(e) => {
                    if cli.verbose {
                        eprintln!("Warning: Failed to poll ChatGPT response: {}", e);
                    }
                    thread::sleep(Duration::from_millis(100));
                    wait_cycles += 1;
                    continue;
                }
            };

            if let Ok(parsed) = parse_script_result(&check_res) {
                let status = parsed["status"].as_str().unwrap_or("waiting");
                let is_new = parsed["isNew"].as_bool().unwrap_or(false);

                if status == "done" && is_new {
                    stable_done_checks += 1;
                    if stable_done_checks >= 3 {
                        finished = true;
                    }
                } else {
                    stable_done_checks = 0;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
        wait_cycles += 1;
    }

    if is_terminal {
        print!("\r\x1b[K");
        io::stdout().flush()?;
    }

    if !finished {
        eprintln!("\nWarning: Output stream did not complete within the timeout period.");
    }

    if finished {
        if cli.verbose {
            println!("Copying final response from ChatGPT toolbar...");
        }
        match copy_latest_markdown(&config_path) {
            Ok(content) => {
                last_markdown = content;
            }
            Err(e) => {
                eprintln!("Error copying response from ChatGPT toolbar: {}", e);
            }
        }
    }

    if let Err(e) = render_markdown(&last_markdown, use_glow) {
        eprintln!("Error rendering Markdown: {}", e);
    }

    // Print the URL link of the current conversation thread
    let url_opt = call_mcp_tool(
        &config_path,
        "evaluate_script",
        serde_json::json!({
            "function": "() => window.location.href"
        }),
    )
    .ok()
    .and_then(|url_val| parse_script_result(&url_val).ok())
    .and_then(|u| u.as_str().map(|s| s.to_string()));

    if let Some(url) = url_opt {
        if is_terminal {
            println!("\n🌐 \x1b[1mThread Link:\x1b[0m \x1b[4;36m{}\x1b[0m", url);
        } else {
            println!("\nThread Link: {}", url);
        }
    }

    if let Some(ref output_path) = cli.output {
        if let Err(e) = std::fs::write(output_path, &last_markdown) {
            eprintln!("Error writing output file: {}", e);
        } else if cli.verbose {
            println!("Successfully wrote Markdown response to {}", output_path);
        }
    }

    Ok(())
}
