# 🤖 AI Agents 開發與維護指南 (AGENTS.md)

歡迎來到 `ask-bridge` 專案！本文件專為前來本專案進行開發、維護或升級的 AI 代理人（Agents）所撰寫。它記錄了本專案的核心架構、跨平台邏輯、開發工作流以及必須遵守的規範。

---

## 📌 專案概述

`ask-bridge` 是一個基於 Rust 語言編寫的命令列工具，旨在透過 **Google Chrome** 的遠端除錯協定（Remote Debugging Protocol）來自動化操控網頁版的 **ChatGPT**、**Gemini** 與 **Claude**，並透過 MCP (Model Context Protocol) 與外部進行對接與問答。

### ⚙️ 核心工作原理
1. **Chrome 遠端除錯**：啟動一個監聽於連接埠 `9223` 的 Chrome 實例（提供專屬的 user-data-dir 設定檔目錄）。
2. **MCP 工具呼叫**：本專案採用 `chrome-devtools-mcp` 套件，透過 Chrome DevTools Protocol 與背景執行的 Chrome 互動，模擬真實使用者輸入與抓取回答。
3. **隱身/背景運行 (Headless)**：提供背景執行模式，並於各系統中以原生方式減少對使用者的視窗干擾。

---

## 💻 跨平台相容性設計 (macOS vs Windows)

本專案支援 **macOS** 與 **Windows** 系統。由於系統底層指令不同，在開發或重構以下功能時需特別注意平台相容性：

### 1. Google Chrome 路徑偵測
* **macOS** 預設路徑：`/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
* **Windows** 偵測邏輯（於 `src/main.rs:find_chrome_path` 實作）：
  1. 優先查詢系統 `Program Files` 底下的 Chrome 安裝路徑。
  2. 若無，則查詢 `Program Files (x86)` 安裝路徑。
  3. 最後查詢使用者本地目錄 `%LOCALAPPDATA%` 下的 Chrome 安裝路徑。

### 2. 視窗隱藏與 AppleScript
* 在 macOS 上，程式會異步調用 `osascript`（AppleScript）以將背景 Chrome 實例隱藏（防止 Dock 閃爍）。
* 在 Windows 平台上，所有 AppleScript 呼叫接已透過條件編譯 `#[cfg(target_os = "macos")]` 予以排除。

### 3. 偵測佔用連接埠 `9223` 的處理程序 (PID)
* **macOS / Linux**：使用 `lsof -tiTCP:9223 -sTCP:LISTEN` 指令。
* **Windows**：使用 `netstat -ano -p tcp` 查詢，並篩選出包含 `:9223` 且狀態為 `LISTENING` 的行，藉此分析出最後一個欄位的 PID。

### 4. 查詢指定 PID 的啟動命令參數
用於確保該 PID 的 Chrome 確實是由 `ask-bridge` 啟動（藉由檢查參數是否包含 `--user-data-dir`）。
* **macOS / Linux**：使用 `ps -p <PID> -o command=`。
* **Windows**：優先調用 `wmic process where processid=<PID> get commandline`。若失敗或系統不支援，則 Fallback 呼叫 PowerShell 的 `Get-CimInstance Win32_Process`。

### 5. 終止處理程序 (Kill Process)
* **macOS / Linux**：使用 `kill -TERM <PID>`。
* **Windows**：使用 `taskkill /F /PID <PID>`。

---

## 🛠️ 開發常用指令 (Makefile Targets)

本專案配有 `Makefile`。AI 代理人可使用下列標準指令進行日常維護：

* **程式碼檢查**：`make check` (執行 `cargo check`)
* **格式化美化**：`make fmt` (執行 `cargo fmt`)
* **編譯測試**：`make build` (Debug) 或 `make release` (Release)
* **功能驗證**：`make test-query` (編譯 Release 並執行快速問答測試)
* **本地安裝安裝**：`make install` / `make uninstall` (建立/刪除全域 symlinks)

---

## 🚀 版本升級與釋出 (Bump & Release)

當需要更新專案版本時，請務必載入專案內置的 AI 代理人專用技能：
* **技能路徑**：`.agents/skills/bump-and-release/SKILL.md`

### ⚠️ 版本更新必須同步修改的 6 個檔案：
1. `Cargo.toml` (`version`)
2. `package.json` (`version`)
3. `src/main.rs` (`#[command(version)]` 巨集)
4. `install.ps1` (`$Version` 變數)
5. `install.sh` (`VERSION` 變數)
6. `scripts/ask.sh` (`VERSION` 變數)

*修改完後，必須執行 `cargo check` 或 `cargo build` 來自動更新 `Cargo.lock`。最後，完成 Git 提交（Commit）與推送（Push）後，必須建立帶有 `vX.Y.Z` 格式的 Git Tag 並推送到遠端倉庫。*

---

## 🚨 代理人開發規範（必守規則）

在您（AI 代理人）完成任何代碼的修改或編輯後，**必須且無條件**遵守下列開發驗證規則：

> [!IMPORTANT]
> 1. **格式化驗證**：所有對原始碼的編輯，最終都**必須**在終端機中執行此命令進行驗證：
>    ```bash
>    cargo fmt --all -- --check
>    ```
>    若檢查失敗，請執行 `cargo fmt --all` 修正格式後再試，確保提交的程式碼完全符合格式規範。
>
> 2. **語言偏好**：撰寫說明文檔、Commit 訊息（可包含英文 Conventional Commits 格式首碼）、或回答使用者時，一律使用**繁體中文 (zh-tw)**。
>
> 3. **Git 提交規範**：
>    * 預設僅提交 staged 變更。若無，請先執行 `git add .`。
>    * 採用 **Conventional Commits 1.0.0** 格式。
>    * 提交訊息必須包含詳盡、具備可讀性的繁體中文主體與項目說明。
