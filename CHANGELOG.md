# 更新日誌 (CHANGELOG)

本專案的所有重要變更皆會記錄於本文件中。

## [Unreleased]

### 🚀 新增 (Added)
- **Claude 網頁版 provider 支援**：新增 `--provider claude`，可透過 claude.ai 網頁送出文字 prompt 並取回回覆。登入偵測、composer、送出／停止按鈕與回覆容器 selector 均採語意屬性（`data-testid` / `aria-label`）並經實站校正。第一版僅支援純文字 prompt，尚不支援 `--image` / `--file` 附件與 `--model` 模型切換（由 `validate_provider_feature_support` 明確擋下並回報清楚訊息）。

### 🔧 修復 (Fixed)
- 回應 codex 對抗審查／PR review comments：(1) Claude `ready_check` 加入登入頁指標（`/login` 連結或 Log in／Sign in／登入 文字），避免登出／新 profile 首次 `login` 卡到 timeout；(2) Claude assistant／回覆 selector 改用 `div[data-is-streaming="false"]`（僅完成的訊息），避免串流中被誤判為完成而截斷（實測 claude 完成後容器為 `data-is-streaming="false"`）。

---

## [0.1.4] - 2026-07-09

### 🔧 修復 (Fixed)
- 修正 Linux/WSL 執行 `ask-bridge --verbose login` 時誤尋找 macOS Chrome 路徑的問題，現在會偵測 `PATH` 中的 `google-chrome` / `google-chrome-stable`，並支援 `/usr/bin/google-chrome` 等常見安裝路徑。
- 修正 `make install` 在 Linux/WSL 環境下的 Chrome 檢查邏輯，避免套用 macOS-only 的 `/Applications/Google Chrome.app` 偵測。

---

## [0.1.3] - 2026-07-08

### 🔧 變更 (Changed)
- 將 `mcp-cli` 依賴從本機路徑更換為指向官方 GitHub 倉庫，使其可以持續同步並拉取最新釋出的 `mcp-cli` 版本（已拉取最新 `v0.2.0` 版本）。

---

## [0.1.2] - 2026-07-08

### 🚀 新增 (Added)
- 建立專利維護指南 [AGENTS.md](file:///G:/Projects/ask-bridge/AGENTS.md)，提供後續 AI 協作者完整的開發架構與相容性修復準則。
- 建立 AI 專用技能定義文件 [.agents/skills/bump-and-release/SKILL.md](file:///G:/Projects/ask-bridge/.agents/skills/bump-and-release/SKILL.md)，詳細說明版本號提升 SOP 與 Git 提交標記步驟。

### 🔧 修復 (Fixed)
- **跨平台 Windows 完整支援**：
  - **Google Chrome 路徑自動偵測**：修正原先硬編碼為 macOS 路徑的問題。現在可在 Windows 環境下自動搜尋系統 `Program Files`、`Program Files (x86)` 與 `%LOCALAPPDATA%` 中的預設安裝位置。
  - **行程與連接埠管理**：
    - Windows 環境中改用 `netstat -ano` 代替 `lsof` 搜尋佔用 `9223` 連接埠的處理程序。
    - 優先使用 `wmic` 取得 Chrome 啟動參數確認其擁有權，若失敗則 Fallback 呼叫 `PowerShell` 命令。
    - 在 Windows 下改用 `taskkill /F` 取代 Unix 的 `kill -TERM` 終止處理程序。
  - **系統限制過濾**：使用 `#[cfg(target_os = "macos")]` 條件編譯，確保 Windows 平台不會觸發 macOS 獨有的 `osascript`（AppleScript）命令。
- **編譯警告優化**：消除 Windows 編譯時因條件編譯產生的未使變數（`unused variables`）警告。
- **程式碼排版美化**：使用 `cargo fmt` 重新校正並排版全專案，確保代碼完全符合 Rustfmt 官方規範。

---

## [0.1.1] - 2024-04-10

- 初始公開釋出版。
- 支援透過 macOS Chrome 的遠端除錯協定（連接埠 `9223`）進行 ChatGPT 與 Gemini 自動化。
- 提供 MCP 連接、背景視窗隱藏與快速問答功能。
