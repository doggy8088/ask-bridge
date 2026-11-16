//! Batch runner for issue #24: run many prompt jobs from a JSONL file,
//! each job in its own browser tab, with per-job artifacts.
//!
//! Design constraints (no breaking changes):
//! - The single-prompt flow is untouched; batch is a separate code path
//!   behind the `batch` subcommand.
//! - Every browser operation for a job runs inside the batch control lock
//!   as `select_page(job page) -> operate`, so jobs can never act on each
//!   other's tabs even once jobs run concurrently.
//! - A job whose prompt submission hits a transport error is marked
//!   `Recoverable` and is never resent automatically (the prompt may
//!   already have been submitted).

use super::{
    ChatGptImageResponseState, LoginState, Provider, call_mcp_tool, check_login_status,
    copy_latest_markdown, download_images_from_latest_message, ensure_chatgpt_chat_mode,
    ensure_provider_tab_with_url, ensure_temporary_chat, mcp_error_is_transport, open_url_tab,
    parse_pages, parse_script_result, poll_response_once, resolve_selection_plan,
    resolve_session_target, submit_prompt_to_provider, switch_model, switch_reasoning,
    upload_attachments_to_provider,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const DEFAULT_JOB_TIMEOUT_SECS: u64 = 300;

/// Serializes multi-call browser operations (`select_page` + operation) for
/// batch jobs. Individual MCP calls are already serialized by the global
/// MCP session mutex; this lock adds atomicity across a call sequence.
static BATCH_CONTROL_LOCK: Mutex<()> = Mutex::new(());

pub fn with_control_lock<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = BATCH_CONTROL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f()
}

fn select_page(config_path: &str, page_id: usize, headless: bool) -> Result<(), String> {
    call_mcp_tool(
        config_path,
        "select_page",
        serde_json::json!({
            "pageId": page_id,
            "bringToFront": !headless
        }),
    )?;
    Ok(())
}

/// Run `f` with the job's page selected, holding the batch control lock for
/// the whole sequence.
pub fn with_target_page<T>(
    config_path: &str,
    page_id: usize,
    headless: bool,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    with_control_lock(|| {
        select_page(config_path, page_id, headless)?;
        f()
    })
}

// ---------------------------------------------------------------------------
// Job model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    pub id: String,
    pub provider: Option<Provider>,
    pub prompt: String,
    pub timeout_secs: u64,
    pub session: Option<String>,
    pub temporary: bool,
    pub model: Option<String>,
    pub reasoning: Option<String>,
    pub files: Vec<String>,
    pub images: Vec<String>,
    pub download_images: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawJob {
    id: Option<String>,
    provider: Option<String>,
    prompt: Option<String>,
    timeout: Option<u64>,
    new: Option<bool>,
    session: Option<String>,
    temporary: Option<bool>,
    model: Option<String>,
    reasoning: Option<String>,
    files: Option<Vec<String>>,
    images: Option<Vec<String>>,
    download_images: Option<bool>,
}

fn parse_provider_name(name: &str, line_no: usize) -> Result<Provider, String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "chatgpt" => Ok(Provider::ChatGpt),
        "gemini" => Ok(Provider::Gemini),
        "claude" => Ok(Provider::Claude),
        other => Err(format!(
            "line {}: unsupported provider \"{}\" (expected chatgpt, gemini or claude)",
            line_no, other
        )),
    }
}

/// Job IDs become directory names, so keep them to a conservative,
/// cross-platform-safe alphabet.
pub fn validate_job_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id != "."
        && id != ".."
        && id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid job id \"{}\": use 1-64 characters of [A-Za-z0-9._-], starting with a letter or digit",
            id
        ))
    }
}

pub fn parse_jobs_jsonl(input: &str) -> Result<Vec<JobSpec>, String> {
    let mut raws: Vec<(usize, RawJob)> = Vec::new();
    for (idx, line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let raw: RawJob = serde_json::from_str(trimmed)
            .map_err(|e| format!("line {}: invalid job JSON: {}", line_no, e))?;
        raws.push((line_no, raw));
    }
    if raws.is_empty() {
        return Err("no jobs found in input".to_string());
    }

    let mut used_ids: HashSet<String> = raws.iter().filter_map(|(_, raw)| raw.id.clone()).collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut jobs = Vec::with_capacity(raws.len());
    for (position, (line_no, raw)) in raws.into_iter().enumerate() {
        let id = match raw.id {
            Some(id) => {
                validate_job_id(&id).map_err(|e| format!("line {}: {}", line_no, e))?;
                if !seen.insert(id.clone()) {
                    return Err(format!("line {}: duplicate job id \"{}\"", line_no, id));
                }
                id
            }
            None => {
                let mut candidate = format!("job-{}", position + 1);
                let mut suffix = 2;
                while used_ids.contains(&candidate) {
                    candidate = format!("job-{}-{}", position + 1, suffix);
                    suffix += 1;
                }
                used_ids.insert(candidate.clone());
                seen.insert(candidate.clone());
                candidate
            }
        };

        let prompt = raw
            .prompt
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .ok_or_else(|| {
                format!(
                    "line {}: job \"{}\" is missing a non-empty \"prompt\"",
                    line_no, id
                )
            })?;

        let provider = raw
            .provider
            .as_deref()
            .map(|name| parse_provider_name(name, line_no))
            .transpose()?;

        if raw.new == Some(false) {
            return Err(format!(
                "line {}: job \"{}\": batch jobs always run in their own tab; \"new\": false is not supported",
                line_no, id
            ));
        }
        if raw.session.is_some() && raw.temporary == Some(true) {
            return Err(format!(
                "line {}: job \"{}\": \"temporary\" cannot be combined with \"session\"",
                line_no, id
            ));
        }
        let timeout_secs = raw.timeout.unwrap_or(DEFAULT_JOB_TIMEOUT_SECS);
        if timeout_secs == 0 {
            return Err(format!(
                "line {}: job \"{}\": \"timeout\" must be at least 1 second",
                line_no, id
            ));
        }
        if provider == Some(Provider::Gemini)
            && raw.images.as_ref().is_some_and(|images| !images.is_empty())
        {
            return Err(format!(
                "line {}: job \"{}\": Gemini image attachments are not supported yet; use \"files\" for Gemini",
                line_no, id
            ));
        }

        jobs.push(JobSpec {
            id,
            provider,
            prompt,
            timeout_secs,
            session: raw.session,
            temporary: raw.temporary.unwrap_or(false),
            model: raw.model,
            reasoning: raw.reasoning,
            files: raw.files.unwrap_or_default(),
            images: raw.images.unwrap_or_default(),
            download_images: raw.download_images.unwrap_or(true),
        });
    }
    Ok(jobs)
}

// ---------------------------------------------------------------------------
// Target registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBinding {
    pub page_id: usize,
    pub thread_url: Option<String>,
}

/// Maps job IDs to their bound browser page and (once known) thread URL.
/// Bindings are write-once per job: a job can never be rebound to another
/// job's page.
#[derive(Debug, Default)]
pub struct TargetRegistry {
    inner: Mutex<HashMap<String, TargetBinding>>,
}

impl TargetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&self, job_id: &str, page_id: usize) -> Result<(), String> {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        match map.get(job_id) {
            Some(existing) if existing.page_id != page_id => Err(format!(
                "job \"{}\" is already bound to page {}, refusing to rebind to page {}",
                job_id, existing.page_id, page_id
            )),
            Some(_) => Ok(()),
            None => {
                map.insert(
                    job_id.to_string(),
                    TargetBinding {
                        page_id,
                        thread_url: None,
                    },
                );
                Ok(())
            }
        }
    }

    pub fn set_thread_url(&self, job_id: &str, url: &str) {
        let mut map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(binding) = map.get_mut(job_id) {
            binding.thread_url = Some(url.to_string());
        }
    }

    pub fn get(&self, job_id: &str) -> Option<TargetBinding> {
        let map = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        map.get(job_id).cloned()
    }
}

// ---------------------------------------------------------------------------
// Outcomes and artifacts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobTerminal {
    Completed,
    Failed,
    TimedOut,
    Recoverable,
    Cancelled,
}

impl JobTerminal {
    pub fn as_str(self) -> &'static str {
        match self {
            JobTerminal::Completed => "completed",
            JobTerminal::Failed => "failed",
            JobTerminal::TimedOut => "timed_out",
            JobTerminal::Recoverable => "recoverable",
            JobTerminal::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobOutcome {
    pub id: String,
    pub terminal: JobTerminal,
    pub page_id: Option<usize>,
    pub thread_url: Option<String>,
    pub error: Option<String>,
    pub note: Option<String>,
    pub elapsed: Duration,
}

impl JobOutcome {
    fn failed(id: &str, error: String, started: Instant) -> Self {
        Self {
            id: id.to_string(),
            terminal: JobTerminal::Failed,
            page_id: None,
            thread_url: None,
            error: Some(error),
            note: None,
            elapsed: started.elapsed(),
        }
    }

    fn cancelled(id: &str) -> Self {
        Self {
            id: id.to_string(),
            terminal: JobTerminal::Cancelled,
            page_id: None,
            thread_url: None,
            error: Some("cancelled by --fail-fast after an earlier job failed".to_string()),
            note: None,
            elapsed: Duration::ZERO,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BatchSummary {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub timed_out: usize,
    pub recoverable: usize,
    pub cancelled: usize,
}

impl BatchSummary {
    pub fn record(&mut self, terminal: JobTerminal) {
        self.total += 1;
        match terminal {
            JobTerminal::Completed => self.completed += 1,
            JobTerminal::Failed => self.failed += 1,
            JobTerminal::TimedOut => self.timed_out += 1,
            JobTerminal::Recoverable => self.recoverable += 1,
            JobTerminal::Cancelled => self.cancelled += 1,
        }
    }

    pub fn exit_code(&self) -> i32 {
        if self.failed + self.timed_out + self.recoverable + self.cancelled == 0 {
            0
        } else {
            1
        }
    }

    pub fn render(&self) -> String {
        format!(
            "Total: {}\nCompleted: {}\nFailed: {}\nTimed out: {}\nRecoverable: {}\nCancelled: {}",
            self.total,
            self.completed,
            self.failed,
            self.timed_out,
            self.recoverable,
            self.cancelled
        )
    }
}

pub fn job_output_dir(base: &Path, job_id: &str) -> PathBuf {
    base.join(job_id)
}

/// Write a file via a temporary sibling plus rename, so readers never see a
/// partially written artifact.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "artifact".to_string());
    let tmp = path.with_file_name(format!(".{}.tmp-{}", file_name, std::process::id()));
    std::fs::write(&tmp, content)
        .map_err(|e| format!("Failed to write {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, path)
        .map_err(|e| format!("Failed to finalize {}: {}", path.display(), e))?;
    Ok(())
}

fn outcome_metadata(outcome: &JobOutcome, provider: Provider) -> String {
    serde_json::json!({
        "id": outcome.id,
        "provider": provider.to_string(),
        "pageId": outcome.page_id,
        "threadUrl": outcome.thread_url,
        "state": outcome.terminal.as_str(),
        "elapsedSeconds": outcome.elapsed.as_secs(),
        "error": outcome.error,
        "note": outcome.note,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Sequential execution core (generic over the executor for unit tests)
// ---------------------------------------------------------------------------

pub fn execute_batch_sequential(
    jobs: &[JobSpec],
    fail_fast: bool,
    mut executor: impl FnMut(&JobSpec) -> JobOutcome,
) -> (Vec<JobOutcome>, BatchSummary) {
    let mut outcomes = Vec::with_capacity(jobs.len());
    let mut summary = BatchSummary::default();
    let mut stopped = false;
    for job in jobs {
        let outcome = if stopped {
            JobOutcome::cancelled(&job.id)
        } else {
            let outcome = executor(job);
            if fail_fast && outcome.terminal != JobTerminal::Completed {
                stopped = true;
            }
            outcome
        };
        summary.record(outcome.terminal);
        outcomes.push(outcome);
    }
    (outcomes, summary)
}

// ---------------------------------------------------------------------------
// Real per-job execution against the browser
// ---------------------------------------------------------------------------

pub struct BatchContext {
    pub config_path: String,
    pub default_provider: Provider,
    pub headless: bool,
    pub verbose: bool,
    pub output_dir: PathBuf,
    pub registry: TargetRegistry,
}

fn selected_page_id(config_path: &str) -> Result<usize, String> {
    let list_res = call_mcp_tool(config_path, "list_pages", serde_json::json!({}))?;
    let text = list_res
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("Invalid list_pages response structure: {:?}", list_res))?;
    parse_pages(text)
        .into_iter()
        .find(|page| page.selected)
        .map(|page| page.id)
        .ok_or_else(|| "no selected page after opening the job tab".to_string())
}

fn current_page_url(config_path: &str) -> Option<String> {
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({ "function": "() => window.location.href" }),
    )
    .ok()?;
    parse_script_result(&res)
        .ok()
        .and_then(|u| u.as_str().map(|s| s.to_string()))
}

fn count_assistant_messages(config_path: &str, provider: Provider) -> Result<usize, String> {
    let assistant_selector = serde_json::to_string(provider.assistant_selector())
        .map_err(|e| format!("Failed to serialize assistant selector: {}", e))?;
    let res = call_mcp_tool(
        config_path,
        "evaluate_script",
        serde_json::json!({
            "function": format!(
                "() => document.querySelectorAll({}).length",
                assistant_selector
            )
        }),
    )?;
    Ok(parse_script_result(&res)
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize)
}

fn run_on_page<T>(
    ctx: &BatchContext,
    page_id: usize,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    with_target_page(&ctx.config_path, page_id, ctx.headless, f)
}

pub fn run_job(ctx: &BatchContext, job: &JobSpec) -> JobOutcome {
    let started = Instant::now();
    let config_path = &ctx.config_path;

    // Resolve the effective provider (a session URL may dictate it).
    let mut provider = job.provider.unwrap_or(ctx.default_provider);
    let mut session_url: Option<String> = None;
    let mut early_error: Option<String> = None;
    if let Some(session) = &job.session {
        match resolve_session_target(provider, job.provider.is_some(), session) {
            Ok((session_provider, url)) => {
                provider = session_provider;
                session_url = Some(url);
            }
            Err(e) => early_error = Some(e),
        }
    }
    if early_error.is_none() && provider == Provider::Gemini && !job.images.is_empty() {
        early_error = Some(
            "Gemini image attachments are not supported yet; use \"files\" for Gemini".to_string(),
        );
    }
    let selection_plan = if early_error.is_none() {
        match resolve_selection_plan(provider, job.model.as_deref(), job.reasoning.as_deref()) {
            Ok(plan) => {
                if plan.used_legacy_model {
                    eprintln!(
                        "[batch] job {}: warning: reasoning-like model values are deprecated; use \"reasoning\" instead",
                        job.id
                    );
                }
                Some(plan)
            }
            Err(e) => {
                early_error = Some(e);
                None
            }
        }
    } else {
        None
    };
    if let Some(error) = early_error {
        let mut outcome = JobOutcome::failed(&job.id, error, started);
        outcome.elapsed = started.elapsed();
        write_job_metadata(ctx, job, provider, &outcome);
        return outcome;
    }
    let selection_plan = selection_plan.expect("selection plan resolved");

    // Open the job's own tab and bind it in the registry. The whole
    // open -> identify -> bind sequence holds the control lock, so no other
    // job can interleave a tab creation or selection change.
    let page_id = match with_control_lock(|| {
        if let Some(url) = &session_url {
            open_url_tab(config_path, provider, url, ctx.headless, ctx.verbose)?;
        } else {
            ensure_provider_tab_with_url(
                config_path,
                provider,
                true,
                ctx.headless,
                ctx.verbose,
                if job.temporary {
                    provider.temporary_start_url()
                } else {
                    provider.home_url()
                },
            )?;
        }
        let page_id = selected_page_id(config_path)?;
        ctx.registry.bind(&job.id, page_id)?;
        Ok(page_id)
    }) {
        Ok(page_id) => page_id,
        Err(e) => {
            let outcome =
                JobOutcome::failed(&job.id, format!("failed to open tab: {}", e), started);
            write_job_metadata(ctx, job, provider, &outcome);
            return outcome;
        }
    };
    let mut outcome = JobOutcome {
        id: job.id.clone(),
        terminal: JobTerminal::Failed,
        page_id: Some(page_id),
        thread_url: None,
        error: None,
        note: None,
        elapsed: Duration::ZERO,
    };
    macro_rules! step {
        ($body:expr) => {
            if let Err(e) = run_on_page(ctx, page_id, || $body) {
                outcome.error = Some(e);
                outcome.elapsed = started.elapsed();
                write_job_metadata(ctx, job, provider, &outcome);
                return outcome;
            }
        };
    }

    // Login must hold for this provider; check on the job's own tab.
    step!(
        match check_login_status(config_path, provider, ctx.verbose) {
            Ok(LoginState::LoggedOut) => Err(format!(
                "not logged in to {}; run `ask-bridge --provider {} login` first",
                provider.display_name(),
                provider
            )),
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    );

    if provider == Provider::ChatGpt {
        step!(ensure_chatgpt_chat_mode(config_path, ctx.verbose));
    }
    if job.temporary {
        step!(ensure_temporary_chat(config_path, provider, ctx.verbose));
    }
    if let Some(model) = &selection_plan.model {
        let model = model.clone();
        step!(switch_model(config_path, provider, &model, ctx.verbose));
    }
    if let Some(reasoning) = selection_plan.reasoning {
        step!(switch_reasoning(
            config_path,
            provider,
            reasoning,
            ctx.verbose
        ));
    }
    if !job.images.is_empty() || !job.files.is_empty() {
        step!(upload_attachments_to_provider(
            config_path,
            provider,
            &job.images,
            &job.files,
            ctx.verbose
        ));
    }

    let mut initial_count = 0usize;
    step!(match count_assistant_messages(config_path, provider) {
        Ok(count) => {
            initial_count = count;
            Ok(())
        }
        Err(e) => Err(e),
    });

    // Submission is the one step that must never be retried blindly: a
    // transport failure here is ambiguous (the prompt may already be in),
    // so the job becomes Recoverable instead of Failed.
    let submit_result = run_on_page(ctx, page_id, || {
        submit_prompt_to_provider(config_path, provider, &job.prompt, ctx.verbose).map(|_| ())
    });
    if let Err(e) = submit_result {
        outcome.terminal = if mcp_error_is_transport(&e) || e.contains("MCP session was reset") {
            JobTerminal::Recoverable
        } else {
            JobTerminal::Failed
        };
        outcome.error = Some(format!("prompt submission failed: {}", e));
        outcome.elapsed = started.elapsed();
        write_job_metadata(ctx, job, provider, &outcome);
        return outcome;
    }

    // Record the thread URL as early as possible for recovery.
    let _ = run_on_page(ctx, page_id, || {
        if let Some(url) = current_page_url(config_path) {
            ctx.registry.set_thread_url(&job.id, &url);
            outcome.thread_url = Some(url);
        }
        Ok(())
    });

    // Wait for completion, polling only inside short locked sections so
    // other jobs keep making progress between polls.
    let wait_timeout = Duration::from_secs(job.timeout_secs);
    let wait_started = Instant::now();
    let mut stable_done_checks = 0usize;
    let mut image_state = ChatGptImageResponseState::default();
    let mut finished = false;
    let mut cycles = 0usize;
    while wait_started.elapsed() < wait_timeout {
        std::thread::sleep(Duration::from_millis(100));
        cycles += 1;
        if cycles % 5 != 0 {
            continue;
        }
        let poll = run_on_page(ctx, page_id, || {
            poll_response_once(config_path, provider, initial_count)
        });
        match poll {
            Ok(Some(parsed)) => {
                if provider == Provider::ChatGpt {
                    image_state.observe(&parsed);
                }
                let status = parsed["status"].as_str().unwrap_or("waiting");
                let is_new = parsed["isNew"].as_bool().unwrap_or(false);
                if status == "done" && is_new {
                    stable_done_checks += 1;
                    if stable_done_checks >= 3 {
                        finished = true;
                        break;
                    }
                } else {
                    stable_done_checks = 0;
                }
            }
            Ok(None) => stable_done_checks = 0,
            Err(_) => stable_done_checks = 0,
        }
    }

    if !finished {
        outcome.terminal = JobTerminal::TimedOut;
        outcome.error = Some(format!(
            "response did not complete within {} seconds",
            job.timeout_secs
        ));
        let _ = run_on_page(ctx, page_id, || {
            if let Some(url) = current_page_url(config_path) {
                ctx.registry.set_thread_url(&job.id, &url);
                outcome.thread_url = Some(url);
            }
            Ok(())
        });
        outcome.elapsed = started.elapsed();
        write_job_metadata(ctx, job, provider, &outcome);
        return outcome;
    }

    if let Err(e) = image_state.ensure_completed(true) {
        outcome.error = Some(e);
        outcome.elapsed = started.elapsed();
        write_job_metadata(ctx, job, provider, &outcome);
        return outcome;
    }

    // Extract the response on the job's own page (the clipboard sequence is
    // atomic under the control lock).
    let mut markdown = String::new();
    step!(match copy_latest_markdown(config_path, provider) {
        Ok(content) => {
            markdown = content;
            Ok(())
        }
        Err(e) => Err(format!("failed to extract response: {}", e)),
    });

    let job_dir = job_output_dir(&ctx.output_dir, &job.id);
    if let Err(e) = write_atomic(&job_dir.join("response.md"), markdown.as_bytes()) {
        outcome.error = Some(e);
        outcome.elapsed = started.elapsed();
        write_job_metadata(ctx, job, provider, &outcome);
        return outcome;
    }

    if job.download_images {
        let images_dir = job_dir.join("images");
        let _ = std::fs::create_dir_all(&images_dir);
        let images_path = images_dir.to_string_lossy().to_string();
        let remaining = wait_timeout.saturating_sub(wait_started.elapsed());
        let download = run_on_page(ctx, page_id, || {
            download_images_from_latest_message(
                config_path,
                provider,
                Some(images_path.as_str()),
                remaining,
                Some(initial_count),
                ctx.verbose,
            )
        });
        if let Err(e) = download {
            outcome.note = Some(format!("image download warning: {}", e));
        }
    }

    let _ = run_on_page(ctx, page_id, || {
        if let Some(url) = current_page_url(config_path) {
            ctx.registry.set_thread_url(&job.id, &url);
            outcome.thread_url = Some(url);
        }
        Ok(())
    });

    outcome.terminal = JobTerminal::Completed;
    outcome.elapsed = started.elapsed();
    write_job_metadata(ctx, job, provider, &outcome);

    // Completed jobs close their own tab so the tab count stays bounded by
    // the concurrency level instead of by the total job count.
    let _ = with_control_lock(|| {
        call_mcp_tool(
            config_path,
            "close_page",
            serde_json::json!({ "pageId": page_id }),
        )?;
        Ok(())
    });
    outcome
}

fn write_job_metadata(
    ctx: &BatchContext,
    _job: &JobSpec,
    provider: Provider,
    outcome: &JobOutcome,
) {
    let path = job_output_dir(&ctx.output_dir, &outcome.id).join("metadata.json");
    let _ = write_atomic(
        path.as_path(),
        outcome_metadata(outcome, provider).as_bytes(),
    );
}

// ---------------------------------------------------------------------------
// Command entry point
// ---------------------------------------------------------------------------

/// Read and validate the batch inputs (jobs file/stdin and the concurrency
/// gate) *before* any browser startup, so configuration errors fail fast
/// without launching Chrome.
pub fn prepare_batch(input: Option<&str>, concurrency: u32) -> Result<Vec<JobSpec>, String> {
    if concurrency != 1 {
        return Err(
            "batch --concurrency greater than 1 is not enabled yet; run with --concurrency 1"
                .to_string(),
        );
    }
    let input_text = match input {
        Some("-") | None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("Failed to read jobs from stdin: {}", e))?;
            buf
        }
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read jobs file {}: {}", path, e))?,
    };
    parse_jobs_jsonl(&input_text)
}

pub fn run_batch_command(
    config_path: &str,
    default_provider: Provider,
    headless: bool,
    verbose: bool,
    jobs: Vec<JobSpec>,
    concurrency: u32,
    output_dir: &str,
    fail_fast: bool,
) -> Result<i32, String> {
    let ctx = BatchContext {
        config_path: config_path.to_string(),
        default_provider,
        headless,
        verbose,
        output_dir: PathBuf::from(output_dir),
        registry: TargetRegistry::new(),
    };
    std::fs::create_dir_all(&ctx.output_dir).map_err(|e| {
        format!(
            "Failed to create output directory {}: {}",
            ctx.output_dir.display(),
            e
        )
    })?;

    println!(
        "[batch] running {} job(s), concurrency {}",
        jobs.len(),
        concurrency
    );
    let (outcomes, summary) = execute_batch_sequential(&jobs, fail_fast, |job| {
        println!("[batch] job {}: running", job.id);
        run_job(&ctx, job)
    });
    for (job, outcome) in jobs.iter().zip(&outcomes) {
        match outcome.terminal {
            JobTerminal::Completed => println!(
                "[batch] job {}: completed in {}s -> {}",
                outcome.id,
                outcome.elapsed.as_secs(),
                job_output_dir(&ctx.output_dir, &job.id)
                    .join("response.md")
                    .display()
            ),
            JobTerminal::Cancelled => {
                let provider = job.provider.unwrap_or(default_provider);
                write_job_metadata(&ctx, job, provider, outcome);
                println!("[batch] job {}: cancelled", outcome.id);
            }
            _ => println!(
                "[batch] job {}: {} ({})",
                outcome.id,
                outcome.terminal.as_str(),
                outcome.error.as_deref().unwrap_or("")
            ),
        }
    }
    println!("[batch] summary\n{}", summary.render());
    Ok(summary.exit_code())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str) -> JobSpec {
        JobSpec {
            id: id.to_string(),
            provider: None,
            prompt: "hello".to_string(),
            timeout_secs: DEFAULT_JOB_TIMEOUT_SECS,
            session: None,
            temporary: false,
            model: None,
            reasoning: None,
            files: Vec::new(),
            images: Vec::new(),
            download_images: true,
        }
    }

    fn outcome(id: &str, terminal: JobTerminal) -> JobOutcome {
        JobOutcome {
            id: id.to_string(),
            terminal,
            page_id: None,
            thread_url: None,
            error: None,
            note: None,
            elapsed: Duration::ZERO,
        }
    }

    #[test]
    fn parses_minimal_and_full_jobs() {
        let jobs = parse_jobs_jsonl(
            "{\"prompt\":\"first\"}\n\n{\"id\":\"img-1\",\"provider\":\"ChatGPT\",\"prompt\":\"draw\",\"timeout\":60,\"files\":[\"a.pdf\"],\"downloadImages\":false}\n",
        )
        .unwrap();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, "job-1");
        assert_eq!(jobs[0].timeout_secs, DEFAULT_JOB_TIMEOUT_SECS);
        assert!(jobs[0].download_images);
        assert_eq!(jobs[1].id, "img-1");
        assert_eq!(jobs[1].provider, Some(Provider::ChatGpt));
        assert_eq!(jobs[1].timeout_secs, 60);
        assert_eq!(jobs[1].files, vec!["a.pdf".to_string()]);
        assert!(!jobs[1].download_images);
    }

    #[test]
    fn rejects_invalid_job_definitions() {
        // duplicate id
        assert!(
            parse_jobs_jsonl("{\"id\":\"a\",\"prompt\":\"x\"}\n{\"id\":\"a\",\"prompt\":\"y\"}")
                .is_err()
        );
        // unsafe id (path traversal)
        assert!(parse_jobs_jsonl("{\"id\":\"../evil\",\"prompt\":\"x\"}").is_err());
        // missing prompt
        assert!(parse_jobs_jsonl("{\"id\":\"a\"}").is_err());
        // unknown field
        assert!(parse_jobs_jsonl("{\"prompt\":\"x\",\"bogus\":1}").is_err());
        // unknown provider
        assert!(parse_jobs_jsonl("{\"prompt\":\"x\",\"provider\":\"copilot\"}").is_err());
        // timeout 0
        assert!(parse_jobs_jsonl("{\"prompt\":\"x\",\"timeout\":0}").is_err());
        // new: false unsupported
        assert!(parse_jobs_jsonl("{\"prompt\":\"x\",\"new\":false}").is_err());
        // session + temporary conflict
        assert!(
            parse_jobs_jsonl("{\"prompt\":\"x\",\"session\":\"abc\",\"temporary\":true}").is_err()
        );
        // gemini images unsupported
        assert!(
            parse_jobs_jsonl("{\"prompt\":\"x\",\"provider\":\"gemini\",\"images\":[\"a.png\"]}")
                .is_err()
        );
        // empty input
        assert!(parse_jobs_jsonl("\n\n").is_err());
    }

    #[test]
    fn generated_ids_avoid_collisions_with_explicit_ids() {
        let jobs =
            parse_jobs_jsonl("{\"prompt\":\"a\"}\n{\"id\":\"job-1\",\"prompt\":\"b\"}\n").unwrap();
        assert_eq!(jobs[0].id, "job-1-2");
        assert_eq!(jobs[1].id, "job-1");
    }

    #[test]
    fn registry_binds_once_and_tracks_thread_url() {
        let registry = TargetRegistry::new();
        registry.bind("a", 3).unwrap();
        registry.bind("a", 3).unwrap();
        assert!(registry.bind("a", 4).is_err());
        registry.bind("b", 4).unwrap();
        registry.set_thread_url("a", "https://chatgpt.com/c/1");
        assert_eq!(registry.get("a").unwrap().page_id, 3);
        assert_eq!(
            registry.get("a").unwrap().thread_url.as_deref(),
            Some("https://chatgpt.com/c/1")
        );
        assert_eq!(registry.get("b").unwrap().thread_url, None);
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn sequential_executor_honors_fail_fast_and_counts_summary() {
        let jobs = vec![spec("a"), spec("b"), spec("c")];
        let (outcomes, summary) = execute_batch_sequential(&jobs, true, |job| {
            if job.id == "b" {
                outcome("b", JobTerminal::Failed)
            } else {
                outcome(&job.id, JobTerminal::Completed)
            }
        });
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[2].terminal, JobTerminal::Cancelled);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.cancelled, 1);
        assert_eq!(summary.exit_code(), 1);

        let (_, summary) =
            execute_batch_sequential(&jobs, false, |job| outcome(&job.id, JobTerminal::Completed));
        assert_eq!(summary.completed, 3);
        assert_eq!(summary.exit_code(), 0);
    }

    #[test]
    fn write_atomic_replaces_content_without_leftovers() {
        let base =
            std::env::temp_dir().join(format!("ask-bridge-batch-test-{}", std::process::id()));
        let path = base.join("job-x").join("response.md");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn summary_render_uses_exclusive_terminal_counts() {
        let mut summary = BatchSummary::default();
        summary.record(JobTerminal::Completed);
        summary.record(JobTerminal::TimedOut);
        summary.record(JobTerminal::Recoverable);
        let rendered = summary.render();
        assert!(rendered.contains("Total: 3"));
        assert!(rendered.contains("Completed: 1"));
        assert!(rendered.contains("Timed out: 1"));
        assert!(rendered.contains("Recoverable: 1"));
    }
}
