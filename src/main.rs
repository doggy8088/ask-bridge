use base64::{Engine as _, engine::general_purpose};
use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use mcp_cli::McpClient;
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::io::{self, IsTerminal, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const ASK_BRIDGE_CHROME_MARKER: &str = "--ask-bridge-instance";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoginState {
    LoggedIn,
    LoggedOut,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct LoginSignals {
    account: bool,
    auth_control: bool,
    auth_path: bool,
    composer: bool,
}

impl LoginSignals {
    fn state(self) -> LoginState {
        match (
            self.account,
            self.auth_control || self.auth_path,
            self.composer,
        ) {
            (true, _, _) => LoginState::LoggedIn,
            (false, true, _) => LoginState::LoggedOut,
            (false, false, _) => LoginState::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Provider {
    #[value(name = "chatgpt")]
    ChatGpt,
    #[value(name = "gemini")]
    Gemini,
}

impl Provider {
    fn from_config_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "chatgpt" | "chat-gpt" | "chat_gpt" => Some(Provider::ChatGpt),
            "gemini" => Some(Provider::Gemini),
            _ => None,
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Provider::ChatGpt => "ChatGPT",
            Provider::Gemini => "Gemini",
        }
    }

    fn home_url(self) -> &'static str {
        match self {
            Provider::ChatGpt => "https://chatgpt.com/",
            Provider::Gemini => "https://gemini.google.com/app",
        }
    }

    fn owns_url(self, url: &str) -> bool {
        match self {
            Provider::ChatGpt => url.contains("chatgpt.com"),
            Provider::Gemini => url.contains("gemini.google.com"),
        }
    }

    fn from_url(url: &str) -> Option<Self> {
        [Provider::ChatGpt, Provider::Gemini]
            .into_iter()
            .find(|provider| provider.owns_url(url))
    }

    fn ready_check_js(self) -> &'static str {
        match self {
            Provider::ChatGpt => r#"() => document.getElementById('prompt-textarea') !== null"#,
            Provider::Gemini => {
                r#"() => {
                    return document.querySelector('div[role="textbox"][aria-label*="Gemini"]') !== null ||
                           document.querySelector('rich-textarea [contenteditable="true"]') !== null ||
                           document.querySelector('.ql-editor[contenteditable="true"]') !== null ||
                           document.querySelector('a[href*="accounts.google.com"]') !== null ||
                           /Sign in|登入/.test(document.body.innerText || '');
                }"#
            }
        }
    }

    fn login_signals_js(self) -> &'static str {
        match self {
            Provider::ChatGpt => {
                r#"() => {
                    const isVisible = (el) => {
                        if (!el) return false;
                        const style = window.getComputedStyle(el);
                        const rect = el.getBoundingClientRect();
                        return style.display !== 'none' &&
                            style.visibility !== 'hidden' &&
                            style.opacity !== '0' &&
                            rect.width > 0 &&
                            rect.height > 0;
                    };

                    const textFor = (el) => [
                        el.getAttribute('aria-label'),
                        el.getAttribute('title'),
                        el.textContent
                    ].filter(Boolean).join(' ').trim();

                    const visibleAuthButton = Array.from(document.querySelectorAll('a, button'))
                        .some((el) => {
                            if (!isVisible(el)) return false;
                            const text = textFor(el);
                            return /^(log in|login|sign in|sign up|登入|登錄|登录|註冊|注册)$/i.test(text);
                        });

                    const composer = document.querySelector('#prompt-textarea') ||
                        document.querySelector('[data-testid="composer-text-input"]') ||
                        document.querySelector('textarea[placeholder*="Message"]') ||
                        document.querySelector('textarea[placeholder*="訊息"]') ||
                        document.querySelector('[contenteditable="true"]');

                    const accountMenu = document.querySelector('[data-testid="profile-button"]') ||
                        document.querySelector('[data-testid="account-menu-button"]') ||
                        document.querySelector('[data-testid="user-menu-button"]') ||
                        document.querySelector('button[aria-label*="Profile"]') ||
                        document.querySelector('button[aria-label*="profile"]') ||
                        document.querySelector('button[aria-label*="Account"]') ||
                        document.querySelector('button[aria-label*="account"]') ||
                        document.querySelector('button[aria-label*="User"]') ||
                        document.querySelector('button[aria-label*="user"]') ||
                        document.querySelector('button[aria-label*="帳戶"]') ||
                        document.querySelector('button[aria-label*="使用者"]');

                    const authPath = /\/(auth|login|signup)(\/|$)/i.test(window.location.pathname);
                    return {
                        account: isVisible(accountMenu),
                        auth_control: Boolean(visibleAuthButton),
                        auth_path: authPath,
                        composer: Boolean(composer)
                    };
                }"#
            }
            Provider::Gemini => {
                r#"() => {
                    const isVisible = (el) => {
                        if (!el) return false;
                        const style = window.getComputedStyle(el);
                        const rect = el.getBoundingClientRect();
                        return style.display !== 'none' &&
                            style.visibility !== 'hidden' &&
                            style.opacity !== '0' &&
                            rect.width > 0 &&
                            rect.height > 0;
                    };
                    const composer = document.querySelector('div[role="textbox"][aria-label*="Gemini"]') ||
                        document.querySelector('rich-textarea [contenteditable="true"]') ||
                        document.querySelector('.ql-editor[contenteditable="true"]');
                    const account = document.querySelector('a[href*="accounts.google.com/SignOutOptions"]') ||
                        document.querySelector('[aria-label*="Google 帳戶"]') ||
                        document.querySelector('[aria-label*="Google Account"]');
                    const signIn = Array.from(document.querySelectorAll('a, button'))
                        .some((el) => isVisible(el) && /Sign in|登入/.test([
                                el.getAttribute('aria-label'),
                                el.textContent
                            ].filter(Boolean).join(' ')));
                    const authPath = /\/(auth|login|signin|signup)(\/|$)/i.test(window.location.pathname);
                    return {
                        account: isVisible(account),
                        auth_control: Boolean(signIn),
                        auth_path: authPath,
                        composer: Boolean(composer)
                    };
                }"#
            }
        }
    }

    fn assistant_selector(self) -> &'static str {
        match self {
            Provider::ChatGpt => "[data-message-author-role=\"assistant\"], .agent-turn",
            Provider::Gemini => "model-response",
        }
    }

    fn latest_response_selector(self) -> &'static str {
        match self {
            Provider::ChatGpt => {
                "[data-message-author-role=\"assistant\"], .agent-turn, model-response, .model-response, [data-test-id*=\"response\"], [data-testid*=\"response\"]"
            }
            Provider::Gemini => "model-response",
        }
    }

    fn response_content_selector(self) -> &'static str {
        match self {
            Provider::ChatGpt => "",
            Provider::Gemini => {
                "message-content, .markdown, structured-content-container.model-response-text"
            }
        }
    }

    fn composer_selectors_json(self) -> &'static str {
        match self {
            Provider::ChatGpt => r##"["#prompt-textarea"]"##,
            Provider::Gemini => {
                r#"[
                    "div[role=\"textbox\"][aria-label*=\"Gemini\"]",
                    "rich-textarea [contenteditable=\"true\"]",
                    ".ql-editor[contenteditable=\"true\"]"
                ]"#
            }
        }
    }

    fn send_button_selectors_json(self) -> &'static str {
        match self {
            Provider::ChatGpt => {
                r##"[
                    "[data-testid=\"send-button\"]",
                    "#composer-submit-button",
                    "button[aria-label*=\"Send\"]",
                    "button[aria-label*=\"傳送\"]",
                    "button[aria-label*=\"发送\"]"
                ]"##
            }
            Provider::Gemini => {
                r#"[
                    "button[aria-label=\"傳送訊息\"]",
                    "button[aria-label=\"Submit\"]",
                    "button[aria-label*=\"Send\"]",
                    "button[aria-label*=\"傳送\"]",
                    "button[aria-label*=\"提交\"]"
                ]"#
            }
        }
    }

    fn stop_button_selectors_json(self) -> &'static str {
        match self {
            Provider::ChatGpt => {
                r##"[
                    "[data-testid=\"stop-button\"]",
                    "#composer-stop-button",
                    "button[aria-label=\"Stop generating\"]"
                ]"##
            }
            Provider::Gemini => {
                r#"[
                    "button[aria-label=\"停止回覆\"]",
                    "button[aria-label*=\"Stop\"]",
                    "button[aria-label*=\"停止\"]"
                ]"#
            }
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::ChatGpt => write!(f, "chatgpt"),
            Provider::Gemini => write!(f, "gemini"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ChatGptAgentPrompt<'a> {
    agent_mention: &'a str,
    body: &'a str,
}

fn parse_chatgpt_agent_prompt(prompt: &str) -> Option<ChatGptAgentPrompt<'_>> {
    let rest = prompt.strip_prefix('@')?;
    let mut agent_chars = 0usize;

    for (idx, ch) in rest.char_indices() {
        if ch.is_whitespace() {
            if agent_chars == 0 || agent_chars > 10 {
                return None;
            }

            let body = rest[idx + ch.len_utf8()..].trim_start_matches(char::is_whitespace);
            if body.is_empty() {
                return None;
            }

            return Some(ChatGptAgentPrompt {
                agent_mention: &prompt[..idx + 1],
                body,
            });
        }

        agent_chars += 1;
        if agent_chars > 10 {
            return None;
        }
    }

    None
}

#[derive(Parser)]
#[command(name = "ask-bridge")]
#[command(version = "0.2.1")]
#[command(disable_version_flag = true)]
#[command(about = "AI browser CLI - Ask ChatGPT or Gemini from your Terminal with your subscription", long_about = None)]
struct Cli {
    /// The prompt to send to the selected provider.
    /// If standard input is piped and this value is present, they are combined as:
    /// `prompt + "\\n\\n" + stdin`.
    prompt: Option<String>,

    /// AI provider to automate. Overrides ~/.config/ask-bridge/config.json.
    #[arg(long, short = 'p', value_enum, global = true)]
    provider: Option<Provider>,

    /// Run Chrome in headless mode. Defaults to true.
    #[arg(long, require_equals = true, num_args = 0..=1, default_value = "true", default_missing_value = "true")]
    headless: bool,

    /// Create a brand new provider session by opening a new tab and closing old ones.
    #[arg(long, default_value_t = false)]
    new: bool,

    /// Print version information.
    #[arg(
        long = "version",
        short = 'v',
        short_alias = 'V',
        action = ArgAction::Version
    )]
    _version: (),

    /// Print verbose debugging status messages.
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Write the final response in Markdown format to the specified file.
    #[arg(long, short, value_name = "FILE")]
    output: Option<String>,

    /// Write the downloaded images to the specified folder or file path.
    #[arg(long, short = 'i', value_name = "IMAGE_PATH")]
    image_output: Option<String>,

    /// Attach one or more local image files to the prompt (can be specified multiple times).
    #[arg(long = "image", value_name = "IMAGE_FILE", num_args = 1)]
    images: Vec<String>,

    /// Attach one or more local document files (PDF, Word, Excel, text, etc.) to the prompt
    /// (can be specified multiple times).
    #[arg(long = "file", value_name = "FILE", num_args = 1)]
    files: Vec<String>,

    /// Switch the provider model before sending the prompt.
    /// ChatGPT examples: "GPT-5.5", "GPT-5.4", "GPT-5.3", "o3", or thinking levels such as
    /// "即時", "中等", "高", "超高", "專業", "智慧". Gemini examples: "3.5 Flash",
    /// "3.1 Flash-Lite", or "3.1 Pro". Matching is case- and punctuation-insensitive.
    #[arg(long = "model", value_name = "MODEL")]
    model: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Open Chrome browser, optionally navigate to a URL, and copy the latest response
    #[command(hide = true)]
    Open {
        /// Optional conversation URL to open before copying the latest response.
        url: Option<String>,
    },
    /// Retrieve the latest response from the selected provider (defaults to headless)
    #[command(hide = true)]
    Get {
        /// Optional conversation URL to fetch before copying the latest response.
        url: Option<String>,
    },
    /// Open Chrome browser and wait for manual login
    Login,
    /// Close the managed Chrome browser instance
    Close,
    /// Set or show the global default provider used when --provider is not specified.
    Config,
    /// Dump the current browser tab HTML for debugging
    #[command(hide = true)]
    Dump,
    /// Take a screenshot of the current browser tab for debugging
    #[command(hide = true)]
    Screenshot,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AppConfig {
    provider: Option<String>,
}

fn config_file_path() -> Result<PathBuf, String> {
    let mut config_path = home::home_dir().ok_or("Could not locate home directory")?;
    config_path.push(".config/ask-bridge/config.json");
    Ok(config_path)
}

fn parse_configured_provider(content: &str) -> Result<Option<Provider>, String> {
    let config: AppConfig =
        serde_json::from_str(content).map_err(|e| format!("Failed to parse config.json: {}", e))?;

    match config.provider {
        Some(provider) => Provider::from_config_value(&provider)
            .map(Some)
            .ok_or_else(|| format!("Invalid provider in config.json: {}", provider)),
        None => Ok(None),
    }
}

fn load_configured_provider() -> Result<Option<Provider>, String> {
    let config_path = config_file_path()?;
    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        format!(
            "Failed to read config file {}: {}",
            config_path.to_string_lossy(),
            e
        )
    })?;

    parse_configured_provider(&content).map_err(|e| {
        format!(
            "{}. Expected format: {{\"provider\":\"chatgpt\"}} or {{\"provider\":\"gemini\"}}",
            e
        )
    })
}

fn effective_provider(
    cli_provider: Option<Provider>,
    configured_provider: Option<Provider>,
) -> Provider {
    cli_provider
        .or(configured_provider)
        .unwrap_or(Provider::ChatGpt)
}

fn resolve_provider_with<F>(
    cli_provider: Option<Provider>,
    load_provider: F,
) -> Result<Provider, String>
where
    F: FnOnce() -> Result<Option<Provider>, String>,
{
    if let Some(provider) = cli_provider {
        return Ok(provider);
    }

    Ok(effective_provider(None, load_provider()?))
}

fn resolve_provider(cli_provider: Option<Provider>) -> Result<Provider, String> {
    resolve_provider_with(cli_provider, load_configured_provider)
}

fn write_global_provider_config(provider: Provider) -> Result<(), String> {
    let config_path = config_file_path()?;
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create config directory {}: {}",
                parent.to_string_lossy(),
                e
            )
        })?;
    }

    let content =
        serde_json::to_string_pretty(&serde_json::json!({"provider": provider.to_string()}))
            .map_err(|e| format!("Failed to serialize provider config: {}", e))?;
    std::fs::write(&config_path, format!("{}\n", content)).map_err(|e| {
        format!(
            "Failed to write config file {}: {}",
            config_path.to_string_lossy(),
            e
        )
    })?;

    println!(
        "Set default provider to '{}' in {}",
        provider,
        config_path.to_string_lossy()
    );

    Ok(())
}

fn run_config_command(cli_provider: Option<Provider>) -> Result<(), String> {
    match cli_provider {
        Some(provider) => write_global_provider_config(provider),
        None => {
            let config_path = config_file_path()?;
            let configured_provider = load_configured_provider()?;
            match configured_provider {
                Some(provider) => {
                    println!("Current default provider: {}", provider);
                }
                None => {
                    println!("No default provider configured.");
                    println!("The effective provider is ChatGPT.");
                }
            }
            if config_path.exists() {
                println!("Config file: {}", config_path.to_string_lossy());
            } else {
                println!(
                    "Config file not created yet: {}",
                    config_path.to_string_lossy()
                );
            }
            println!("Set default provider with: ask-bridge config --provider <chatgpt|gemini>");
            println!("This is a one-time override example: ask-bridge --provider gemini <prompt>");
            Ok(())
        }
    }
}

struct Page {
    id: usize,
    url: String,
    selected: bool,
}

#[derive(Clone, Copy, Debug)]
struct PageLoginState {
    id: usize,
    selected: bool,
    login_state: LoginState,
}

fn preferred_provider_page_id(pages: &[PageLoginState]) -> Option<usize> {
    pages
        .iter()
        .find(|page| page.login_state == LoginState::LoggedIn)
        .or_else(|| pages.iter().find(|page| page.selected))
        .or_else(|| pages.first())
        .map(|page| page.id)
}

fn write_mcp_config(quiet_mcp: bool, headless: bool) -> Result<String, String> {
    let mut config_dir = home::home_dir().ok_or("Could not locate home directory")?;
    config_dir.push(".config/ask-bridge");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let log_path = config_dir
        .join("chrome-devtools-mcp.log")
        .to_string_lossy()
        .to_string();

    config_dir.push("mcp_servers.json");
    let config_path = config_dir.to_string_lossy().to_string();

    let mut chrome_devtools_server = if quiet_mcp && !cfg!(target_os = "windows") {
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
            "command": if cfg!(target_os = "windows") { "npx.cmd" } else { "npx" },
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

fn chrome_profile_path() -> Result<String, String> {
    let mut profile_dir = home::home_dir().ok_or("Could not locate home directory")?;
    profile_dir.push(".config/ask-bridge/chrome-profile");
    std::fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to create chrome profile directory: {}", e))?;

    Ok(profile_dir.to_string_lossy().to_string())
}

fn chrome_pid_path() -> Result<PathBuf, String> {
    let mut path = home::home_dir().ok_or("Could not locate home directory")?;
    path.push(".config/ask-bridge/chrome.pid");
    Ok(path)
}

fn write_chrome_pid(pid: u32) -> Result<(), String> {
    let path = chrome_pid_path()?;
    std::fs::write(&path, pid.to_string())
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

fn read_chrome_pid() -> Option<String> {
    let path = chrome_pid_path().ok()?;
    let pid = std::fs::read_to_string(path).ok()?;
    let pid = pid.trim();
    if pid.is_empty() {
        None
    } else {
        Some(pid.to_string())
    }
}

fn remove_chrome_pid_file() -> Result<(), String> {
    let path = chrome_pid_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

#[cfg(any(target_os = "linux", test))]
const LINUX_CHROME_COMMANDS: &[&str] = &["google-chrome", "google-chrome-stable"];

#[cfg(any(target_os = "linux", test))]
fn first_existing_path(paths: &[&str]) -> Option<String> {
    paths
        .iter()
        .find(|path| Path::new(path).exists())
        .map(|path| (*path).to_string())
}

#[cfg(any(target_os = "linux", test))]
fn find_command_in_path(command: &str, path_env: Option<&std::ffi::OsStr>) -> Option<String> {
    let path_env = path_env?;

    std::env::split_paths(path_env)
        .map(|dir| dir.join(command))
        .find(|path| path.exists())
        .map(|path| path.to_string_lossy().to_string())
}

#[cfg(any(target_os = "linux", test))]
fn find_chrome_command_in_path(path_env: Option<&std::ffi::OsStr>) -> Option<String> {
    LINUX_CHROME_COMMANDS
        .iter()
        .find_map(|command| find_command_in_path(command, path_env))
}

#[cfg(any(target_os = "linux", test))]
fn find_linux_chrome_path(
    path_env: Option<&std::ffi::OsStr>,
    path_candidates: &[&str],
) -> Option<String> {
    find_chrome_command_in_path(path_env).or_else(|| first_existing_path(path_candidates))
}

fn find_chrome_path() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Program Files
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let path = format!(r"{}\Google\Chrome\Application\chrome.exe", pf);
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        } else {
            let path = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // 2. Program Files (x86)
        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            let path = format!(r"{}\Google\Chrome\Application\chrome.exe", pf86);
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        } else {
            let path = r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe";
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // 3. LocalAppData
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let path = format!(r"{}\Google\Chrome\Application\chrome.exe", local_app_data);
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        }

        Err("Google Chrome was not found in standard Windows installation paths. Please install Google Chrome.".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
        if std::path::Path::new(path).exists() {
            Ok(path.to_string())
        } else {
            Err("Google Chrome not found at /Applications/Google Chrome.app".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        const LINUX_CHROME_PATHS: &[&str] = &[
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/local/bin/google-chrome",
            "/usr/local/bin/google-chrome-stable",
            "/opt/google/chrome/google-chrome",
        ];

        let path_env = std::env::var_os("PATH");
        find_linux_chrome_path(path_env.as_deref(), LINUX_CHROME_PATHS).ok_or_else(|| {
            "Google Chrome was not found in PATH or standard Linux installation paths. Please install Google Chrome or add google-chrome to PATH.".to_string()
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Google Chrome auto-detection is not supported on this operating system. Please use macOS, Windows, or Linux.".to_string())
    }
}

fn start_chrome_if_needed(headless: bool, verbose: bool) -> Result<(), String> {
    let profile_path = chrome_profile_path()?;

    if TcpStream::connect("127.0.0.1:9223").is_ok() {
        let ask_pids = ask_chrome_pids_on_debug_port(&profile_path);
        if !ask_pids.is_empty() {
            if let Some(pid) = ask_pids.first().and_then(|pid| pid.parse::<u32>().ok()) {
                let _ = write_chrome_pid(pid);
            }
            if headless {
                // Force hide any existing background Chrome PIDs asynchronously just in case they are currently visible
                #[cfg(target_os = "macos")]
                {
                    let pids = ask_pids.clone();
                    thread::spawn(move || {
                        for pid_str in pids {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                let script = format!(
                                    "tell application \"System Events\" to set visible of first application process whose unix id is {} to false",
                                    pid
                                );
                                let _ = Command::new("osascript").arg("-e").arg(&script).status();
                            }
                        }
                    });
                }
            }
            if verbose && headless && !is_debug_chrome_background(&profile_path) {
                println!(
                    "Reusing existing ask-bridge Chrome on port 9223. Run `ask-bridge close` if you want to restart it in background mode."
                );
            }
            return Ok(());
        }

        if debug_port_listener_pids().is_empty() {
            if verbose {
                println!(
                    "Port 9223 is listening, but ask-bridge could not identify the listener process. Reusing it."
                );
            }
            return Ok(());
        }

        return Err(
            "Port 9223 is already used by a non-ask Chrome process. Stop it or use a different debugging port."
                .to_string(),
        );
    }

    if verbose {
        println!(
            "Chrome is not running on port 9223. Starting Chrome with remote debugging (headless: {})...",
            headless
        );
    }

    let chrome_path = find_chrome_path()?;

    let mut cmd = Command::new(&chrome_path);
    cmd.arg("--remote-debugging-port=9223")
        .arg(format!("--user-data-dir={}", profile_path))
        .arg(ASK_BRIDGE_CHROME_MARKER)
        .arg("--no-first-run")
        .arg("--no-default-browser-check");

    #[cfg(target_os = "windows")]
    {
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    if headless {
        cmd.arg("--ask-bridge-background")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--window-size=1440,1200")
            .arg("--window-position=-2000,-2000");
    }

    let child = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start Google Chrome: {}", e))?;

    let child_pid = child.id();
    if let Err(e) = write_chrome_pid(child_pid)
        && verbose
    {
        eprintln!("Warning: Failed to record Chrome PID: {}", e);
    }

    if verbose {
        println!(
            "Started ask-bridge Chrome PID {} with profile {}.",
            child_pid, profile_path
        );
    }

    if headless {
        #[cfg(target_os = "macos")]
        {
            let pid = child.id();
            thread::spawn(move || {
                // Rapidly set visibility to false during startup to prevent window from flashing or drawing
                for _ in 0..40 {
                    let script = format!(
                        "tell application \"System Events\" to try\nset visible of first application process whose unix id is {} to false\nend try",
                        pid
                    );
                    let _ = Command::new("osascript").arg("-e").arg(&script).status();
                    thread::sleep(Duration::from_millis(50));
                }
            });
        }
    }

    let _ = child; // Avoid unused variable warning on non-macOS platforms

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

    let _ = remove_chrome_pid_file();
    Err("Timed out waiting for Chrome to start on port 9223".to_string())
}

fn normalize_profile_match_text(value: &str) -> String {
    let normalized = value.replace('\\', "/").replace(['"', '\''], "");

    #[cfg(target_os = "windows")]
    {
        normalized.to_ascii_lowercase()
    }

    #[cfg(not(target_os = "windows"))]
    {
        normalized
    }
}

fn command_uses_profile(command: &str, profile_path: &str) -> bool {
    let command = normalize_profile_match_text(command);
    let profile_path = normalize_profile_match_text(profile_path);

    command.contains(&format!("--user-data-dir={}", profile_path))
        || command.contains(&format!("--user-data-dir {}", profile_path))
}

fn command_identifies_ask_chrome(command: &str, profile_path: &str) -> bool {
    command_uses_profile(command, profile_path) || command.contains(ASK_BRIDGE_CHROME_MARKER)
}

fn find_ask_chrome_owner_pid_with<C, P>(
    listener_pid: &str,
    profile_path: &str,
    mut command_for: C,
    mut parent_for: P,
) -> Option<String>
where
    C: FnMut(&str) -> Option<String>,
    P: FnMut(&str) -> Option<String>,
{
    let mut current_pid = listener_pid.to_string();

    for _ in 0..16 {
        if command_for(&current_pid)
            .map(|command| command_identifies_ask_chrome(&command, profile_path))
            .unwrap_or(false)
        {
            return Some(current_pid);
        }

        let parent_pid = parent_for(&current_pid)?;
        if parent_pid.is_empty() || parent_pid == "0" || parent_pid == current_pid {
            return None;
        }
        current_pid = parent_pid;
    }

    None
}

fn find_ask_chrome_owner_pid(listener_pid: &str, profile_path: &str) -> Option<String> {
    find_ask_chrome_owner_pid_with(
        listener_pid,
        profile_path,
        process_command,
        process_parent_pid,
    )
}

fn ask_chrome_pids_on_debug_port(profile_path: &str) -> Vec<String> {
    let mut owner_pids = Vec::new();
    for listener_pid in debug_port_listener_pids() {
        if let Some(owner_pid) = find_ask_chrome_owner_pid(&listener_pid, profile_path)
            && !owner_pids.contains(&owner_pid)
        {
            owner_pids.push(owner_pid);
        }
    }
    owner_pids
}

fn debug_port_listener_pids() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netstat").args(["-ano", "-p", "tcp"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut pids = Vec::new();
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.contains(":9223") && line.contains("LISTENING") {
                        if let Some(pid) = line.split_whitespace().last() {
                            if !pids.contains(&pid.to_string()) {
                                pids.push(pid.to_string());
                            }
                        }
                    }
                }
                pids
            }
            _ => Vec::new(),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
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
}

fn process_command(pid: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("processid={}", pid),
                "get",
                "commandline",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut cmd_lines = stdout.lines().skip(1);
                if let Some(cmd) = cmd_lines.next() {
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        return Some(cmd.to_string());
                    }
                }
            }
        }

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-CimInstance Win32_Process -Filter 'ProcessId = {}').CommandLine",
                    pid
                ),
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !stdout.is_empty() {
                    return Some(stdout);
                }
            }
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("ps")
            .args(["-p", pid, "-o", "command="])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn process_parent_pid(pid: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("processid={}", pid),
                "get",
                "parentprocessid",
            ])
            .output();

        if let Ok(out) = output
            && out.status.success()
            && let Some(parent_pid) = String::from_utf8_lossy(&out.stdout)
                .lines()
                .skip(1)
                .map(str::trim)
                .find(|line| !line.is_empty())
        {
            return Some(parent_pid.to_string());
        }

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-CimInstance Win32_Process -Filter 'ProcessId = {}').ParentProcessId",
                    pid
                ),
            ])
            .output();

        if let Ok(out) = output
            && out.status.success()
        {
            let parent_pid = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !parent_pid.is_empty() {
                return Some(parent_pid);
            }
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("ps")
            .args(["-p", pid, "-o", "ppid="])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let parent_pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if parent_pid.is_empty() {
            None
        } else {
            Some(parent_pid)
        }
    }
}

fn is_debug_chrome_background(profile_path: &str) -> bool {
    ask_chrome_pids_on_debug_port(profile_path)
        .iter()
        .any(|pid| {
            process_command(pid)
                .map(|cmd| cmd.contains("--ask-bridge-background"))
                .unwrap_or(false)
        })
}

fn close_ask_chrome_on_debug_port(profile_path: &str) -> Result<bool, String> {
    let listener_pids = debug_port_listener_pids();
    if listener_pids.is_empty() {
        let _ = remove_chrome_pid_file();
        return Ok(false);
    }

    let ask_pids: Vec<String> = listener_pids
        .into_iter()
        .filter(|pid| {
            process_command(pid)
                .map(|cmd| command_uses_profile(&cmd, profile_path))
                .unwrap_or(false)
        })
        .collect();

    if ask_pids.is_empty() {
        return Err(
            "Port 9223 is already used by a non-ask Chrome process. Stop it or use a different debugging port."
                .to_string(),
        );
    }

    for pid in &ask_pids {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .arg("/F")
                .arg("/PID")
                .arg(pid)
                .status();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("kill").args(["-TERM", pid]).status();
        }
    }

    for _ in 0..50 {
        if TcpStream::connect("127.0.0.1:9223").is_err() {
            let _ = remove_chrome_pid_file();
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(100));
    }

    Err("Timed out waiting for existing ask-bridge Chrome to stop".to_string())
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

fn tool_text(val: &Value) -> Result<String, String> {
    val.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("text"))
        .and_then(|t| t.as_str())
        .map(|text| text.to_string())
        .ok_or_else(|| format!("Could not extract text field from tool result: {:?}", val))
}

fn take_snapshot_text(config_path: &str) -> Result<String, String> {
    let res = call_mcp_tool(config_path, "take_snapshot", serde_json::json!({}))?;
    tool_text(&res)
}

fn extract_snapshot_uid(line: &str) -> Option<String> {
    let marker_pos = line.find("uid=")?;
    let mut rest = line[marker_pos + 4..].trim_start();
    rest = rest.trim_start_matches(['"', '\'', '[']);
    let uid: String = rest
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'' && *c != ']')
        .collect();
    if uid.is_empty() { None } else { Some(uid) }
}

fn find_snapshot_uid(snapshot: &str, include: &[&str], exclude: &[&str]) -> Option<String> {
    snapshot.lines().find_map(|line| {
        let lower = line.to_lowercase();
        let includes_all = include
            .iter()
            .all(|needle| lower.contains(&needle.to_lowercase()));
        let excludes_all = exclude
            .iter()
            .all(|needle| !lower.contains(&needle.to_lowercase()));
        if includes_all && excludes_all {
            extract_snapshot_uid(line)
        } else {
            None
        }
    })
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
            let stdin_opt = child.stdin.take();
            if let Some(mut stdin) = stdin_opt {
                let _ = stdin.write_all(markdown.as_bytes()).map_err(|e| {
                    eprintln!("Failed to send Markdown content to glow: {}", e);
                });
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

fn validate_provider_feature_support(provider: Provider, cli: &Cli) -> Result<(), String> {
    if provider == Provider::Gemini && !cli.images.is_empty() {
        return Err(
            "Gemini image attachments are not supported yet. Use --file for Gemini document attachments."
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_dir(name: &str) -> std::path::PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "ask_bridge_{}_{}_{}",
            name,
            std::process::id(),
            timestamp
        ))
    }

    #[test]
    fn parses_provider_as_global_argument() {
        let cli = Cli::try_parse_from(["ask-bridge", "--provider", "gemini", "login"]).unwrap();
        assert_eq!(cli.provider, Some(Provider::Gemini));
        assert!(matches!(cli.command, Some(Commands::Login)));

        let cli = Cli::try_parse_from(["ask-bridge", "login", "--provider", "gemini"]).unwrap();
        assert_eq!(cli.provider, Some(Provider::Gemini));
        assert!(matches!(cli.command, Some(Commands::Login)));
    }

    #[test]
    fn parses_config_command() {
        let cli = Cli::try_parse_from(["ask-bridge", "config", "--provider", "gemini"]).unwrap();
        assert_eq!(cli.provider, Some(Provider::Gemini));
        assert!(matches!(cli.command, Some(Commands::Config)));
    }

    #[test]
    fn parses_config_command_without_provider() {
        let cli = Cli::try_parse_from(["ask-bridge", "config"]).unwrap();
        assert_eq!(cli.provider, None);
        assert!(matches!(cli.command, Some(Commands::Config)));
    }

    #[test]
    fn leaves_provider_unset_when_cli_argument_is_missing() {
        let cli = Cli::try_parse_from(["ask-bridge", "hello"]).unwrap();
        assert_eq!(cli.provider, None);
    }

    #[test]
    fn parses_provider_from_config_json() {
        assert_eq!(
            parse_configured_provider(r#"{"provider":"gemini"}"#).unwrap(),
            Some(Provider::Gemini)
        );
        assert_eq!(
            parse_configured_provider(r#"{"provider":"chatgpt"}"#).unwrap(),
            Some(Provider::ChatGpt)
        );
        assert_eq!(
            parse_configured_provider(r#"{"provider":"chat-gpt"}"#).unwrap(),
            Some(Provider::ChatGpt)
        );
        assert_eq!(parse_configured_provider(r#"{}"#).unwrap(), None);
    }

    #[test]
    fn resolves_provider_precedence() {
        assert_eq!(
            effective_provider(Some(Provider::ChatGpt), Some(Provider::Gemini)),
            Provider::ChatGpt
        );
        assert_eq!(
            effective_provider(None, Some(Provider::Gemini)),
            Provider::Gemini
        );
        assert_eq!(effective_provider(None, None), Provider::ChatGpt);
    }

    #[test]
    fn cli_provider_bypasses_invalid_config() {
        let provider = resolve_provider_with(Some(Provider::ChatGpt), || {
            Err("config should not be loaded".to_string())
        })
        .unwrap();

        assert_eq!(provider, Provider::ChatGpt);
    }

    #[test]
    fn resolves_provider_from_config_when_cli_provider_is_missing() {
        let provider = resolve_provider_with(None, || Ok(Some(Provider::Gemini))).unwrap();
        assert_eq!(provider, Provider::Gemini);
    }

    #[test]
    fn rejects_invalid_provider_in_config_json() {
        let err = parse_configured_provider(r#"{"provider":"claude"}"#).unwrap_err();
        assert!(err.contains("Invalid provider"));
    }

    #[test]
    fn parses_close_command() {
        let cli = Cli::try_parse_from(["ask-bridge", "close"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Close)));
    }

    #[test]
    fn hides_debug_commands_from_help() {
        let mut command = Cli::command();
        let help = command.render_long_help().to_string();

        assert!(!help.contains("\n  open"));
        assert!(!help.contains("\n  get"));
        assert!(!help.contains("\n  dump"));
        assert!(!help.contains("\n  screenshot"));
        assert!(help.contains("\n  login"));
        assert!(help.contains("\n  close"));
    }

    #[test]
    fn still_parses_hidden_debug_commands() {
        let cli = Cli::try_parse_from(["ask-bridge", "open"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Open { .. })));

        let cli = Cli::try_parse_from(["ask-bridge", "get"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Get { .. })));

        let cli = Cli::try_parse_from(["ask-bridge", "dump"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Dump)));

        let cli = Cli::try_parse_from(["ask-bridge", "screenshot"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Screenshot)));
    }

    #[test]
    fn rejects_unknown_provider() {
        assert!(Cli::try_parse_from(["ask-bridge", "--provider", "claude", "hello"]).is_err());
    }

    #[test]
    fn maps_provider_urls() {
        assert_eq!(
            Provider::from_url("https://chatgpt.com/c/abc"),
            Some(Provider::ChatGpt)
        );
        assert_eq!(
            Provider::from_url("https://gemini.google.com/app/abc"),
            Some(Provider::Gemini)
        );
        assert_eq!(Provider::from_url("https://example.com"), None);
    }

    #[test]
    fn parses_chatgpt_agent_prompt_with_chinese_agent_name() {
        assert_eq!(
            parse_chatgpt_agent_prompt(
                "@智慧 研究多奇數位創意有限公司的發展沿革與創辦人的豐功偉業"
            ),
            Some(ChatGptAgentPrompt {
                agent_mention: "@智慧",
                body: "研究多奇數位創意有限公司的發展沿革與創辦人的豐功偉業"
            })
        );
    }

    #[test]
    fn parses_chatgpt_agent_prompt_with_ten_character_agent_name() {
        assert_eq!(
            parse_chatgpt_agent_prompt("@一二三四五六七八九十 查資料"),
            Some(ChatGptAgentPrompt {
                agent_mention: "@一二三四五六七八九十",
                body: "查資料"
            })
        );
    }

    #[test]
    fn trims_extra_whitespace_between_chatgpt_agent_and_body() {
        assert_eq!(
            parse_chatgpt_agent_prompt("@智慧 \n\t查資料").unwrap().body,
            "查資料"
        );
    }

    #[test]
    fn rejects_invalid_chatgpt_agent_prompt_shapes() {
        assert_eq!(parse_chatgpt_agent_prompt("智慧 查資料"), None);
        assert_eq!(parse_chatgpt_agent_prompt("@ 查資料"), None);
        assert_eq!(parse_chatgpt_agent_prompt("@智慧"), None);
        assert_eq!(parse_chatgpt_agent_prompt("@智慧   "), None);
        assert_eq!(
            parse_chatgpt_agent_prompt("@一二三四五六七八九十甲 查資料"),
            None
        );
    }

    #[test]
    fn extracts_snapshot_uid_from_common_formats() {
        assert_eq!(
            extract_snapshot_uid(r#"- button "上傳檔案" [uid="1_23"]"#),
            Some("1_23".to_string())
        );
        assert_eq!(
            extract_snapshot_uid(r#"- button "Upload file" uid=42"#),
            Some("42".to_string())
        );
    }

    #[test]
    fn finds_snapshot_uid_with_include_and_exclude_terms() {
        let snapshot = r#"
            - button "加入雲端硬碟檔案" [uid="1_10"]
            - menuitem "上傳檔案. 文件、資料、程式碼檔案" [uid="1_11"]
        "#;
        assert_eq!(
            find_snapshot_uid(snapshot, &["上傳檔案"], &["雲端"]),
            Some("1_11".to_string())
        );
    }

    #[test]
    fn rejects_gemini_image_attachments() {
        let cli = Cli::try_parse_from([
            "ask-bridge",
            "--provider",
            "gemini",
            "--image",
            "token.png",
            "read",
        ])
        .unwrap();
        assert!(validate_provider_feature_support(Provider::Gemini, &cli).is_err());
    }

    #[test]
    fn allows_gemini_file_attachments() {
        let cli = Cli::try_parse_from([
            "ask-bridge",
            "--provider",
            "gemini",
            "--file",
            "token.txt",
            "read",
        ])
        .unwrap();
        assert!(validate_provider_feature_support(Provider::Gemini, &cli).is_ok());
    }

    #[test]
    fn finds_linux_google_chrome_command_from_path() {
        let root = make_test_dir("chrome_path");
        let first_dir = root.join("first");
        let second_dir = root.join("second");
        std::fs::create_dir_all(&first_dir).unwrap();
        std::fs::create_dir_all(&second_dir).unwrap();

        let stable_path = first_dir.join("google-chrome-stable");
        let chrome_path = second_dir.join("google-chrome");
        std::fs::write(&stable_path, "").unwrap();
        std::fs::write(&chrome_path, "").unwrap();

        let path_env = std::env::join_paths([first_dir.as_os_str(), second_dir.as_os_str()])
            .expect("test PATH should be joinable");

        let found = find_linux_chrome_path(Some(path_env.as_os_str()), &[]);

        assert_eq!(found, Some(chrome_path.to_string_lossy().to_string()));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn finds_linux_chrome_from_standard_candidates_when_path_misses() {
        let root = make_test_dir("chrome_candidate");
        std::fs::create_dir_all(&root).unwrap();
        let chrome_path = root.join("google-chrome");
        std::fs::write(&chrome_path, "").unwrap();

        let chrome_path_str = chrome_path.to_string_lossy().to_string();
        let candidates = [chrome_path_str.as_str()];

        let found = find_linux_chrome_path(None, &candidates);

        assert_eq!(found, Some(chrome_path_str));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn returns_none_when_linux_chrome_is_missing() {
        assert_eq!(find_linux_chrome_path(None, &[]), None);
    }

    #[test]
    fn matches_profile_argument_with_quotes_and_slashes() {
        let command = r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9223 "--user-data-dir=C:\Users\Will\.config\ask-bridge\chrome-profile""#;
        let profile_path = r"C:/Users/Will/.config/ask-bridge/chrome-profile";

        assert!(command_uses_profile(command, profile_path));
    }

    #[test]
    fn matches_profile_argument_when_value_is_separated_by_space() {
        let command = r#"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome --remote-debugging-port=9223 --user-data-dir /Users/will/.config/ask-bridge/chrome-profile"#;
        let profile_path = "/Users/will/.config/ask-bridge/chrome-profile";

        assert!(command_uses_profile(command, profile_path));
    }

    #[test]
    fn rejects_different_profile_argument() {
        let command = r#"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome --remote-debugging-port=9223 --user-data-dir=/Users/will/.config/other/chrome-profile"#;
        let profile_path = "/Users/will/.config/ask-bridge/chrome-profile";

        assert!(!command_uses_profile(command, profile_path));
    }

    #[test]
    fn composer_without_account_or_auth_controls_has_unknown_login_state() {
        let signals = LoginSignals {
            account: false,
            auth_control: false,
            auth_path: false,
            composer: true,
        };

        assert_eq!(signals.state(), LoginState::Unknown);
    }

    #[test]
    fn account_control_has_logged_in_state() {
        let signals = LoginSignals {
            account: true,
            auth_control: false,
            auth_path: false,
            composer: true,
        };

        assert_eq!(signals.state(), LoginState::LoggedIn);
    }

    #[test]
    fn auth_control_or_auth_path_has_logged_out_state() {
        let visible_auth_control = LoginSignals {
            account: false,
            auth_control: true,
            auth_path: false,
            composer: true,
        };
        let auth_path = LoginSignals {
            account: false,
            auth_control: false,
            auth_path: true,
            composer: false,
        };

        assert_eq!(visible_auth_control.state(), LoginState::LoggedOut);
        assert_eq!(auth_path.state(), LoginState::LoggedOut);
    }

    #[test]
    fn empty_login_signals_have_unknown_state() {
        let signals = LoginSignals {
            account: false,
            auth_control: false,
            auth_path: false,
            composer: false,
        };

        assert_eq!(signals.state(), LoginState::Unknown);
    }

    #[test]
    fn prefers_logged_in_provider_page_over_selected_page() {
        let pages = [
            PageLoginState {
                id: 2,
                selected: true,
                login_state: LoginState::LoggedOut,
            },
            PageLoginState {
                id: 7,
                selected: false,
                login_state: LoginState::LoggedIn,
            },
        ];

        assert_eq!(preferred_provider_page_id(&pages), Some(7));
    }

    #[test]
    fn falls_back_to_selected_provider_page_when_none_are_logged_in() {
        let pages = [
            PageLoginState {
                id: 2,
                selected: false,
                login_state: LoginState::Unknown,
            },
            PageLoginState {
                id: 7,
                selected: true,
                login_state: LoginState::LoggedOut,
            },
        ];

        assert_eq!(preferred_provider_page_id(&pages), Some(7));
    }

    #[test]
    fn marker_identifies_ask_bridge_chrome_without_profile_argument() {
        let command = r#"chrome.exe --type=browser --ask-bridge-instance"#;

        assert!(command_identifies_ask_chrome(
            command,
            r"C:\Users\Will\.config\ask-bridge\chrome-profile"
        ));
    }

    #[test]
    fn finds_ask_chrome_owner_in_parent_process_chain() {
        let commands = std::collections::HashMap::from([
            ("100", "chrome.exe --type=utility"),
            (
                "50",
                "chrome.exe --remote-debugging-port=9223 --ask-bridge-instance",
            ),
        ]);
        let parents = std::collections::HashMap::from([("100", "50"), ("50", "1")]);

        let owner = find_ask_chrome_owner_pid_with(
            "100",
            "/tmp/ask-bridge/chrome-profile",
            |pid| commands.get(pid).map(|command| (*command).to_string()),
            |pid| parents.get(pid).map(|parent| (*parent).to_string()),
        );

        assert_eq!(owner, Some("50".to_string()));
    }

    #[test]
    fn rejects_process_chain_without_profile_or_marker() {
        let commands = std::collections::HashMap::from([
            ("100", "chrome.exe --type=utility"),
            ("50", "chrome.exe --remote-debugging-port=9223"),
        ]);
        let parents = std::collections::HashMap::from([("100", "50"), ("50", "1")]);

        let owner = find_ask_chrome_owner_pid_with(
            "100",
            "/tmp/ask-bridge/chrome-profile",
            |pid| commands.get(pid).map(|command| (*command).to_string()),
            |pid| parents.get(pid).map(|parent| (*parent).to_string()),
        );

        assert_eq!(owner, None);
    }
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

fn click_latest_copy_button(config_path: &str, provider: Provider) -> Result<(), String> {
    let response_selector = serde_json::to_string(provider.latest_response_selector())
        .map_err(|e| format!("Failed to serialize response selector: {}", e))?;
    let script = r#"() => {
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
                    if (el.closest('model-response, response-container, [data-message-author-role="assistant"], .agent-turn')) return 50;
                    return 10;
                };
                const messages = Array.from(document.querySelectorAll(__RESPONSE_SELECTOR__));
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
    .replace("__RESPONSE_SELECTOR__", &response_selector);
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": script
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

fn wait_for_page_load(config_path: &str, provider: Provider, verbose: bool) -> Result<(), String> {
    if verbose {
        println!("Waiting for page readyState...");
    }

    // Phase 1: Wait for readyState complete or interactive
    let mut ready = false;
    for _ in 0..90 {
        let ready_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": "() => document.readyState === 'complete' || document.readyState === 'interactive'"
            }),
        );

        if ready_res
            .and_then(|res| parse_script_result(&res))
            .map(|parsed| parsed.as_bool().unwrap_or(false))
            .unwrap_or(false)
        {
            ready = true;
            break;
        }

        thread::sleep(Duration::from_millis(500));
    }

    if !ready {
        return Err("Timeout waiting for page readyState to be loaded".to_string());
    }

    if verbose {
        println!("Waiting for {} page elements...", provider.display_name());
    }

    // Phase 2: Wait for key provider elements to render.
    for _ in 0..60 {
        let element_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": provider.ready_check_js()
            }),
        );

        if element_res
            .and_then(|res| parse_script_result(&res))
            .map(|parsed| parsed.as_bool().unwrap_or(false))
            .unwrap_or(false)
        {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }

    if verbose {
        println!(
            "Warning: Timeout waiting for {} page elements. Proceeding anyway...",
            provider.display_name()
        );
    }
    Ok(())
}

fn open_url_tab(
    config_path: &str,
    provider: Provider,
    url: &str,
    headless: bool,
    verbose: bool,
) -> Result<(), String> {
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

            let page_provider = Provider::from_url(url).unwrap_or(provider);
            return wait_for_page_load(config_path, page_provider, verbose);
        }

        thread::sleep(Duration::from_millis(250));
    }

    let page_provider = Provider::from_url(url).unwrap_or(provider);
    wait_for_page_load(config_path, page_provider, verbose)
}

fn copy_latest_markdown(config_path: &str, provider: Provider) -> Result<String, String> {
    match copy_latest_markdown_via_clipboard(config_path, provider) {
        Ok(content) => Ok(content),
        Err(_) => scrape_latest_markdown_from_dom(config_path, provider),
    }
}

fn copy_latest_markdown_via_clipboard(
    config_path: &str,
    provider: Provider,
) -> Result<String, String> {
    let clipboard_before = read_clipboard().unwrap_or_default();
    let sentinel = format!("__ASK_CHATGPT_COPY_PENDING_{}__", std::process::id());
    write_clipboard(&sentinel)?;

    // Click the copy button, retrying if the message or button is not found yet (due to asynchronous rendering of Single Page App)
    let mut click_err = None;
    for _ in 0..30 {
        match click_latest_copy_button(config_path, provider) {
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

    let content = copied_content
        .ok_or_else(|| "Timed out waiting for clipboard content after clicking copy".to_string())?;

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

fn scrape_latest_markdown_from_dom(
    config_path: &str,
    provider: Provider,
) -> Result<String, String> {
    let latest_selector = serde_json::to_string(provider.latest_response_selector())
        .map_err(|e| format!("Failed to serialize response selector: {}", e))?;
    let content_selector = serde_json::to_string(provider.response_content_selector())
        .map_err(|e| format!("Failed to serialize response content selector: {}", e))?;
    let inspect_js = r#"() => {
        const latestSelector = __LATEST_SELECTOR__;
        const contentSelector = __CONTENT_SELECTOR__;
        const messages = Array.from(document.querySelectorAll(latestSelector))
            .filter((el) => ((el.innerText || el.textContent || '').trim().length > 0));
        const latest = messages[messages.length - 1];
        if (!latest) return 'No assistant message found';
        const turn = contentSelector ? (latest.querySelector(contentSelector) || latest) : latest;
        
        const elementToMarkdown = (element) => {
            let markdown = '';
            const processedSrcs = new Set();
            const walk = (node) => {
                if (node.nodeType === Node.TEXT_NODE) {
                    markdown += node.textContent;
                    return;
                }
                if (node.nodeType !== Node.ELEMENT_NODE) return;

                const tag = node.tagName.toLowerCase();
                
                const classText = Array.from(node.classList || []).join(' ');
                if (node.classList.contains('sr-only') ||
                    /screen-reader|visually-hidden|cdk-visually-hidden/.test(classText) ||
                    tag === 'button' || tag === 'style' || tag === 'script') {
                    return;
                }

                // Code blocks
                if (tag === 'pre') {
                    const codeEl = node.querySelector('code');
                    const langClass = codeEl ? Array.from(codeEl.classList).find(c => c.startsWith('language-')) : '';
                    const lang = langClass ? langClass.replace('language-', '') : '';
                    const codeText = codeEl ? codeEl.textContent : node.textContent;
                    markdown += '\n```' + lang + '\n' + codeText + '\n```\n';
                    return;
                }

                // Inline code
                if (tag === 'code') {
                    if (!node.closest('pre')) {
                        markdown += '`' + node.textContent + '`';
                        return;
                    }
                }

                // Bold
                if (tag === 'strong' || tag === 'b') {
                    markdown += '**';
                    for (const child of node.childNodes) walk(child);
                    markdown += '**';
                    return;
                }

                // Italics
                if (tag === 'em' || tag === 'i') {
                    markdown += '*';
                    for (const child of node.childNodes) walk(child);
                    markdown += '*';
                    return;
                }

                // Links
                if (tag === 'a') {
                    const href = node.getAttribute('href') || '';
                    const text = node.textContent || '';
                    if (href && text) {
                        markdown += '[' + text + '](' + href + ')';
                        return;
                    }
                }

                // Paragraphs, headers, list items
                if (tag === 'p') markdown += '\n';
                if (tag === 'br') markdown += '\n';
                if (tag === 'h1') markdown += '\n# ';
                if (tag === 'h2') markdown += '\n## ';
                if (tag === 'h3') markdown += '\n### ';
                if (tag === 'h4') markdown += '\n#### ';
                if (tag === 'h5') markdown += '\n##### ';
                if (tag === 'h6') markdown += '\n###### ';
                if (tag === 'li') markdown += '\n* ';

                // Images
                if (tag === 'img') {
                    const src = node.getAttribute('src') || '';
                    const alt = node.getAttribute('alt') || 'image';
                    if (src && !src.includes('avatar') && !src.includes('profile')) {
                        if (processedSrcs.has(src)) return;
                        processedSrcs.add(src);
                        markdown += '\n![' + alt + '](' + src + ')\n';
                        return;
                    }
                }

                for (const child of node.childNodes) {
                    walk(child);
                }

                if (['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'li'].includes(tag)) {
                    markdown += '\n';
                }
            };

            walk(element);
            return markdown.trim().replace(/\n{3,}/g, '\n\n');
        };
        
        return elementToMarkdown(turn);
    }"#
    .replace("__LATEST_SELECTOR__", &latest_selector)
    .replace("__CONTENT_SELECTOR__", &content_selector);

    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": inspect_js
        }),
    )?;

    let val = parse_script_result(&res)?;
    let content = val
        .as_str()
        .ok_or_else(|| "DOM scraper returned non-string result".to_string())?
        .to_string();

    if content == "No assistant message found" {
        return Err(format!(
            "No assistant message found on {} page",
            provider.display_name()
        ));
    }

    Ok(content)
}

fn download_images_from_latest_message(
    config_path: &str,
    provider: Provider,
    image_output: Option<&str>,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        println!("Checking for generated images in the latest assistant response...");
    }
    let latest_selector = serde_json::to_string(provider.latest_response_selector())
        .map_err(|e| format!("Failed to serialize response selector: {}", e))?;
    let image_scan_js = r#"() => {
                window.__downloaded_images_status = "pending";
                window.__downloaded_images = null;
                (async () => {
                    try {
                        const messages = document.querySelectorAll(__LATEST_SELECTOR__);
                        const latestMessage = messages[messages.length - 1];
                        if (!latestMessage) {
                            window.__downloaded_images = [];
                            window.__downloaded_images_status = "success";
                            return;
                        }
                        
                        const imgs = Array.from(latestMessage.querySelectorAll('img'));
                        const seenSrcs = new Set();
                        const candidateImgs = imgs.filter(img => {
                            const src = img.src || '';
                            if (src.includes('avatar') || src.includes('profile')) return false;
                            const width = img.naturalWidth || img.width || 0;
                            const height = img.naturalHeight || img.height || 0;
                            if (width > 0 && width < 100) return false;
                            if (height > 0 && height < 100) return false;
                            if (!src.startsWith('http') && !src.startsWith('blob:') && !src.startsWith('data:image/')) return false;
                            if (seenSrcs.has(src)) return false;
                            seenSrcs.add(src);
                            return true;
                        });

                        const imagesData = [];
                        for (let i = 0; i < candidateImgs.length; i++) {
                            const img = candidateImgs[i];
                            try {
                                if (!img.complete) {
                                    await new Promise((resolve) => {
                                        img.addEventListener('load', resolve);
                                        img.addEventListener('error', resolve);
                                        setTimeout(resolve, 10000);
                                    });
                                }

                                let dataUrl = "";
                                if ((img.src || '').startsWith('data:image/')) {
                                    dataUrl = img.src;
                                } else {
                                    try {
                                        const response = await fetch(img.src);
                                        const blob = await response.blob();
                                        dataUrl = await new Promise((resolve, reject) => {
                                            const reader = new FileReader();
                                            reader.onloadend = () => resolve(reader.result);
                                            reader.onerror = reject;
                                            reader.readAsDataURL(blob);
                                        });
                                    } catch (fetchErr) {
                                        const canvas = document.createElement('canvas');
                                        canvas.width = img.naturalWidth || img.width || 512;
                                        canvas.height = img.naturalHeight || img.height || 512;
                                        const ctx = canvas.getContext('2d');
                                        ctx.drawImage(img, 0, 0);
                                        dataUrl = canvas.toDataURL('image/png');
                                    }
                                }

                                if (dataUrl && dataUrl.startsWith('data:image/')) {
                                    imagesData.push({
                                        index: i,
                                        src: img.src,
                                        alt: img.alt || "",
                                        dataUrl: dataUrl
                                    });
                                }
                            } catch (err) {
                                // ignore
                            }
                        }
                        window.__downloaded_images = imagesData;
                        window.__downloaded_images_status = "success";
                    } catch (e) {
                        window.__downloaded_images_status = "error: " + e.message;
                    }
                })();
                return { ok: true };
            }"#
    .replace("__LATEST_SELECTOR__", &latest_selector);

    let start_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": image_scan_js
        }),
    )?;

    let start_parsed = parse_script_result(&start_res)?;
    if !start_parsed["ok"].as_bool().unwrap_or(false) {
        return Err("Failed to initiate image scanning script".to_string());
    }

    let mut wait_cycles = 0;
    let mut status = String::from("pending");
    while status == "pending" && wait_cycles < 150 {
        thread::sleep(Duration::from_millis(100));
        let check_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": "() => window.__downloaded_images_status || 'pending'"
            }),
        )?;
        if let Some(s) = parse_script_result(&check_res)
            .ok()
            .and_then(|p| p.as_str().map(|str_ref| str_ref.to_string()))
        {
            status = s;
        }
        wait_cycles += 1;
    }

    if status.starts_with("error:") {
        return Err(format!("Image scanning failed: {}", status));
    }

    if status == "pending" {
        return Err("Timed out waiting for images to download in browser".to_string());
    }

    let get_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": r#"() => {
                const res = window.__downloaded_images || [];
                delete window.__downloaded_images;
                delete window.__downloaded_images_status;
                return res;
            }"#
        }),
    )?;

    let parsed = parse_script_result(&get_res)?;
    let images = match parsed.as_array() {
        Some(arr) => arr,
        None => return Ok(()),
    };

    if images.is_empty() {
        if verbose {
            println!("No generated images found in the latest response.");
        }
        return Ok(());
    }

    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let total = images.len();
    for (idx, img) in images.iter().enumerate() {
        let data_url = match img["dataUrl"].as_str() {
            Some(s) => s,
            None => continue,
        };

        let parts: Vec<&str> = data_url.splitn(2, ',').collect();
        if parts.len() != 2 {
            continue;
        }

        let header = parts[0];
        let base64_data = parts[1];

        let ext = if header.contains("image/png") {
            "png"
        } else if header.contains("image/jpeg") || header.contains("image/jpg") {
            "jpg"
        } else if header.contains("image/webp") {
            "webp"
        } else {
            "png"
        };

        let decoded = general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64 data: {}", e))?;

        let file_path = match image_output {
            Some(output_str) => {
                let path = std::path::Path::new(output_str);
                let is_dir = path.is_dir()
                    || output_str.ends_with('/')
                    || output_str.ends_with('\\')
                    || path.extension().is_none();

                if is_dir {
                    std::fs::create_dir_all(path)
                        .map_err(|e| format!("Failed to create directory {:?}: {}", path, e))?;
                    path.join(format!("generated_{}_{}.{}", epoch, idx, ext))
                } else {
                    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            format!("Failed to create parent directory {:?}: {}", parent, e)
                        })?;
                    }
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| "Invalid file name".to_string())?;
                    let file_ext = path.extension().and_then(|e| e.to_str()).unwrap_or(ext);

                    if total <= 1 {
                        parent.join(format!("{}.{}", file_stem, file_ext))
                    } else {
                        parent.join(format!("{}_{}.{}", file_stem, idx + 1, file_ext))
                    }
                }
            }
            None => {
                std::fs::create_dir_all("target")
                    .map_err(|e| format!("Failed to create target/ directory: {}", e))?;
                std::path::PathBuf::from(format!("target/generated_{}_{}.{}", epoch, idx, ext))
            }
        };

        std::fs::write(&file_path, decoded)
            .map_err(|e| format!("Failed to write image file {:?}: {}", file_path, e))?;

        println!(
            "Downloaded and saved generated image to: {}",
            file_path.to_string_lossy()
        );
    }

    Ok(())
}

/// Display an image in the terminal using kitty's icat protocol.
/// Silently skips if kitty icat is not available.
fn display_image_in_terminal(image_path: &str) {
    let _ = Command::new("kitty").args(["icat", image_path]).status();
}

fn wait_for_attachment_indicator(
    config_path: &str,
    provider: Provider,
    path: &str,
    verbose: bool,
) -> Result<(), String> {
    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let file_stem = Path::new(path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or(file_name);
    let file_name_json = serde_json::to_string(file_name)
        .map_err(|e| format!("Failed to serialize file name: {}", e))?;
    let file_stem_json = serde_json::to_string(file_stem)
        .map_err(|e| format!("Failed to serialize file stem: {}", e))?;
    let js = r#"() => {
        const fileName = __FILE_NAME__;
        const fileStem = __FILE_STEM__;
        const text = document.body.innerText || '';
        return text.includes(fileName) || text.includes(fileStem);
    }"#
    .replace("__FILE_NAME__", &file_name_json)
    .replace("__FILE_STEM__", &file_stem_json);

    for _ in 0..30 {
        let check_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({ "function": js }),
        )?;
        if parse_script_result(&check_res)
            .ok()
            .and_then(|p| p.as_bool())
            .unwrap_or(false)
        {
            if verbose {
                println!(
                    "{} accepted attachment '{}'",
                    provider.display_name(),
                    file_name
                );
            }
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err(format!(
        "Timed out waiting for {} to show attachment '{}'",
        provider.display_name(),
        file_name
    ))
}

fn upload_attachments_via_file_chooser(
    config_path: &str,
    provider: Provider,
    image_paths: &[String],
    file_paths: &[String],
    verbose: bool,
) -> Result<(), String> {
    for (path, verify_filename) in image_paths
        .iter()
        .map(|path| (path, false))
        .chain(file_paths.iter().map(|path| (path, true)))
    {
        let canonical_path = std::fs::canonicalize(path)
            .map_err(|e| format!("Failed to resolve file '{}': {}", path, e))?;
        let file_path = canonical_path.to_string_lossy().to_string();

        let snapshot = take_snapshot_text(config_path)?;
        let menu_uid = match provider {
            Provider::Gemini => {
                find_snapshot_uid(&snapshot, &["上傳與工具"], &["更多", "雲端", "drive"])
                    .or_else(|| find_snapshot_uid(&snapshot, &["upload"], &["drive"]))
            }
            Provider::ChatGpt => find_snapshot_uid(&snapshot, &["attach"], &["settings", "menu"]),
        }
        .ok_or_else(|| {
            format!(
                "Could not find {} upload menu in page snapshot",
                provider.display_name()
            )
        })?;

        call_mcp_tool(
            config_path,
            "click",
            serde_json::json!({
                "uid": menu_uid,
                "includeSnapshot": false
            }),
        )?;
        thread::sleep(Duration::from_millis(500));

        let snapshot = take_snapshot_text(config_path)?;
        let upload_uid = match provider {
            Provider::Gemini => find_snapshot_uid(&snapshot, &["上傳檔案"], &["雲端", "drive"])
                .or_else(|| find_snapshot_uid(&snapshot, &["upload", "file"], &["drive"])),
            Provider::ChatGpt => find_snapshot_uid(&snapshot, &["file"], &["drive", "connect"]),
        }
        .unwrap_or_else(|| menu_uid.clone());

        if verbose {
            println!(
                "Uploading attachment '{}' to {}...",
                file_path,
                provider.display_name()
            );
        }
        call_mcp_tool(
            config_path,
            "upload_file",
            serde_json::json!({
                "uid": upload_uid,
                "filePath": file_path,
                "includeSnapshot": false
            }),
        )?;
        if verify_filename {
            wait_for_attachment_indicator(config_path, provider, path, verbose)?;
        } else {
            thread::sleep(Duration::from_millis(800));
        }
    }

    Ok(())
}

/// Map a file extension to a MIME type. Covers common image and document formats.
/// `ext` is expected to already be lowercased by the caller.
fn mime_type_for_extension(ext: &str) -> &'static str {
    match ext {
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "rtf" => "application/rtf",
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "xml" => "application/xml",
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "ts" => "text/typescript",
        "tsx" => "text/typescript",
        "js" | "mjs" | "cjs" => "text/javascript",
        "jsx" => "text/javascript",
        "css" => "text/css",
        "py" => "text/x-python",
        "rb" => "text/x-ruby",
        "go" => "text/x-go",
        "rs" => "text/x-rust",
        "java" => "text/x-java",
        "kt" => "text/x-kotlin",
        "c" => "text/x-c",
        "h" => "text/x-c",
        "cpp" | "cc" | "cxx" => "text/x-c++",
        "hpp" => "text/x-c++",
        "cs" => "text/x-csharp",
        "swift" => "text/x-swift",
        "php" => "text/x-php",
        "sh" => "application/x-sh",
        "bash" => "application/x-sh",
        "zsh" => "application/x-sh",
        "sql" => "application/sql",
        "toml" => "application/toml",
        "ini" => "text/plain",
        "log" => "text/plain",
        // Archives
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "bz2" => "application/x-bzip2",
        "7z" => "application/x-7z-compressed",
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        // Video
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

/// Upload local image and/or document files to the provider prompt composer using the
/// best available provider-specific upload mechanism.
/// Returns an error string if any attachment fails to upload.
fn upload_attachments_to_provider(
    config_path: &str,
    provider: Provider,
    image_paths: &[String],
    file_paths: &[String],
    verbose: bool,
) -> Result<(), String> {
    let total = image_paths.len() + file_paths.len();
    if total == 0 {
        return Ok(());
    }

    let data_transfer_image_paths: &[String] = if provider == Provider::Gemini
        && !image_paths.is_empty()
    {
        match upload_attachments_via_file_chooser(config_path, provider, image_paths, &[], verbose)
        {
            Ok(()) => &[],
            Err(e) => {
                if verbose {
                    eprintln!(
                        "Warning: {} image file chooser upload failed, trying DataTransfer fallback: {}",
                        provider.display_name(),
                        e
                    );
                }
                image_paths
            }
        }
    } else {
        image_paths
    };

    let data_transfer_total = data_transfer_image_paths.len() + file_paths.len();
    if data_transfer_total == 0 {
        return Ok(());
    }

    if verbose {
        println!(
            "Attaching {} attachment(s) ({} image(s), {} file(s)) to the prompt...",
            data_transfer_total,
            data_transfer_image_paths.len(),
            file_paths.len()
        );
    }

    // Build a JSON array of { name, mime, base64 } objects. Images first, then other files.
    // We pass raw base64 + mime and decode in JS to avoid `fetch(data:...)` which ChatGPT's
    // Content-Security-Policy blocks (results in "Failed to fetch").
    let mut files_json = Vec::new();
    for path in data_transfer_image_paths.iter().chain(file_paths.iter()) {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mime = mime_type_for_extension(&ext);
        let b64 = general_purpose::STANDARD.encode(&bytes);
        let file_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_string();
        files_json.push(serde_json::json!({
            "name": file_name,
            "mime": mime,
            "base64": b64
        }));
    }

    let files_json_str = serde_json::to_string(&files_json)
        .map_err(|e| format!("Failed to serialize attachment data: {}", e))?;
    let composer_selectors = provider.composer_selectors_json();
    // Build JS without raw strings to avoid r#"..."# termination conflicts
    let js = "() => {\n".to_string()
        + "    window.__upload_images_status = 'pending';\n"
        + "    (async () => {\n"
        + "        try {\n"
        + &format!("            const filesData = {};\n", files_json_str)
        + "            const decodeB64 = (b64) => {\n"
        + "                const bin = atob(b64);\n"
        + "                const len = bin.length;\n"
        + "                const bytes = new Uint8Array(len);\n"
        + "                for (let i = 0; i < len; i++) bytes[i] = bin.charCodeAt(i);\n"
        + "                return bytes;\n"
        + "            };\n"
        + "            const fileObjects = filesData.map((f) => {\n"
        + "                const bytes = decodeB64(f.base64);\n"
        + "                const blob = new Blob([bytes], { type: f.mime || 'application/octet-stream' });\n"
        + "                return new File([blob], f.name, { type: blob.type });\n"
        + "            });\n"
        + &format!(
            "            const composerSelectors = {};\n",
            composer_selectors
        )
        + "            const el = composerSelectors.map((s) => document.querySelector(s)).find(Boolean);\n"
        + "            if (!el) {\n"
        + "                window.__upload_images_status = 'error: composer not found';\n"
        + "                return;\n"
        + "            }\n"
        + "            el.focus();\n"
        + "            const fileInputs = Array.from(document.querySelectorAll('input[type=\"file\"]'));\n"
        + "            // Pick the file input whose `accept` attribute covers every attached file.\n"
        + "            // An input accepts a file when accept is empty, contains `*/*` or a matching\n"
        + "            // wildcard (e.g. `image/*`), or lists the file's exact MIME type.\n"
        + "            const accepts = (input, file) => {\n"
        + "                const acc = (input.getAttribute('accept') || '').trim();\n"
        + "                if (!acc) return true;\n"
        + "                const parts = acc.split(',').map(s => s.trim().toLowerCase()).filter(Boolean);\n"
        + "                const mime = (file.type || '').toLowerCase();\n"
        + "                const top = mime.split('/')[0];\n"
        + "                return parts.some(p => p === '*/*' || p === mime || (p.endsWith('/*') && top && p === top + '/*'));\n"
        + "            };\n"
        + "            const fileInput = fileInputs.find(i => fileObjects.every(f => accepts(i, f)))\n"
        + "                || fileInputs.find(i => !i.getAttribute('accept'))\n"
        + "                || fileInputs[0];\n"
        + "            if (fileInput) {\n"
        + "                const dt = new DataTransfer();\n"
        + "                for (const f of fileObjects) dt.items.add(f);\n"
        + "                fileInput.files = dt.files;\n"
        + "                fileInput.dispatchEvent(new Event('change', { bubbles: true }));\n"
        + "                window.__upload_images_status = 'success:file-input';\n"
        + "                return;\n"
        + "            }\n"
        + "            const dt = new DataTransfer();\n"
        + "            for (const f of fileObjects) dt.items.add(f);\n"
        + "            const targets = [el, el.closest('form'), document.querySelector('main'), document.body].filter(Boolean);\n"
        + "            for (const target of targets) {\n"
        + "                for (const type of ['dragenter', 'dragover', 'drop']) {\n"
        + "                    target.dispatchEvent(new DragEvent(type, {\n"
        + "                        bubbles: true, cancelable: true, dataTransfer: dt\n"
        + "                    }));\n"
        + "                }\n"
        + "            }\n"
        + "            const pasteEvent = new ClipboardEvent('paste', {\n"
        + "                bubbles: true, cancelable: true, clipboardData: dt\n"
        + "            });\n"
        + "            el.dispatchEvent(pasteEvent);\n"
        + "            window.__upload_images_status = 'success:drop';\n"
        + "        } catch (e) {\n"
        + "            window.__upload_images_status = 'error: ' + e.message;\n"
        + "        }\n"
        + "    })();\n"
        + "    return true;\n"
        + "}";

    let start_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({ "function": js }),
    )?;

    let start_parsed = parse_script_result(&start_res)?;
    if !start_parsed.as_bool().unwrap_or(false) {
        return Err("Failed to initiate attachment upload script".to_string());
    }

    // Poll for completion. Allow up to ~60s for large document uploads.
    let mut wait_cycles = 0;
    let mut status = String::from("pending");
    while status == "pending" && wait_cycles < 300 {
        thread::sleep(Duration::from_millis(200));
        let check_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({ "function": "() => window.__upload_images_status || 'pending'" }),
        )?;
        if let Some(s) = parse_script_result(&check_res)
            .ok()
            .and_then(|p| p.as_str().map(|r| r.to_string()))
        {
            status = s;
        }
        wait_cycles += 1;
    }

    if status.starts_with("error:") {
        return Err(format!("Attachment upload failed: {}", status));
    }
    if status == "pending" {
        return Err("Timed out waiting for attachments to upload".to_string());
    }

    if verbose {
        println!("Attachments attached successfully ({})", status);
    }

    // Give the UI a moment to render the attachments before typing the prompt
    thread::sleep(Duration::from_millis(800));

    if provider == Provider::Gemini {
        // Gemini renders image attachments as thumbnails without a stable filename in
        // the accessible text. Text/document chips do expose their filename, so keep
        // the stricter post-upload check for `--file` attachments only.
        for path in file_paths {
            if let Err(e) = wait_for_attachment_indicator(config_path, provider, path, verbose) {
                if verbose {
                    eprintln!(
                        "Warning: {} DataTransfer upload was not detected, trying file chooser fallback: {}",
                        provider.display_name(),
                        e
                    );
                }
                return upload_attachments_via_file_chooser(
                    config_path,
                    provider,
                    image_paths,
                    file_paths,
                    verbose,
                );
            }
        }
    }

    Ok(())
}

/// Switch the selected provider to the specified model. The page must already be
/// loaded and logged in. `model` is matched case- and punctuation-insensitively.
fn switch_model(
    config_path: &str,
    provider: Provider,
    model: &str,
    verbose: bool,
) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("Empty model name".to_string());
    }
    let target_json = serde_json::to_string(model.trim())
        .map_err(|e| format!("Failed to serialize model name: {}", e))?;

    if verbose {
        println!(
            "Switching {} model to '{}'...",
            provider.display_name(),
            model.trim()
        );
    }

    let js = match provider {
        Provider::ChatGpt => {
            // The script opens the composer pill menu, walks visible leaves and submenu
            // triggers, and clicks the first leaf whose normalized label matches.
            "() => {\n".to_string()
                + "    window.__switch_model_status = 'pending';\n"
                + "    (async () => {\n"
                + "    try {\n"
                + "        const sleep = (ms) => new Promise((r) => setTimeout(r, ms));\n"
                + "        const norm = (s) => (s || '').toLowerCase().replace(/[\\s.\\-_]/g, '');\n"
                + &format!("        const target = norm({});\n", target_json)
                + "        if (!target) { window.__switch_model_status = 'error: empty target'; return; }\n"
                + "        const visited = new Set();\n"
                + "        const closeMenus = async () => {\n"
                + "            document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', keyCode: 27, bubbles: true }));\n"
                + "            await sleep(400);\n"
                + "        };\n"
                + "        await closeMenus();\n"
                + "        let pill = null;\n"
                + "        for (let i = 0; i < 20; i++) {\n"
                + "            pill = document.querySelector('button.__composer-pill');\n"
                + "            if (pill) break;\n"
                + "            await sleep(250);\n"
                + "        }\n"
                + "        if (!pill) { window.__switch_model_status = 'error: composer pill not found'; return; }\n"
                + "        pill.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }));\n"
                + "        pill.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));\n"
                + "        pill.click();\n"
                + "        await sleep(800);\n"
                + "        let clicked = false;\n"
                + "        let chosen = '';\n"
                + "        for (let depth = 0; depth < 6 && !clicked; depth++) {\n"
                + "            const all = Array.from(document.querySelectorAll('[role=\"menuitem\"], [role=\"menuitemradio\"]'));\n"
                + "            const leaves = all.filter((it) => it.getAttribute('aria-haspopup') !== 'menu');\n"
                + "            for (const it of leaves) {\n"
                + "                const t = norm(it.innerText);\n"
                + "                if (t && t === target) {\n"
                + "                    it.click();\n"
                + "                    clicked = true;\n"
                + "                    chosen = it.innerText;\n"
                + "                    break;\n"
                + "                }\n"
                + "            }\n"
                + "            if (clicked) break;\n"
                + "            const trigs = all.filter((it) => it.getAttribute('aria-haspopup') === 'menu');\n"
                + "            const trig = trigs.find((it) => {\n"
                + "                const k = norm(it.innerText) + '|' + (it.getAttribute('aria-label') || '');\n"
                + "                return !visited.has(k);\n"
                + "            });\n"
                + "            if (!trig) break;\n"
                + "            visited.add(norm(trig.innerText) + '|' + (trig.getAttribute('aria-label') || ''));\n"
                + "            trig.dispatchEvent(new MouseEvent('pointerenter', { bubbles: true }));\n"
                + "            trig.dispatchEvent(new MouseEvent('pointermove', { bubbles: true }));\n"
                + "            trig.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));\n"
                + "            trig.click();\n"
                + "            await sleep(750);\n"
                + "        }\n"
                + "        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', keyCode: 27, bubbles: true }));\n"
                + "        if (!clicked) {\n"
                + "            window.__switch_model_status = 'error: model not found in menu';\n"
                + "            return;\n"
                + "        }\n"
                + "        window.__switch_model_status = 'success:' + chosen;\n"
                + "    } catch (e) {\n"
                + "        window.__switch_model_status = 'error: ' + e.message;\n"
                + "    }\n"
                + "    })();\n"
                + "    return true;\n"
                + "}"
        }
        Provider::Gemini => {
            let template = r#"() => {
                window.__switch_model_status = 'pending';
                (async () => {
                    try {
                        const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
                        const norm = (s) => (s || '').toLowerCase().replace(/[^\p{Letter}\p{Number}]+/gu, '');
                        const canonical = (s) => {
                            const n = norm(s).replace(/^已選取/, '');
                            if (n.includes('flashlite') || n.includes('31flashlite')) return 'flashlite';
                            if (n.includes('35flash') || (n.endsWith('flash') && !n.includes('lite'))) return 'flash';
                            if (n.includes('31pro') || n === 'pro') return 'pro';
                            return n;
                        };
                        const target = canonical(__TARGET_MODEL__);
                        if (!target) { window.__switch_model_status = 'error: empty target'; return; }
                        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', keyCode: 27, bubbles: true }));
                        await sleep(250);
                        const buttons = Array.from(document.querySelectorAll('button'));
                        const modeButton = buttons.find((button) => /模式挑選器|model picker|mode picker/i.test([
                            button.getAttribute('aria-label'),
                            button.textContent
                        ].filter(Boolean).join(' ')));
                        if (!modeButton) { window.__switch_model_status = 'error: Gemini mode picker not found'; return; }
                        modeButton.click();
                        await sleep(800);
                        const items = Array.from(document.querySelectorAll('[role="menuitem"], [role="menuitemradio"]'));
                        let chosen = null;
                        for (const item of items) {
                            const label = item.innerText || item.textContent || item.getAttribute('aria-label') || '';
                            if (canonical(label) === target || norm(label) === norm(__TARGET_MODEL__)) {
                                chosen = item;
                                break;
                            }
                        }
                        if (!chosen) {
                            window.__switch_model_status = 'error: model not found in menu';
                            return;
                        }
                        chosen.click();
                        await sleep(500);
                        window.__switch_model_status = 'success:' + (chosen.innerText || chosen.textContent || '').trim();
                    } catch (e) {
                        window.__switch_model_status = 'error: ' + e.message;
                    }
                })();
                return true;
            }"#;
            template.replace("__TARGET_MODEL__", &target_json)
        }
    };

    let start_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({ "function": js }),
    )?;
    let start_parsed = parse_script_result(&start_res)?;
    if !start_parsed.as_bool().unwrap_or(false) {
        return Err("Failed to initiate model switch script".to_string());
    }

    let mut wait_cycles = 0;
    let mut status = String::from("pending");
    while status == "pending" && wait_cycles < 60 {
        thread::sleep(Duration::from_millis(200));
        let check_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({ "function": "() => window.__switch_model_status || 'pending'" }),
        )?;
        if let Some(s) = parse_script_result(&check_res)
            .ok()
            .and_then(|p| p.as_str().map(|r| r.to_string()))
        {
            status = s;
        }
        wait_cycles += 1;
    }

    if status.starts_with("error:") {
        return Err(format!("Model switch failed: {}", status));
    }
    if status == "pending" {
        return Err("Timed out waiting for model switch".to_string());
    }

    if verbose {
        println!("Model switched successfully ({})", status);
    }

    // Give the UI a moment to settle after switching models
    thread::sleep(Duration::from_millis(500));

    Ok(())
}

fn wait_for_submit_status(config_path: &str) -> Result<String, String> {
    let mut wait_cycles = 0;
    let mut status = String::from("pending");

    // Page-side submission scripts may wait up to 15s for ChatGPT/Gemini to
    // enable the send button, so keep this host-side polling window longer.
    while status == "pending" && wait_cycles < 180 {
        thread::sleep(Duration::from_millis(100));
        let check_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": "() => window.__submit_status || 'pending'"
            }),
        )?;
        if let Some(s) = parse_script_result(&check_res)
            .ok()
            .and_then(|p| p.as_str().map(|str_ref| str_ref.to_string()))
        {
            status = s;
        }
        wait_cycles += 1;
    }

    if status.starts_with("error:") {
        return Err(status);
    }

    if status == "pending" {
        return Err("Timed out waiting for send button to activate and submit".to_string());
    }

    Ok(status)
}

fn focus_and_clear_composer(config_path: &str, provider: Provider) -> Result<(), String> {
    let js = r#"() => {
            const composerSelectors = __COMPOSER_SELECTORS__;
            const el = composerSelectors.map((s) => document.querySelector(s)).find(Boolean);
            if (!el) {
                return { ok: false, error: 'composer not found' };
            }

            el.focus();
            try {
                const range = document.createRange();
                range.selectNodeContents(el);
                const sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(range);
                document.execCommand('delete');
            } catch (e) {}

            const currentText = typeof el.value !== 'undefined' ? el.value : (el.innerText || el.textContent || '');
            if ((currentText || '').trim().length > 0) {
                if (typeof el.value !== 'undefined') {
                    el.value = '';
                    if (el._valueTracker) {
                        el._valueTracker.setValue('');
                    }
                } else {
                    el.innerHTML = '<p><br></p>';
                }
                el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'deleteContentBackward' }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
            }

            el.focus();
            return { ok: true };
        }"#
    .replace("__COMPOSER_SELECTORS__", provider.composer_selectors_json());

    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({ "function": js }),
    )?;
    let parsed = parse_script_result(&res)?;
    if parsed
        .get("ok")
        .and_then(|ok| ok.as_bool())
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(parsed
            .get("error")
            .and_then(|err| err.as_str())
            .unwrap_or("failed to focus and clear composer")
            .to_string())
    }
}

fn wait_for_chatgpt_agent_menu(config_path: &str) -> Result<(), String> {
    let js = r#"() => {
            const isVisible = (el) => {
                if (!el) return false;
                const style = window.getComputedStyle(el);
                if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                const rect = el.getBoundingClientRect();
                return rect.width > 0 && rect.height > 0;
            };
            const composer = document.querySelector('#prompt-textarea');
            const composerRect = composer ? composer.getBoundingClientRect() : null;
            const isNearComposer = (el) => {
                if (!composerRect) return true;
                const rect = el.getBoundingClientRect();
                const itemCenterX = (rect.left + rect.right) / 2;
                const composerCenterX = (composerRect.left + composerRect.right) / 2;
                const maxHorizontalDistance = Math.max(500, composerRect.width);
                return Math.abs(itemCenterX - composerCenterX) <= maxHorizontalDistance &&
                    Math.abs(rect.top - composerRect.bottom) <= 500;
            };
            const items = Array.from(document.querySelectorAll(
                '.popover .__menu-item, [class*="popover"] .__menu-item, [role="menuitem"], [role="option"], [cmdk-item]'
            ))
                .filter((el) => isVisible(el) && isNearComposer(el))
                .map((el) => (el.innerText || el.textContent || '').trim())
                .filter(Boolean);

            return { ok: items.length > 0, items: items.slice(0, 5) };
        }"#;

    let mut last_state = String::new();
    for _ in 0..40 {
        thread::sleep(Duration::from_millis(125));
        let res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({ "function": js }),
        )?;
        let parsed = parse_script_result(&res)?;
        if parsed
            .get("ok")
            .and_then(|ok| ok.as_bool())
            .unwrap_or(false)
        {
            return Ok(());
        }
        last_state = parsed.to_string();
    }

    Err(format!(
        "Timed out waiting for ChatGPT agent menu after typing mention ({})",
        last_state
    ))
}

fn wait_for_chatgpt_agent_selection(config_path: &str) -> Result<(), String> {
    let js = r#"() => {
            const composer = document.querySelector('#prompt-textarea');
            if (!composer) {
                return { ok: false, error: 'composer not found' };
            }
            const agentPill = composer.querySelector(
                '[data-id="agent"], [data-system-hint-type="agent"], [data-symbol="ecosystemMention"], [data-inline-selection-pill][contenteditable="false"]'
            );
            return {
                ok: Boolean(agentPill),
                text: (composer.innerText || composer.textContent || '').trim(),
                keyword: agentPill ? (agentPill.getAttribute('data-keyword') || agentPill.textContent || '').trim() : ''
            };
        }"#;

    let mut last_state = String::new();
    for _ in 0..40 {
        thread::sleep(Duration::from_millis(125));
        let res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({ "function": js }),
        )?;
        let parsed = parse_script_result(&res)?;
        if parsed
            .get("ok")
            .and_then(|ok| ok.as_bool())
            .unwrap_or(false)
        {
            return Ok(());
        }
        last_state = parsed.to_string();
    }

    Err(format!(
        "Timed out waiting for ChatGPT agent selection after Tab ({})",
        last_state
    ))
}

fn submit_regular_prompt(
    config_path: &str,
    provider: Provider,
    prompt: &str,
) -> Result<String, String> {
    let prompt_json = serde_json::to_string(prompt)
        .map_err(|e| format!("Failed to serialize prompt text: {}", e))?;
    let set_and_submit_js = r#"() => {
            window.__submit_status = 'pending';
            (async () => {
                try {
                    const composerSelectors = __COMPOSER_SELECTORS__;
                    const sendSelectors = __SEND_SELECTORS__;
                    const el = composerSelectors.map((s) => document.querySelector(s)).find(Boolean);
                    if (!el) {
                        window.__submit_status = 'error: composer not found';
                        return;
                    }
                    el.focus();
                    
                    const value = __PROMPT__;
                    el.focus();
                    
                    try {
                        const range = document.createRange();
                        range.selectNodeContents(el);
                        const sel = window.getSelection();
                        sel.removeAllRanges();
                        sel.addRange(range);
                    } catch (e) {}
                    
                    let pasted = false;
                    try {
                        const dataTransfer = new DataTransfer();
                        dataTransfer.setData('text/plain', value);
                        const event = new ClipboardEvent('paste', {
                            bubbles: true,
                            cancelable: true
                        });
                        Object.defineProperty(event, 'clipboardData', {
                            value: dataTransfer,
                            writable: false,
                            configurable: true
                        });
                        el.dispatchEvent(event);
                        
                        const currentText = typeof el.value !== 'undefined' ? el.value : el.textContent;
                        if (currentText && currentText.trim().length > 0) {
                            pasted = true;
                        }
                    } catch (e) {}
                    
                    if (!pasted) {
                        const success = document.execCommand('insertText', false, value);
                        if (!success) {
                            if (typeof el.value !== 'undefined') {
                                el.value = value;
                                if (el._valueTracker) {
                                    el._valueTracker.setValue('');
                                }
                            } else {
                                el.innerText = value;
                            }
                            el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: value }));
                            el.dispatchEvent(new Event('change', { bubbles: true }));
                        }
                    }
                    
                    const isVisible = (el) => {
                        if (!el || el.disabled || el.getAttribute('aria-disabled') === 'true') return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    };
                    const findAndClickSendButton = () => {
                        let btn = null;
                        for (const s of sendSelectors) {
                            btn = document.querySelector(s);
                            if (isVisible(btn)) break;
                        }
                        
                        if (btn && !btn.disabled && btn.getAttribute('aria-disabled') !== 'true') {
                            btn.click();
                            return { ok: true, clicked: true, buttonLabel: btn.getAttribute('aria-label') };
                        }
                        return null;
                    };
                    
                    let result = findAndClickSendButton();
                    if (result) {
                        window.__submit_status = 'success:' + JSON.stringify(result);
                        return;
                    }

                    for (let i = 0; i < 150; i++) {
                        await new Promise(r => setTimeout(r, 100));
                        result = findAndClickSendButton();
                        if (result) {
                            window.__submit_status = 'success:' + JSON.stringify(result);
                            return;
                        }
                    }
                    
                    window.__submit_status = 'error: Send button did not become active/enabled';
                } catch (e) {
                    window.__submit_status = 'error: ' + e.message;
                }
            })();
            return true;
        }"#
    .replace("__COMPOSER_SELECTORS__", provider.composer_selectors_json())
    .replace("__SEND_SELECTORS__", provider.send_button_selectors_json())
    .replace("__PROMPT__", &prompt_json);

    let start_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": set_and_submit_js
        }),
    )?;

    let start_parsed = parse_script_result(&start_res)?;
    if !start_parsed.as_bool().unwrap_or(false) {
        return Err("Failed to initiate text entry and submission script".to_string());
    }

    wait_for_submit_status(config_path)
}

fn submit_chatgpt_agent_prompt(
    config_path: &str,
    parts: &ChatGptAgentPrompt<'_>,
    verbose: bool,
) -> Result<String, String> {
    if verbose {
        println!(
            "Selecting ChatGPT agent '{}' before submitting prompt...",
            parts.agent_mention
        );
    }

    focus_and_clear_composer(config_path, Provider::ChatGpt)?;
    call_mcp_tool(
        config_path,
        "type_text",
        serde_json::json!({
            "text": parts.agent_mention
        }),
    )?;
    wait_for_chatgpt_agent_menu(config_path)?;
    call_mcp_tool(
        config_path,
        "press_key",
        serde_json::json!({
            "key": "Tab",
            "includeSnapshot": false
        }),
    )?;
    wait_for_chatgpt_agent_selection(config_path)?;

    let body_json = serde_json::to_string(parts.body)
        .map_err(|e| format!("Failed to serialize prompt body: {}", e))?;
    let paste_and_submit_js = r#"() => {
            window.__submit_status = 'pending';
            (async () => {
                try {
                    const sendSelectors = __SEND_SELECTORS__;
                    const el = document.querySelector('#prompt-textarea');
                    if (!el) {
                        window.__submit_status = 'error: composer not found';
                        return;
                    }
                    const agentPill = el.querySelector(
                        '[data-id="agent"], [data-system-hint-type="agent"], [data-symbol="ecosystemMention"], [data-inline-selection-pill][contenteditable="false"]'
                    );
                    if (!agentPill) {
                        window.__submit_status = 'error: ChatGPT agent was not selected into the composer';
                        return;
                    }

                    const body = __BODY__;
                    const currentText = el.textContent || '';
                    const value = currentText && !/\s$/.test(currentText) ? ' ' + body : body;
                    el.focus();

                    try {
                        const range = document.createRange();
                        range.selectNodeContents(el);
                        range.collapse(false);
                        const sel = window.getSelection();
                        sel.removeAllRanges();
                        sel.addRange(range);
                    } catch (e) {}

                    let pasted = false;
                    try {
                        const dataTransfer = new DataTransfer();
                        dataTransfer.setData('text/plain', value);
                        const event = new ClipboardEvent('paste', {
                            bubbles: true,
                            cancelable: true
                        });
                        Object.defineProperty(event, 'clipboardData', {
                            value: dataTransfer,
                            writable: false,
                            configurable: true
                        });
                        el.dispatchEvent(event);
                        const afterPasteText = el.innerText || el.textContent || '';
                        pasted = afterPasteText.includes(body);
                    } catch (e) {}

                    if (!pasted) {
                        const success = document.execCommand('insertText', false, value);
                        if (!success) {
                            el.appendChild(document.createTextNode(value));
                            el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: value }));
                            el.dispatchEvent(new Event('change', { bubbles: true }));
                        }
                    }

                    const afterText = el.innerText || el.textContent || '';
                    if (!afterText.includes(body)) {
                        window.__submit_status = 'error: prompt body was not pasted after ChatGPT agent selection';
                        return;
                    }

                    const isVisible = (el) => {
                        if (!el || el.disabled || el.getAttribute('aria-disabled') === 'true') return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    };
                    const findAndClickSendButton = () => {
                        let btn = null;
                        for (const s of sendSelectors) {
                            btn = document.querySelector(s);
                            if (isVisible(btn)) break;
                        }
                        if (btn && !btn.disabled && btn.getAttribute('aria-disabled') !== 'true') {
                            btn.click();
                            return { ok: true, clicked: true, buttonLabel: btn.getAttribute('aria-label') };
                        }
                        return null;
                    };

                    let result = findAndClickSendButton();
                    if (result) {
                        window.__submit_status = 'success:' + JSON.stringify(result);
                        return;
                    }

                    for (let i = 0; i < 150; i++) {
                        await new Promise(r => setTimeout(r, 100));
                        result = findAndClickSendButton();
                        if (result) {
                            window.__submit_status = 'success:' + JSON.stringify(result);
                            return;
                        }
                    }

                    window.__submit_status = 'error: Send button did not become active/enabled';
                } catch (e) {
                    window.__submit_status = 'error: ' + e.message;
                }
            })();
            return true;
        }"#
    .replace(
        "__SEND_SELECTORS__",
        Provider::ChatGpt.send_button_selectors_json(),
    )
    .replace("__BODY__", &body_json);

    let start_res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": paste_and_submit_js
        }),
    )?;
    let start_parsed = parse_script_result(&start_res)?;
    if !start_parsed.as_bool().unwrap_or(false) {
        return Err("Failed to initiate ChatGPT agent prompt submission script".to_string());
    }

    wait_for_submit_status(config_path)
}

fn submit_prompt_to_provider(
    config_path: &str,
    provider: Provider,
    prompt: &str,
    verbose: bool,
) -> Result<String, String> {
    if provider == Provider::ChatGpt
        && let Some(parts) = parse_chatgpt_agent_prompt(prompt)
    {
        return submit_chatgpt_agent_prompt(config_path, &parts, verbose);
    }

    submit_regular_prompt(config_path, provider, prompt)
}

fn ensure_provider_tab(
    config_path: &str,
    provider: Provider,
    force_new: bool,
    headless: bool,
    verbose: bool,
) -> Result<(), String> {
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
        let old_provider_ids: Vec<usize> = pages
            .iter()
            .filter(|p| provider.owns_url(&p.url))
            .map(|p| p.id)
            .collect();

        if verbose {
            println!("Opening a brand new {} session...", provider.display_name());
        }
        call_mcp_tool(
            config_path,
            "new_page",
            serde_json::json!({
                "url": provider.home_url()
            }),
        )?;

        for id in old_provider_ids {
            if verbose {
                println!(
                    "Closing old {} tab (ID: {})...",
                    provider.display_name(),
                    id
                );
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

        if let Some(page) = refreshed_pages.iter().find(|p| provider.owns_url(&p.url)) {
            if verbose {
                println!(
                    "Selecting new {} tab (ID: {})...",
                    provider.display_name(),
                    page.id
                );
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
                    println!("Closing non-selected tab (ID: {})...", stale_page.id);
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
        let provider_pages: Vec<&Page> = pages
            .iter()
            .filter(|page| provider.owns_url(&page.url))
            .collect();

        let provider_page_id = if provider_pages.len() > 1 {
            let mut page_states = Vec::with_capacity(provider_pages.len());
            for page in &provider_pages {
                call_mcp_tool(
                    config_path,
                    "select_page",
                    serde_json::json!({
                        "pageId": page.id,
                        "bringToFront": false
                    }),
                )?;
                let login_state =
                    check_login_status(config_path, provider).unwrap_or(LoginState::Unknown);
                page_states.push(PageLoginState {
                    id: page.id,
                    selected: page.selected,
                    login_state,
                });
            }
            preferred_provider_page_id(&page_states)
        } else {
            provider_pages.first().map(|page| page.id)
        };

        match provider_page_id {
            Some(page_id) => {
                let page = provider_pages
                    .iter()
                    .find(|page| page.id == page_id)
                    .ok_or_else(|| "Selected provider page disappeared".to_string())?;
                if verbose {
                    println!(
                        "Found {} tab (ID: {}, selected: {}). Selecting/focusing...",
                        provider.display_name(),
                        page.id,
                        page.selected
                    );
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
            None => {
                // No provider tab. If there is only one blank tab, navigate it. Otherwise open a new page.
                if pages.len() == 1
                    && (pages[0].url == "about:blank"
                        || pages[0].url.contains("new-tab-page")
                        || pages[0].url.contains("chrome://welcome"))
                {
                    if verbose {
                        println!(
                            "Navigating existing blank tab to {}...",
                            provider.display_name()
                        );
                    }
                    call_mcp_tool(
                        config_path,
                        "navigate_page",
                        serde_json::json!({
                            "url": provider.home_url()
                        }),
                    )?;
                } else {
                    if verbose {
                        println!("Opening a new tab for {}...", provider.display_name());
                    }
                    call_mcp_tool(
                        config_path,
                        "new_page",
                        serde_json::json!({
                            "url": provider.home_url()
                        }),
                    )?;
                }
            }
        }
    }

    // Wait for the provider composer to be present.
    if verbose {
        println!("Waiting for {} to load...", provider.display_name());
    }
    for attempt in 0..90 {
        if attempt > 0 && attempt % 10 == 0 {
            let page_opt = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))
                .ok()
                .and_then(|res| {
                    res.get("content")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|obj| obj.get("text"))
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string())
                })
                .and_then(|text| {
                    parse_pages(&text)
                        .into_iter()
                        .find(|p| provider.owns_url(&p.url))
                });
            if let Some(page) = page_opt {
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

        let ready_res = call_mcp_tool(
            config_path,
            "evaluate_script",
            serde_json::json!({
                "function": provider.ready_check_js()
            }),
        );
        let ready_res = match ready_res {
            Ok(res) => res,
            Err(e) => {
                if verbose {
                    eprintln!(
                        "Warning: Failed to check {} readiness: {}",
                        provider.display_name(),
                        e
                    );
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

    Err(format!(
        "Timeout waiting for {} page to load",
        provider.display_name()
    ))
}

fn check_login_status(config_path: &str, provider: Provider) -> Result<LoginState, String> {
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": provider.login_signals_js()
        }),
    )?;

    let parsed = parse_script_result(&res)?;
    let signals: LoginSignals = serde_json::from_value(parsed)
        .map_err(|e| format!("Failed to parse login signals: {}", e))?;
    Ok(signals.state())
}

fn print_chrome_diagnostics(profile_path: &str) {
    let listener_pids = debug_port_listener_pids();
    let owner_pids = ask_chrome_pids_on_debug_port(profile_path);
    let recorded_pid = read_chrome_pid().unwrap_or_else(|| "unknown".to_string());

    println!("Chrome diagnostics:");
    println!("  profile: {}", profile_path);
    println!("  recorded PID: {}", recorded_pid);
    println!("  listener PIDs: {:?}", listener_pids);
    println!("  ask-bridge owner PIDs: {:?}", owner_pids);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if matches!(cli.command, Some(Commands::Config)) {
        if let Err(e) = run_config_command(cli.provider) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }

        return Ok(());
    }

    let provider = match resolve_provider(cli.provider) {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = validate_provider_feature_support(provider, &cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

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

    if matches!(cli.command, Some(Commands::Close)) {
        let profile_path = match chrome_profile_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error locating Chrome profile: {}", e);
                std::process::exit(1);
            }
        };

        match close_ask_chrome_on_debug_port(&profile_path) {
            Ok(true) => println!("Closed ask-bridge Chrome browser instance."),
            Ok(false) => println!("No ask-bridge Chrome browser instance is running."),
            Err(e) => {
                eprintln!("Error closing ask-bridge Chrome browser instance: {}", e);
                std::process::exit(1);
            }
        }

        return Ok(());
    }

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
                    let page_provider = Provider::from_url(&url).unwrap_or(provider);
                    if let Err(e) =
                        open_url_tab(&config_path, page_provider, &url, is_headless, cli.verbose)
                    {
                        eprintln!("Error opening URL: {}", e);
                        std::process::exit(1);
                    }

                    match copy_latest_markdown(&config_path, page_provider) {
                        Ok(markdown) => {
                            if let Some(ref output_path) = cli.output {
                                let _ = std::fs::write(output_path, &markdown).map_err(|e| {
                                    eprintln!("Error writing output file: {}", e);
                                    std::process::exit(1);
                                });
                            }
                            if let Err(e) = render_markdown(&markdown, use_glow) {
                                eprintln!("Error rendering Markdown: {}", e);
                                std::process::exit(1);
                            }
                            if let Err(e) = download_images_from_latest_message(
                                &config_path,
                                page_provider,
                                cli.image_output.as_deref(),
                                cli.verbose,
                            ) {
                                eprintln!("Error downloading images: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error copying latest response Markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    if let Err(e) =
                        ensure_provider_tab(&config_path, provider, false, is_headless, cli.verbose)
                    {
                        eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
                        std::process::exit(1);
                    }
                    println!("Successfully opened {}!", provider.display_name());
                }
                return Ok(());
            }
            Commands::Get { url } => {
                let mut page_provider = provider;
                if let Some(url) = url {
                    page_provider = Provider::from_url(&url).unwrap_or(provider);
                    if let Err(e) =
                        open_url_tab(&config_path, page_provider, &url, is_headless, cli.verbose)
                    {
                        eprintln!("Error opening URL: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    if let Err(e) =
                        ensure_provider_tab(&config_path, provider, false, is_headless, cli.verbose)
                    {
                        eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
                        std::process::exit(1);
                    }
                }

                match copy_latest_markdown(&config_path, page_provider) {
                    Ok(markdown) => {
                        if let Some(ref output_path) = cli.output {
                            let _ = std::fs::write(output_path, &markdown).map_err(|e| {
                                eprintln!("Error writing output file: {}", e);
                                std::process::exit(1);
                            });
                        }
                        if let Err(e) = render_markdown(&markdown, use_glow) {
                            eprintln!("Error rendering Markdown: {}", e);
                            std::process::exit(1);
                        }
                        if let Err(e) = download_images_from_latest_message(
                            &config_path,
                            page_provider,
                            cli.image_output.as_deref(),
                            cli.verbose,
                        ) {
                            eprintln!("Error downloading images: {}", e);
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
                if let Err(e) =
                    ensure_provider_tab(&config_path, provider, false, is_headless, cli.verbose)
                {
                    eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
                    std::process::exit(1);
                }
                println!("\n========================================================");
                println!("Please complete the login manually in the Chrome window.");
                println!("Once you have successfully logged in, press [Enter] here.");
                println!("========================================================\n");

                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;

                match check_login_status(&config_path, provider) {
                    Ok(LoginState::LoggedIn) => println!(
                        "Success: Logged in successfully! You can now use the `ask-bridge` command."
                    ),
                    Ok(LoginState::LoggedOut) => println!(
                        "Warning: We still detected a login button on the page. You might not be fully logged in. Please verify."
                    ),
                    Ok(LoginState::Unknown) => println!(
                        "Warning: The page loaded, but ask-bridge could not confirm the account menu. Login status is unknown."
                    ),
                    Err(e) => println!("Warning: Failed to verify login status: {}", e),
                }
                if cli.verbose {
                    match chrome_profile_path() {
                        Ok(profile_path) => print_chrome_diagnostics(&profile_path),
                        Err(e) => eprintln!("Warning: Failed to locate Chrome profile: {}", e),
                    }
                }
                return Ok(());
            }
            Commands::Close => unreachable!("close command is handled before Chrome startup"),
            Commands::Config => unreachable!("config command is handled before Chrome startup"),
            Commands::Dump => {
                let list_res = call_mcp_tool(&config_path, "list_pages", serde_json::json!({}))?;
                println!("All pages: {:?}", list_res);
                if let Err(e) =
                    ensure_provider_tab(&config_path, provider, false, is_headless, cli.verbose)
                {
                    eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
                    std::process::exit(1);
                }
                let url_res = call_mcp_tool(
                    &config_path,
                    "evaluate_script",
                    serde_json::json!({
                        "function": "() => window.location.href"
                    }),
                )?;
                println!("Current page URL: {:?}", parse_script_result(&url_res));
                let res = call_mcp_tool(
                    &config_path,
                    "evaluate_script",
                    serde_json::json!({
                        "function": "() => document.body.innerHTML"
                    }),
                )?;
                let html = parse_script_result(&res)?
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                std::fs::create_dir_all("target").unwrap();
                std::fs::write("target/dump.html", html)?;
                println!("Dumped HTML to target/dump.html");
                return Ok(());
            }
            Commands::Screenshot => {
                if let Err(e) =
                    ensure_provider_tab(&config_path, provider, false, is_headless, cli.verbose)
                {
                    eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
                    std::process::exit(1);
                }
                let res = call_mcp_tool(&config_path, "take_screenshot", serde_json::json!({}))?;

                let mut saved = false;
                if let Some(arr) = res.get("content").and_then(|c| c.as_array()) {
                    for item in arr {
                        if let Some(data) = item
                            .get("type")
                            .filter(|t| t.as_str() == Some("image"))
                            .and_then(|_| item.get("data"))
                            .and_then(|d| d.as_str())
                        {
                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                            match STANDARD.decode(data.trim()) {
                                Ok(bytes) => {
                                    std::fs::create_dir_all("target").unwrap();
                                    std::fs::write("target/screenshot.png", bytes)?;
                                    println!("Saved screenshot to target/screenshot.png");
                                    saved = true;
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode base64 image data: {}", e);
                                }
                            }
                        }
                    }
                }
                if !saved {
                    eprintln!(
                        "Could not find any image item in the tool response content. Full response: {:?}",
                        res
                    );
                }
                return Ok(());
            }
        }
    }

    // Read prompt from arguments and optionally append piped stdin content.
    let mut stdin_prompt = String::new();

    // Check if stdin is a pipe (not a tty)
    if !std::io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut stdin_prompt)?;
    }

    let prompt = match cli.prompt {
        Some(mut p) => {
            if !stdin_prompt.is_empty() {
                p.push_str("\n\n");
                p.push_str(&stdin_prompt);
            }
            p
        }
        None => stdin_prompt,
    };

    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        // No prompt and no command, print help
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        std::process::exit(0);
    }

    if let Err(e) = ensure_provider_tab(&config_path, provider, cli.new, is_headless, cli.verbose) {
        eprintln!("Error ensuring {} tab: {}", provider.display_name(), e);
        std::process::exit(1);
    }

    // Show attached images in the terminal before sending
    if !cli.images.is_empty() {
        for img_path in &cli.images {
            display_image_in_terminal(img_path);
        }
    }

    // Verify login
    match check_login_status(&config_path, provider) {
        Ok(LoginState::LoggedOut) => {
            eprintln!(
                "\nError: You are not logged in to {}.",
                provider.display_name()
            );
            eprintln!(
                "Please run `ask-bridge --provider {} login` to log in manually first, and then run your query again.\n",
                provider
            );
            std::process::exit(1);
        }
        Ok(LoginState::Unknown) => {
            eprintln!(
                "Warning: Could not confirm the {} account menu. Attempting to proceed...",
                provider.display_name()
            );
        }
        Ok(LoginState::LoggedIn) => {}
        Err(e) if cli.verbose => {
            eprintln!(
                "Warning: Failed to verify login status: {}. Attempting to proceed...",
                e
            );
        }
        Err(_) => {}
    }

    // Switch model if requested (before uploading attachments / typing the prompt)
    if let Some(m) = &cli.model
        && let Err(e) = switch_model(&config_path, provider, m, cli.verbose)
    {
        eprintln!("Error switching model: {}", e);
        std::process::exit(1);
    }

    // Upload any attached images/files before counting messages (so the UI is ready)
    if (!cli.images.is_empty() || !cli.files.is_empty())
        && let Err(e) = upload_attachments_to_provider(
            &config_path,
            provider,
            &cli.images,
            &cli.files,
            cli.verbose,
        )
    {
        eprintln!("Error attaching images/files: {}", e);
        std::process::exit(1);
    }

    // Get initial number of assistant messages before submitting the prompt
    let assistant_selector = serde_json::to_string(provider.assistant_selector())
        .map_err(|e| format!("Failed to serialize assistant selector: {}", e))?;
    let count_res = call_mcp_tool(
        &config_path,
        "evaluate_script",
        serde_json::json!({
            "function": format!("() => document.querySelectorAll({}).length", assistant_selector)
        }),
    )?;
    let initial_assistant_count = parse_script_result(&count_res)
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    if cli.verbose {
        println!("Setting prompt text and submitting...");
    }
    let status = submit_prompt_to_provider(&config_path, provider, &prompt, cli.verbose)
        .map_err(|e| format!("Text entry or submission failed: {}", e))?;

    if cli.verbose {
        println!("Prompt submitted successfully: {}", status);
    }

    if cli.verbose {
        println!("Waiting for {} response...", provider.display_name());
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
            print!(
                "\r\x1b[1;36m{}\x1b[0m 正在等待 {} 回應...",
                frame,
                provider.display_name()
            );
            io::stdout().flush()?;
            spinner_idx += 1;
        }

        if wait_cycles % 5 == 0 {
            let stop_selectors = provider.stop_button_selectors_json();
            let assistant_selector = serde_json::to_string(provider.assistant_selector())
                .map_err(|e| format!("Failed to serialize assistant selector: {}", e))?;
            let response_check_js = r#"() => {
                    const stopSelectors = __STOP_SELECTORS__;
                    const isVisible = (el) => {
                        if (!el || el.disabled || el.getAttribute('aria-disabled') === 'true') return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    };
                    const stopButton = stopSelectors.map((selector) => document.querySelector(selector)).find(isVisible);
                    const messages = document.querySelectorAll(__ASSISTANT_SELECTOR__);
                    const isNew = messages.length > __INITIAL_COUNT__;
                    
                    if (isVisible(stopButton)) {
                        return { status: "generating", isNew: isNew };
                    }
                    
                    if (isNew) {
                        return { status: "done", isNew: isNew };
                    }
                    
                    return { status: "waiting", isNew: isNew };
                }"#
            .replace("__STOP_SELECTORS__", stop_selectors)
            .replace("__ASSISTANT_SELECTOR__", &assistant_selector)
            .replace("__INITIAL_COUNT__", &initial_assistant_count.to_string());
            let check_res = match call_mcp_tool(
                &config_path,
                "evaluate_script",
                serde_json::json!({
                    "function": response_check_js
                }),
            ) {
                Ok(res) => res,
                Err(e) => {
                    if cli.verbose {
                        eprintln!(
                            "Warning: Failed to poll {} response: {}",
                            provider.display_name(),
                            e
                        );
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
            println!(
                "Copying final response from {} toolbar...",
                provider.display_name()
            );
        }
        match copy_latest_markdown(&config_path, provider) {
            Ok(content) => {
                last_markdown = content;
            }
            Err(e) => {
                eprintln!(
                    "Error copying response from {} toolbar: {}",
                    provider.display_name(),
                    e
                );
            }
        }
    }

    if let Err(e) = render_markdown(&last_markdown, use_glow) {
        eprintln!("Error rendering Markdown: {}", e);
    }

    if finished {
        let _ = download_images_from_latest_message(
            &config_path,
            provider,
            cli.image_output.as_deref(),
            cli.verbose,
        )
        .map_err(|e| {
            eprintln!("Error downloading images: {}", e);
        });
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
