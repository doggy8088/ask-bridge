# 更新日誌 (CHANGELOG)

本專案的所有重要變更皆會記錄於本文件中。

## [Unreleased]

---

## [0.4.0] - 2026-11-18

### 新增 (Added)
- 新增 ChatGPT 選用 SIWC 後端：以 OAuth 動態註冊與 PKCE 登入，使用 ChatGPT Work／Codex 方案額度或 credits 呼叫公開 Responses API，免 API Key；預設 Browser/CDP 行為不變。
- SIWC 支援單次純文字提問、stdin、帳號模型目錄與 `--model`、SSE 即時輸出、`-o`、整體請求逾時、重新登入與本機登出；可用 `config --chatgpt-backend siwc` 儲存預設值。
- 新增 `batch` JSONL 批次工作，可設定 `--concurrency` 上限並行等待各自分頁的回覆，提供獨立輸出目錄、metadata、生成圖片、終態摘要與 `--fail-fast`。
- 新增 `--temporary`，支援 ChatGPT／Gemini 臨時聊天與 Claude Incognito；送出前驗證模式，無法確認時中止，不送出問題。
- 新增可設定的 Chromium 瀏覽器路徑及獨立 profile：支援 CLI、環境變數與持久設定，設定路徑不存在時直接報錯，不自動退回 Chrome。
- 新增 ChatGPT 圖片生成百分比與圖片就緒狀態追蹤，等待生成完成後再下載圖片。

### 修正 (Fixed)
- 修正 Windows 更新流程使用 PowerShell `$pid` 與內建唯讀 `$PID` 衝突，導致更新失敗。
- 修正 ChatGPT 生圖的最新回覆判定、100% 但尚未載入完成、多圖下載完整性與逾時回報，避免未完成結果被誤報成功。
- 排除引用來源 favicon，避免純文字回覆卡在圖片掃描；完成的文字先寫入 `-o`，圖片下載失敗未指定 `-i` 時回報警告，指定 `-i` 時仍以失敗結束。
- 強化批次分頁選取、擷取與 session 綁定的並行保護；對送出結果不明確的工作保留恢復資訊，不自動重送，並拒絕並行工作指向相同既有對話。
- 強化 SIWC OAuth 驗證、授權網址遮罩、token 更新、SSE UTF-8 跨區塊解析及整體逾時；僅收到 `response.completed` 才視為成功，拒絕不支援的選項與子指令，不靜默切換後端或成本來源。
- SIWC HTTPS 合併內建與系統 CA 信任庫，修正 Windows `UnknownIssuer` 相容性問題，並顯示不含請求 URL、headers 或 body 的底層連線原因。
- 消除多餘初始賦值與未使用函式的三項編譯警告；修正網站手機選單遮擋及韓文長字串溢出。

### 文件與發布 (Documentation and Release)
- 擴充中英文 README 的 SIWC 快速上手、工作上下文、手動接續討論、功能矩陣、憑證管理及疑難排解，並說明 SIWC 不會自動讀取工作目錄、IDE 或 Agent 對話。
- 首頁新增 SIWC 導覽及完整區塊，同步繁體中文、簡體中文、英文、日文與韓文，提供可複製指令、鍵盤可捲動比較表與中英文指南連結。
- 補上方案額度可能與主要 Coding Agent 共用、非免費或無限 token、TLS 檢查可見 token 內容，以及本機 logout 不等同伺服器撤銷授權的說明。
- 發布流程改由 Release 完成上傳後觸發 npm trusted publishing，並以腳本同步版本、驗證 CI、平台產物與 npm 套件。

### 注意事項 (Notes)
- SIWC 目前僅支援 ChatGPT 單次純文字推論；不支援網頁 session、多輪歷史管理、附件、生圖、`--reasoning`、`--temporary` 或 `batch`，相關選項會明確拒絕。
- 依 issue #40 結案紀錄，已在 Windows 實測 SIWC 登入與單次文字對話；refresh 輪替與失效、撤銷效果、額度耗盡、網路中斷及效能／成本基準尚未逐項完成真實環境驗證，不宣稱必然更快或更省 token。

---

## [0.3.1] - 2026-10-15

### 🔧 修復 (Fixed)
- 修正新版 ChatGPT 生圖回覆沒有 `:assistant` 回合單元，導致工具等到逾時且未下載圖片；現在會辨識 `generated-image-gallery` 圖片回覆並正常下載。
- 複製回覆時排除「複製圖像」按鈕，避免把圖片而非文字送進剪貼簿。

---

## [0.3.0] - 2026-10-15

### 🚀 新增 (Added)
- ChatGPT `--reasoning` 新增 `xhigh`（極高），對應新版推理強度滑桿。
- 送出前自動將 ChatGPT 撰寫器從「工作（Work）」切回「對話（Chat）」模式。

### 🔧 修復 (Fixed)
- 適配 ChatGPT 改版後的 DOM：新的 ProseMirror 撰寫器、回覆回合容器、`in_progress` 串流狀態與個人檔案選單。
- 修正 ChatGPT 複製按鈕產生的程式碼區塊遺失換行與語言標記，改以 CodeMirror 編輯器內容修補。
- 修正 ChatGPT `--model`／`--reasoning` 無法操作新版模型選單（推理滑桿與模型清單分頁）。
- 修正 ChatGPT 登出時被導向 `/auth/login` 會卡在載入逾時，現在會正確提示重新登入。
- 修正 `login` 沿用背景 Chrome 時視窗位於螢幕外而無法登入，現在會改以可見模式重新啟動。

---

## [0.2.5] - 2026-07-10

### 🚀 新增 (Added)
- 新增 `ask-bridge` 問答命令的等待逾時參數（`--timeout`），可自訂回應等待秒數。
- 調整預設回應等待逾時為 `300` 秒（`--timeout` 預設值），降低長時間等待中斷機率。
- `ask-bridge login` 新增登入頁面背景輪詢完成檢測，減少手動切換視窗等待時間。
- Windows 安裝腳本新增本地建構安裝模式（`install.ps1 -Local`），可直接安裝 `target\\release\\ask-bridge.exe`，並保留 `ask.exe` alias 安裝。

### 🔧 修復 (Fixed)
- 修正 npm publish 版本對齊流程，避免在重複發佈時版本比對失敗。
- 修正 CI 平台條件，避免在 Windows 專用 parser 測試執行於非 Windows 平台。
- 修正 `bump-and-release` 首版 SOP 與版本搜尋流程，提升版本升級一致性。

---

## [0.2.3] - 2026-07-10

### 🔧 修復 (Fixed)
- 在 Windows 安裝流程加入 Node.js 版本預檢，限制 `node --version` 至 `^20.19.0`、`^22.12.0` 或 `>=23.0.0`，避免安裝後才遇到 MCP 相容性錯誤。
- 在 CLI 啟動前加入 Node.js Runtime 檢查，若版本不符合 chrome-devtools-mcp 要求，提前中止並輸出可行動的錯誤訊息（含重開終端與安裝建議）。
- 補齊 Node 版本判斷的單元測試：覆蓋支援邊界值與錯誤格式，降低版本不相容回歸風險。

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
