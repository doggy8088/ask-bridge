# 更新日誌 (CHANGELOG)

本專案的所有重要變更皆會記錄於本文件中。

## [Unreleased]

### 🚀 新增 (Added)
- **Linux 平台支援**：`find_chrome_path` 新增 Linux 分支，依序偵測 `/usr/bin/google-chrome`、`/usr/bin/google-chrome-stable`、`/opt/google/chrome/chrome` 與 Chromium 等常見路徑；視窗處理與 Windows 一致（螢幕外定位，osascript 仍為 macOS 專屬），PID 偵測／查詢／終止沿用既有 `lsof` / `ps` / `kill`（macOS/Linux 共用）。ChatGPT 與 Gemini 已於 Linux 實機端到端驗證；`make install` 的 `install-browser` 亦改為 OS-aware（Linux 接受 google-chrome / chromium 或給對應套件管理器安裝指引，不再因 macOS-only 檢查而卡住）。

### 🔧 修復 (Fixed)
- 9223 port owner 探測強化：Linux 加入 `ss` fallback（`lsof` 缺失時備援），避免 PID 探測失敗時誤判 9223 上的 Chrome 非本工具所有。
- `find_chrome_path` Linux 分支改用 `std::env::split_paths` 掃描 `PATH`（純 Rust，不依賴外部 `which`），涵蓋 `/usr/local/bin`、`~/.local/bin`、Nix、Linuxbrew 等非標準安裝，與 `install.sh` 的 `command -v` 偵測一致，避免 installer 通過但 runtime 找不到。

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
