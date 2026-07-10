# 更新日誌 (CHANGELOG)

本專案的所有重要變更皆會記錄於本文件中。

## [Unreleased]

---

## [0.2.2] - 2026-07-10

### 🚀 新增 (Added)
- **Claude（claude.ai）provider 支援**：新增 `--provider claude`，透過 Chrome 自動化 claude.ai 網頁送出 prompt 並取回回覆，與 ChatGPT / Gemini 採相同架構。支援登入偵測（三態 `LoggedIn` / `LoggedOut` / `Unknown`）、分頁重用、`--new` 開新對話、Thread Link 輸出與 `-o` Markdown 檔案輸出。
- Claude 支援 `--image` / `--file` 附件上傳（走既有 DataTransfer 路徑）與 `--model` 模型切換（如 `Sonnet`、`Opus`、`Haiku`，不分大小寫與標點，支援子選單走訪）。
- Selector 依 claude.ai 實站校準：composer 以 `data-testid="chat-input"` 優先；回覆容器採 `.font-claude-response`（`data-is-streaming` 屬性僅掛在最後一則回覆容器，不適合用於訊息計數）。

### 🔧 修復 (Fixed)
- 修正非 Windows 系統（如 macOS、Linux）編譯時，僅在 Windows 平台使用的輔助函數 `parse_windows_netstat_listener_pids` 與 `parse_wmic_column_value` 會產生未使用的編譯警告。

---

## [0.2.1] - 2026-07-10

### 🔧 修復 (Fixed)
- 修正 Windows 執行 `ask-bridge login` 後 Chrome 可能隨命令結束而退出的問題；Chrome 現在會以獨立程序群組與脫離式程序啟動，讓登入工作階段可供後續查詢沿用。
- 強化 `9223` 連接埠的 Chrome 擁有權辨識，加入 ask-bridge 專用標記、PID 紀錄與父程序鏈檢查，避免 Windows Chrome 多程序架構造成誤判。
- 將 ChatGPT 與 Gemini 登入判斷改為 `LoggedIn`、`LoggedOut`、`Unknown` 三態；僅有輸入框時不再誤報登入成功，無法確認時則保留查詢嘗試並顯示警告。
- 多個服務提供者分頁同時存在時優先選取已登入分頁，避免誤選登入頁或未登入分頁。
- Windows `ask-bridge close` 改用 `taskkill /F /PID`，並在程序結束後清理 PID 紀錄。

---

## [0.2.0] - 2026-07-09

### 🚀 新增 (Added)
- 支援 ChatGPT `@Agent` 提示詞輸入；符合 `@名稱 正文` 格式且 Agent 名稱為 1 至 10 個非空白字元時，會先輸入 Agent mention、等待選單出現、按下 Tab 建立 Agent pill，再輸入正文並送出。
- 新增 Agent 提示詞解析與互動流程驗證，涵蓋中文 Agent 名稱、10 字上限、額外空白及不符合格式的輸入。
- 一般 ChatGPT 提示詞與 Gemini 提示詞維持原有送出流程，不套用 Agent 特殊處理。

---

## [0.1.5] - 2026-07-09

### 🔧 修復 (Fixed)
- 修正 ChatGPT 登入判斷過度依賴單一登入按鈕 selector 的問題，改以可見登入控制項、輸入框、帳號選單與登入 URL 綜合判斷。
- 查詢時直接重用已監聽 `9223` 的 ask-bridge Chrome，避免從可見登入模式切換至背景模式時重新啟動 Chrome 並遺失登入狀態。
- 正規化 `--user-data-dir` 命令列比對，支援 Windows 反斜線、引號及參數值以空白分隔的形式。
- 調整 Windows `ask-bridge close` 流程，先嘗試正常終止 Chrome，逾時後再強制關閉。

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
