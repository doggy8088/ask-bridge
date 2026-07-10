# 2026-07-10 修正 Windows MCP Node 版本錯誤診斷

## Goal + Acceptance Criteria
- [x] Node.js `v20.17.0` 不再一路進入 MCP `initialize` 後只顯示誤導性的 schema 錯誤。
- [x] Windows 安裝器在下載 binary 前驗證 `chrome-devtools-mcp@latest` 的 Node engines 契約。
- [x] 既有安裝、直接下載與 npm 安裝都由 Rust runtime preflight 兜底。
- [x] `config` 與 `close` 等不需 MCP 的命令不受 Node 版本檢查影響。
- [x] 回歸測試涵蓋 `20.19`、`22.12` 邊界、較舊版本與無效版本輸出。
- [ ] 通過格式、目標測試、完整測試與 `cargo check` 驗證。

## Risk & Rollback
- Risk level: low
- Affected components: Windows 安裝前置檢查、所有需要 MCP 的 runtime 命令。
- Rollback strategy: revert `src/main.rs` 與 `install.ps1` 的 Node preflight；不涉及資料、設定格式或 migration。
- Monitoring signals: 不相容 Node 應在 Chrome 啟動前顯示實際版本與支援範圍，不應再進入 MCP `initialize`。

## Dependencies & Environment
- 上游 `chrome-devtools-mcp@latest` engines：`^20.19.0 || ^22.12.0 || >=23`。
- 使用者截圖：Node.js `v20.17.0`、npm/npx `11.12.1`、ask-bridge `0.2.2`。
- Cargo target 輸出沿用 `%TEMP%\ask-bridge-target`，避免 `G:` 空間不足。

## Working Notes
- Windows quiet MCP 設定使用 `cmd.exe /c ... 2>nul`，因此 mcp-cli 只能回報 `No stderr output available`。
- `Check tool arguments match the expected schema` 是 mcp-cli 的通用 fallback，不是本案 schema 錯誤的證據。
- Runtime preflight 位於 `config`/`close` early return 之後、`write_mcp_config` 與 Chrome 啟動之前。

## Checklist
- [x] Review `tasks/lessons.md`
- [x] Confirm upstream Node engines and reproduce the version mismatch
- [x] Locate MCP startup and installer validation paths
- [x] Implement runtime and Windows installer fail-fast checks
- [x] Add Node version boundary regression tests
- [ ] Run targeted and full verification
- [ ] Summarize changes + verification story

---

# 2026-07-10 修正 Windows ChatGPT 登入延續回歸

## Goal + Acceptance Criteria
- [x] Windows 上 Chrome launcher PID 與 9223 listener PID 不同時，仍能辨識為 ask-bridge 所啟動的 Chrome。
- [x] `login` 結束後直接執行 query，不再誤報 `Port 9223 is already used by a non-ask Chrome process`。
- [x] ChatGPT 頁面登入 UI 尚在 hydration 時，不會因單次暫態訊號立即誤報未登入。
- [x] 明確位於 auth path 或穩定呈現未登入控制項時，仍安全地阻止 query 並提示登入。
- [x] `close` 與啟動重用採相同 ownership 規則，且 Windows 優先正常關閉、逾時才強制終止。
- [x] 回歸測試涵蓋實際 listener PID 記錄、PID fallback、WMIC 空白輸出與登入訊號穩定化。
- [x] 通過格式、Rust tests/check 與 diff whitespace 驗證（未執行 clippy / npm 測試）。

## Results
- `src/main.rs`：
  - 移除已淘汰的 `write_chrome_pid`/舊版 listener 回溯測試。
  - 重構 `start_chrome_if_needed` 與 `close_ask_chrome_on_debug_port` 以以 `CDP browser_id + debug listener` 為主、且由 parent-chain 尋找 `ask-bridge` 擁有者。
  - 將啟動與關閉重用判斷的 `Chrome` ownership 與 `build_chrome_process_record` 對齊。
  - ChatGPT 登入訊號加入 `stable` 欄位與穩定化等待，降低一次性 DOM 暫態誤判。
- 驗證：
  - `cargo fmt --all -- --check`
  - `cargo check`
  - `cargo test`

## Risk & Rollback
- Risk level: medium
- Affected components: Windows Chrome process ownership、9223 listener 重用／關閉、ChatGPT 登入前置判斷。
- Rollback strategy: revert `src/main.rs` 的 listener PID 與登入穩定化變更；不涉及資料格式或 migration。
- Rollout plan: 先以純函式測試與本機 Windows listener 驗證，再由受影響使用者重跑 login → query 原始流程。
- Monitoring signals: verbose diagnostics 中 recorded PID 應等於 listener PID，owner PIDs 不得為空；query 不得再出現 non-ask 或暫態未登入誤判。

## Dependencies & Environment
- Rust/Cargo、Node.js/npm 與本機 Google Chrome。
- `chrome-devtools-mcp@latest` 的 `evaluate_script` 支援 async function，可在單一 MCP 呼叫內等待登入 DOM 穩定。
- 本機 9223 已有既存 ask-bridge Chrome，手動驗證不得破壞其 profile 或登入資料。

## Working Notes
- 使用者證據：launcher/recorded PID `15864`、listener PID `20728`、owner PIDs `[]`；現有 `chrome.pid` 只記 launcher 且未參與 ownership 判定。
- `start_chrome_if_needed` 會因 owner 空集合誤判既有 Chrome；`close_ask_chrome_on_debug_port` 又使用另一套更窄的直接 command-line 判斷。
- Windows WMIC command-line parser 只讀 header 後第一行，遇到空白行便失敗；CIM fallback 也可能受環境限制。
- v0.2.1 的 ChatGPT ready check 只等 composer；訪客 shell 與登入 hydration 都可能先出現 composer，隨後的單次登入訊號便可能暫態為 LoggedOut。
- 登入完成當下已得到 Unknown，證明現行 account-menu selector 不是可靠的唯一已登入依據。

## Checklist
- [x] Review `tasks/lessons.md` if present（檔案不存在）
- [x] Locate existing implementation / patterns and preserve baseline evidence
- [x] Design minimal approach + key decisions
- [x] Implement listener PID ownership fallback and consistent close resolution
- [x] Make ChatGPT login decision tolerate hydration without masking stable logged-out state
- [x] Add/adjust regression tests
- [x] Run targeted and full verification
- [ ] Review correctness/security/performance of final diff
- [ ] Summarize changes + verification story
- [ ] Record lessons if any correction/postmortem occurs

---

# 2026-07-09 修正 WSL Chrome 路徑偵測

## Goal + Acceptance Criteria
- [x] 修正 `ask-bridge --verbose login` 在 WSL/Linux 只尋找 macOS Chrome 路徑的問題。
- [x] 在 Linux/WSL 中可偵測 `PATH` 內的 `google-chrome` / `google-chrome-stable`，並支援 `/usr/bin/google-chrome` 等常見路徑。
- [x] macOS 與 Windows 既有 Chrome 偵測行為不被破壞。
- [x] 加入可重現此路徑選擇行為的單元測試。
- [x] 通過 `cargo fmt --all -- --check` 與 Rust 編譯/測試驗證。

## Risk & Rollback
- Risk level: low
- Affected components: Chrome 啟動前的可執行檔路徑解析。
- Rollback strategy: revert `src/main.rs` 中 Chrome path resolver 相關變更。

## Dependencies & Environment
- Rust/Cargo local toolchain。
- Linux/WSL 目標環境需已安裝 Google Chrome，且 `google-chrome` 或 `google-chrome-stable` 可由 `PATH` 或常見絕對路徑找到。

## Working Notes
- 現有 `find_chrome_path` 對 `#[cfg(not(target_os = "windows"))]` 一律檢查 `/Applications/Google Chrome.app/...`，導致 Linux/WSL 誤報 macOS 路徑。
- `install.sh` 已知道 Linux 應檢查 `google-chrome` 或 `google-chrome-stable`，runtime 偵測邏輯需要與此一致。

## Checklist
- [x] Review `tasks/lessons.md` if present
- [x] Locate existing implementation / patterns
- [x] Design minimal approach + key decisions
- [x] Implement smallest safe slice
- [x] Add/adjust tests
- [x] Run verification (format/tests/build)
- [x] Summarize changes + verification story
- [x] Record lessons if any correction/postmortem occurs (none needed)

## Results
- `src/main.rs` now keeps macOS detection on the macOS-only branch and adds Linux detection for `google-chrome` / `google-chrome-stable` via `PATH`, then common absolute paths including `/usr/bin/google-chrome`.
- `Makefile install-browser` now handles Linux separately instead of applying macOS-only Chrome detection to every Unix-like OS.
- Added unit coverage for Linux Chrome path lookup from `PATH`, fallback candidates, and missing Chrome.
- Verification:
  - `cargo fmt --all -- --check` passed.
  - `cargo test` initially failed on `G:` because only about 768 KB was free (`os error 112` / no space on device).
  - Windows-hosted verification passed with `CARGO_TARGET_DIR=%TEMP%\ask-bridge-target cargo test` (`21 passed`).
  - WSL/Linux verification passed with `CARGO_TARGET_DIR=/mnt/c/Users/wakau/AppData/Local/Temp/ask-bridge-target-wsl cargo test` (`21 passed`).
- Manual WSL Chrome launch was not verified in this environment because `Ubuntu-24.04` reports `google-chrome: command not found`; the user's reported `/usr/bin/google-chrome` path is covered by the new Linux fallback list.

---

# 2026-07-09 bump-and-release 0.1.4

## Goal + Acceptance Criteria
- [ ] Release patch version `0.1.4` for the WSL/Linux Chrome path fix.
- [ ] Keep the required 6 version locations synchronized: `Cargo.toml`, `package.json`, `src/main.rs`, `install.ps1`, `install.sh`, `scripts/ask.sh`.
- [ ] Update `Cargo.lock` through Cargo verification.
- [ ] Update `CHANGELOG.md` with the release entry.
- [ ] Commit version bump, create annotated tag `v0.1.4`, push branch and tag.

## Risk & Rollback
- Risk level: low
- Affected components: package metadata, installer download version, CLI version display, changelog.
- Rollback strategy: revert the version bump commit and delete `v0.1.4` locally/remotely if the release must be withdrawn.

## Dependencies & Environment
- Cargo and npm available locally.
- `G:` has insufficient free space for Cargo target output; use `%TEMP%` / `/mnt/c/...` target directories for heavy Cargo commands.
- Git remote is `origin https://github.com/doggy8088/ask-bridge.git`.

## Working Notes
- Patch bump is appropriate because the preceding change is a bug fix without breaking API/CLI behavior.
- Existing WSL Chrome fix was committed separately as `94c1912` before release version changes.

## Checklist
- [x] Analyze bump type
- [x] Update version files
- [x] Run verification
- [x] Commit release bump
- [x] Create and push tag
- [x] Summarize release outcome

## Results
- Synchronized version `0.1.4` in `Cargo.toml`, `package.json`, `src/main.rs`, `install.ps1`, `install.sh`, and `scripts/ask.sh`.
- Updated `Cargo.lock` through `cargo check`.
- Added `CHANGELOG.md` entry for `0.1.4`.
- Verification passed:
  - `CARGO_TARGET_DIR=%TEMP%\ask-bridge-target cargo check`
  - `cargo fmt --all -- --check`
  - `CARGO_TARGET_DIR=%TEMP%\ask-bridge-target cargo test` (`21 passed`)
  - `CARGO_TARGET_DIR=/mnt/c/Users/wakau/AppData/Local/Temp/ask-bridge-target-wsl cargo test` (`21 passed`)
  - `npm test` (`4 passed`)
- Git results:
  - Bug fix commit: `94c1912`
  - Release commit: `58cb9ca`
  - Pushed `main` to `origin/main`
  - Created and pushed annotated tag `v0.1.4`
