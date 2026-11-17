# Ask Bridge 🦀

`ask-bridge` 是以 Rust 撰寫的輕量命令列工具，可透過真實 Chrome 瀏覽器自動操作 ChatGPT、Gemini 與 Claude。它使用 Model Context Protocol MCP 與 Chrome DevTools Protocol CDP，並透過內建的 `doggy8088/mcp-cli` Rust library dependency 搭配 `chrome-devtools-mcp` 控制 Chrome、輸入 prompt、送出訊息，並將回覆輸出到終端機。未設定全域 provider 時預設使用 ChatGPT，可用 `--provider gemini`、`--provider claude` 或全域設定檔切換 provider。

## 設計意圖

`ask-bridge` 的核心目的不是取代 ChatGPT、Gemini、Claude 或任何 Coding Agent，而是把它們橋接在一起。開發過程中常會出現大量探索性的 AI 需求，例如查資料、整理文件、比較方案、摘要錯誤訊息、分析程式片段、產生初稿或協助釐清不確定的技術問題。這類任務通常不一定需要由主要 Coding Agent 親自完成，也不一定值得消耗與程式碼編輯、測試、重構等高價值工作相同的 agent 額度。

透過 `ask-bridge`，Coding Agent 可以把這些低風險、探索性、可委派的研究工作轉交給 ChatGPT、Gemini 或 Claude 網站處理，再把網站回覆取回終端機或後續工作流程中。由於 ChatGPT、Gemini、Claude 網站的使用額度與 Coding Agent 的執行額度通常分開計算，`ask-bridge` 可以讓開發者更有彈性地分配 AI 資源：主要 agent 專注在理解專案、修改程式、執行測試與整合結果；網站型 AI 則負責背景研究、文字處理與候選方案產出。

換句話說，`ask-bridge` 是一個給 AI Agent 使用的外部研究橋接器：它把原本需要人類切換瀏覽器、貼上 prompt、等待回覆、再複製結果的流程，包裝成可由命令列驅動的自動化能力。這讓 agent 可以在不離開本機工作流程的情況下，自主向 ChatGPT、Gemini 或 Claude 發出請求，取得輔助資訊，並將其納入後續判斷。

此工具特別適合：

- 將大型文件、錯誤訊息或程式片段交給網站型 AI 做摘要、比對或初步分析。
- 讓 Coding Agent 在實作前先委外蒐集背景資料、整理替代方案或產生檢查清單。
- 把不需要直接修改專案檔案的 AI 任務移出主要 agent 執行流程。
- 利用既有 ChatGPT、Gemini 或 Claude 網頁帳號的能力，處理 API 以外的互動式網站功能。

`ask-bridge` 不保證網站 provider 的輸出一定正確，也不應取代本機測試、官方文件查證或人工審查。它的定位是降低探索性 AI 工作的操作成本，讓主要 Coding Agent 能以更低摩擦取得外部 AI 協助。

在預設的瀏覽器後端中，`ask-bridge` 會在真實 Chrome 瀏覽器中執行，並使用持久化的專屬使用者 profile。這表示：

- 只需要手動登入一次，透過 `ask-bridge login` 完成。
- 可處理 CAPTCHA、Cloudflare，並像一般使用者一樣存取所選 provider 的網頁功能。
- Session cookies、登入狀態與對話紀錄會持久保存。

## 主要功能

- **100% Rust 核心**：快速、輕量，編譯後即可執行。
- **多 provider 支援**：使用 `--provider chatgpt|gemini|claude` 選擇 ChatGPT、Gemini 或 Claude。
- **全域 provider 設定**：可在 `~/.config/ask-bridge/config.json` 指定預設 provider，CLI 的 `--provider` 會覆蓋設定檔。
- **可指定瀏覽器**：可用 `--browser-path`、環境變數 `ASK_BRIDGE_BROWSER_PATH` 或設定檔 `browserPath` 改用 ego lite、Brave、Edge、Chromium 等 Chromium 系列瀏覽器，並依瀏覽器使用獨立 profile。
- **真實瀏覽器自動化**：直接控制監聽 `9223` port 的 Chrome debug profile。
- **持久登入狀態**：使用專屬本機 profile 目錄 `~/.config/ask-bridge/chrome-profile`，避免重複登入。
- **回覆輸出**：所選 provider 產生回覆時，將內容輸出到終端機。
- **思考動畫**：等待 provider 回覆時，在終端機顯示旋轉 spinner，開始輸出內容後自動清除。
- **ChatGPT 生圖進度**：讀取網頁上的圖片生成百分比，配合生成狀態與圖片載入狀態判斷完成；終端機與 `--verbose` 可顯示進度。
- **智慧分頁管理**：可重用既有 provider 分頁、聚焦分頁，或開啟新分頁，避免分頁過度增加。
- **Pipe 與 stdin 支援**：支援透過 standard input 傳入 prompt，例如 `cat report.txt | ask-bridge "summarize this"`。
- **圖片與文件上傳**：可透過 `--image` 附上圖片（支援 ChatGPT 與 Claude），或透過 `--file` 附上文件（PDF、Word、Excel、純文字、Markdown、JSON 等皆可），一次可指定多個檔案；Gemini 目前支援 `--file`，不支援 `--image` 圖片輸入。
- **模型與推理模式切換**：使用 `--model` 選模型，並以 `--reasoning` 分別控制 ChatGPT 推理強度或 Gemini 延伸思考。
- **接續既有對話**：使用 `--session`、`--session-id` 或 `--session-url` 指定既有對話，再從終端機送出新的 prompt。
- **臨時聊天**：使用 `--temporary` 建立不保留於一般聊天歷史的臨時對話（ChatGPT／Gemini 的 Temporary Chat、Claude 的 Incognito Chat）；送出前會先確認已進入臨時模式，無法確認時直接中止、不送出問題。
- **批次執行**：使用 `ask-bridge batch` 以 JSONL 一次提交多個工作，每個工作使用獨立分頁與獨立輸出目錄，互不串線；結束時輸出各終態的統計摘要。
- **SIWC API 後端（選用）**：ChatGPT 可改走 Sign in with ChatGPT 官方 OAuth 流程，以訂閱方案額度呼叫公開 Responses API，不需 API Key；OAuth 登入使用系統瀏覽器，後續提問不啟動 Chrome。詳見[第 16 節快速上手](#16-使用-chatgpt-訂閱額度-api-後端siwc)。
- **回應超時**：使用 `--timeout <秒數>` 設定等待回應上限，預設為 `300` 秒。
- **預設安靜模式與 verbose 模式**：預設只輸出最終回覆；加上 `--verbose` 可顯示背景瀏覽器控制流程。
- **版本資訊**：使用 `-v` 或 `--version` 顯示目前版本號。

## 前置需求

使用預設的瀏覽器後端需要以下環境；僅使用 SIWC 的差異見[第 16 節](#16-使用-chatgpt-訂閱額度-api-後端siwc)：

1. 已安裝 **Node.js 20.19.0 LTS 以上，或更新的 LTS 版本**，並確保 `node` 與 `npx` 可在目前 shell 的 `PATH` 中執行。`ask-bridge` 會透過 `npx` 啟動 `chrome-devtools-mcp@latest`；若 Node.js 版本過舊，例如 `v20.11.0`，MCP server 會在 `initialize` 階段直接退出。
2. 已安裝 Google Chrome。macOS 預設路徑通常是 `/Applications/Google Chrome.app`。若缺少 Chrome，且系統有 Homebrew，`make install` 會自動安裝。

可用以下命令確認目前 shell 看到的 Node.js 版本：

```bash
node -v
npx -v
```

| 平台 | 注意事項 |
| --- | --- |
| macOS | 可使用 Homebrew 或 nvm 安裝 Node.js LTS。若使用 nvm，請確認執行 `ask-bridge` 的同一個 shell 已載入 nvm，且 `node -v` 顯示 `v20.19.0` 以上。Chrome 預設偵測 `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`。 |
| Windows | 可使用 Node.js 官方安裝程式、winget 或 nvm-windows 安裝 Node.js LTS。安裝後請重新開啟 PowerShell，並確認 `node -v` 與 `npx -v` 可執行。Chrome 會優先偵測 `Program Files`、`Program Files (x86)` 與 `%LOCALAPPDATA%` 底下的標準安裝路徑。 |
| Linux | 許多發行版內建套件庫可能提供較舊的 Node.js；建議使用 NodeSource、nvm 或官方 Node.js LTS 來源安裝。請安裝 Google Chrome Stable，並確認 `google-chrome` 或 `google-chrome-stable` 位於 `PATH` 中；Snap、Flatpak 或 Chromium 安裝方式可能不符合預設偵測邏輯。 |

不需要安裝全域 `mcp-cli` 執行檔。Rust binary 會透過 Cargo 從 `https://github.com/doggy8088/mcp-cli` 使用 `mcp-cli` 作為 dependency。

## 安裝與建置

### 1. 快速安裝 (推薦)

若您只想使用已編譯好的 Release 版本（不需要安裝 Rust 工具鏈），可直接使用以下一鍵安裝腳本。安裝腳本會自動檢查 Node.js 需求、下載適合您系統架構的 ask-bridge binary，並將其放入 `~/.local/bin/` 目錄中。

#### macOS / Linux
請開啟終端機執行：
```bash
curl -fsSL https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.sh | bash
```

#### Windows
請開啟 PowerShell (建議以系統管理員身分) 執行：
```powershell
irm https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.ps1 | iex
```

> [!NOTE]
> 請確保安裝路徑（macOS/Linux 為 `~/.local/bin`；Windows 為 `$HOME\.local\bin`）已加入您的系統 `PATH` 環境變數中。
> 正式 CLI 命令為 `ask-bridge`；安裝流程也會提供 `ask` 作為向後相容 alias。以下範例皆以 `ask-bridge` 為準。
> Windows 安裝程式會驗證下載檔為 PE 執行檔、執行 `--version` smoke test，
> 並把正式安裝目錄放在 User PATH 最前面。可用 `where.exe ask-bridge`
> 確認第一筆展開後的完整路徑結尾為 `\.local\bin\ask-bridge.exe`。

### 2. 從原始碼建置與安裝 (適用於開發者)

若您想從原始碼編譯並安裝，請在複製本專案後，在專案目錄下執行：

```bash
make install
```

此命令會自動檢查 Node.js 環境、檢查與安裝 Chrome 瀏覽器、建置最佳化的 binary，並在 `~/.local/bin/ask-bridge` 建立正式命令符號連結 (symlink)，同時建立 `ask` alias。

### 3. 只建置不安裝

若只想建置 binary：

```bash
cargo build --release
```

編譯後的 binary 會位於 `target/release/ask-bridge`。

### 4. 安裝 Agent Skill

本專案提供 `ask-bridge` Agent Skill，讓支援 Skills 的 Coding Agent 可以在適合的情境下，自主使用 `ask-bridge` 將探索性研究、摘要、文件分析或方案比較等工作委派給 ChatGPT、Gemini 或 Claude 網站。

請使用 `npx skills` 安裝，不需要手動複製 `skills/` 目錄：

```bash
npx skills add doggy8088/ask-bridge --skill ask-bridge
```

若要安裝到 Codex 的全域 Skills 目錄，可指定 agent 與 global scope：

```bash
npx skills add doggy8088/ask-bridge --skill ask-bridge --agent codex --global
```

## 使用方式

### 1. 首次設定：登入 provider

以下為瀏覽器後端的登入方式；SIWC 請改依[第 16 節](#16-使用-chatgpt-訂閱額度-api-後端siwc)完成 OAuth 授權。送出 prompt 前，需要先登入所選 provider。未設定全域 provider 時，ChatGPT 為預設：

```bash
ask-bridge login
```

若要登入 Gemini 或 Claude：

```bash
ask-bridge --provider gemini login
ask-bridge --provider claude login
```

此命令會：

- 使用專屬且持久化的 debug profile 啟動 Google Chrome。
- 開啟所選 provider 頁面，例如 `https://chatgpt.com/`、`https://gemini.google.com/app` 或 `https://claude.ai/new`。
- 等待你手動登入帳號。
- 本工具會每秒自動偵測登入狀態，不需要你回到終端機按 Enter；若超過 `--timeout`（預設 300 秒）仍未偵測到登入完成，會提醒你再確認一次。

此流程通常只需要執行一次。

#### 全域 provider 設定

若希望未指定 `--provider` 時預設使用 Gemini 或 Claude，可用 `ask-bridge config` 指定：

```bash
ask-bridge config --provider gemini
ask-bridge config --provider claude
```

若要改回 ChatGPT：

```bash
ask-bridge config --provider chatgpt
```

可檢視目前設定：

```bash
ask-bridge config
```

`--provider` 的優先權高於全域設定檔，因此以下命令會暫時使用 ChatGPT：

```bash
ask-bridge --provider chatgpt "請摘要這段內容。"
```

#### 指定瀏覽器（Chromium 系列）

預設自動偵測 Google Chrome。若要改用其他 Chromium 系列瀏覽器（例如 ego lite、Brave、Microsoft Edge、Chromium），可用 `ask-bridge config` 持久設定，或用環境變數／CLI 參數覆寫。優先序為：CLI 參數 > 環境變數 > 設定檔 > Chrome 自動偵測。

```bash
# 持久設定（寫入 ~/.config/ask-bridge/config.json 的 browserPath）
ask-bridge config --browser-path "/Applications/ego lite.app/Contents/MacOS/ego lite"

# 一次性覆寫
ask-bridge --browser-path "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" "請摘要這段內容。"
ASK_BRIDGE_BROWSER_PATH="/usr/bin/chromium" ask-bridge "請摘要這段內容。"

# 清除設定（傳入空字串）
ask-bridge config --browser-path ""
```

自訂瀏覽器預設使用獨立的 profile 目錄 `~/.config/ask-bridge/<瀏覽器名稱>-profile`（例如 `ego-lite-profile`），避免不同瀏覽器共用同一個 profile；也可用 `--browser-profile-dir`、`ASK_BRIDGE_BROWSER_PROFILE_DIR` 或設定檔的 `browserProfileDir` 明確指定。設定的瀏覽器路徑不存在時會直接報錯，不會自動退回 Google Chrome。更換瀏覽器後，請先用該瀏覽器執行一次 `ask-bridge login` 完成登入。

### 2. 直接提問

將 prompt 作為 argument 傳入：

```bash
ask-bridge "Rust struct 和 tuple 有什麼差異？"
ask-bridge --provider gemini "Rust struct 和 tuple 有什麼差異？"
ask-bridge --provider claude "Rust struct 和 tuple 有什麼差異？"
```

執行後：

- Chrome 會開啟或聚焦所選 provider 分頁。
- Prompt 會自動輸入並送出。
- 所選 provider 的回覆會輸出到終端機。

### 3. 開啟全新對話

預設情況下，`ask-bridge` 會重用既有所選 provider 分頁，以避免建立過多分頁。

若要開啟全新的 provider 對話，使用 `--new`：

```bash
ask-bridge "誰是保哥？" --new
```

此模式只會開啟並使用新的所選 provider 分頁。執行前已存在的 provider
分頁與其他網站分頁都會保留；若無法唯一辨識新分頁，命令會停止且不會改用
或關閉既有分頁。

#### 臨時聊天（`--temporary`）

使用 `--temporary` 可建立不保留於一般聊天歷史的臨時對話，適合一次性或
較敏感的內容：

```bash
ask-bridge --provider chatgpt --temporary "請分析這份資料"
ask-bridge --provider gemini --temporary "請分析這份資料"
ask-bridge --provider claude --temporary "請分析這份資料"
```

`--temporary` 自動包含 `--new` 的效果（每次都開新分頁），且不可與
`--session`／`--session-id`／`--session-url` 同時使用；模型切換、推理模式、
附件與 `-o`／`-i` 輸出等既有功能維持不變。

三個 provider 進入臨時模式的方式不同：ChatGPT 透過
`?temporary-chat=true` 參數開啟並以頁面標示驗證；Gemini 需點按頁面的
「Temporary chat」（僅個人 Google 帳號提供）；Claude 需點按右上角的
Incognito（鬼魂圖示）按鈕。送出問題前，ask-bridge 會確認頁面已進入臨時
模式；無法確認時會直接中止並且不送出問題，避免內容被誤存入一般聊天
歷史。請注意，臨時模式由各 provider 定義：對話仍可能依其政策在伺服器端
短暫保留（例如 Gemini 最長 72 小時、Claude Incognito 約 30 天），並非
即時刪除。

### 4. 接續既有對話

使用 provider 對話 ID 或完整對話 URL，可接續網頁端既有的對話脈絡：

```bash
ask-bridge --provider chatgpt --session-id "conversation-uuid" "請接續先前的規劃。"
ask-bridge --session-url "https://chatgpt.com/c/conversation-uuid" "請產出下一步計畫。"
ask-bridge --provider gemini --session "conversation-id" "請繼續分析。"
```

`--session`、`--session-id` 與 `--session-url` 是相同參數的別名。傳入 ID
時會依 `--provider` 或全域設定組成 provider 對話 URL；傳入完整 URL 時會辨識
provider。若同時明確指定不相符的 `--provider`、URL 不屬於支援的 provider，
或與 `--new` 同時使用，命令會在開啟瀏覽器前停止。既有頁籤不會被關閉。

### 5. Headless 模式

一般提問預設使用 headless Chrome，也就是 `--headless=true`。Chrome 會在背景執行，不會搶走焦點或跳出視窗。

若想觀察 Chrome 的操作過程，或需要手動檢查頁面狀態，可改用 headful 模式：

```bash
ask-bridge "誰是保哥？" --headless=false
```

`ask-bridge login` 會強制使用 headful 模式，方便你和瀏覽器 UI 互動；其他 subcommand 若要可見瀏覽器，請明確加上 `--headless=false`。

### 6. Verbose 模式

預設情況下，`ask-bridge` 只輸出所選 provider 的最終回覆，隱藏背景控制訊息。

若要查看完整瀏覽器自動化流程，加入 `--verbose`：

```bash
ask-bridge "誰是保哥？" --verbose
```

Verbose 模式會顯示類似以下流程：

- 檢查已開啟的 Chrome 分頁。
- 聚焦輸入欄位。
- 輸入 prompt。
- 送出訊息。
- 等待 provider 回覆。

### 7. 回應超時（`--timeout`）

回答等待預設 300 秒，若 provider 回應時間較長可提高（或降低）等待上限：

```bash
ask-bridge "請幫我整理這份報告" --timeout 600
```

你也可以明確設定較短時間：

```bash
ask-bridge "簡短回覆" --timeout 60
```

超過設定秒數仍未完成時，會輸出超時警告並直接結束回應等待流程。

### 8. 顯示版本

使用 `-v` 或 `--version` 顯示目前版本號：

```bash
ask-bridge -v
```

### 9. Pipe 與 stdin

可透過 pipe 將文字或檔案內容傳入 `ask-bridge`：

```bash
echo "用一句話解釋 quantum computing" | ask-bridge
```

若同時提供 prompt argument，`ask-bridge` 會先送出該 prompt，接著加上兩個換行後再附加管道內容：

```bash
cat /Users/will/.copilot/session-state/46cc0f1c-79fd-4622-9548-a0b7fa3794be/research/does-cursor-support-byok.md | ask-bridge 'What is this?'
```

也可以讀取檔案內容：

```bash
cat src/main.rs | ask-bridge "這段 Rust code 有記憶體洩漏風險嗎？"
```

### 10. 附上圖片或文件

除了把檔案內容透過 pipe 傳入 prompt，你也可以直接把本機檔案當作附件上傳給所選 provider。

#### 附上圖片

使用 `--image` 附上一或多張本機圖片（可重複指定）。此功能目前支援 ChatGPT 與 Claude；Gemini 圖片輸入尚未支援，搭配 `--provider gemini` 使用會立即回報錯誤。

```bash
ask-bridge "請描述這張圖片的內容。" --image screenshot.png
ask-bridge "比較這兩張圖的差異。" --image v1.png --image v2.png
ask-bridge --provider claude "請描述這張圖片的內容。" --image screenshot.png
```

支援的格式包含 PNG、JPEG、GIF、WebP、SVG、BMP 等。

#### 附上文件

使用 `--file` 附上一或多份本機文件（可重複指定），例如 PDF、Word、Excel、PowerPoint、純文字、Markdown、CSV、JSON、程式碼等。ChatGPT、Gemini 與 Claude 都支援此流程。

```bash
ask-bridge "請摘要這份 PDF 的重點。" --file report.pdf
ask-bridge "這份 CSV 總共有幾筆資料？" --file data.csv
ask-bridge "幫我檢查這段程式碼有沒有問題。" --file src/main.rs
```

也可以同時附上圖片與文件：

```bash
ask-bridge "請對照這張設計圖與規格文件，指出不一致的地方。" --image design.png --file spec.docx
```

#### 下載生成圖片

provider 回覆後，可使用 `-i` / `--image-output` 指定生成圖片的下載路徑（資料夾或檔案路徑）。

```bash
ask-bridge --provider chatgpt "請產生一張茶壺插畫" --image-output /tmp/teapot.png --verbose
```

ChatGPT 顯示圖片生成百分比時，工具會持續讀取進度，避免將生成中的預覽圖當成完成結果。百分比達到 100% 或進度欄位消失後，仍會確認生成已結束且圖片載入完成，再下載圖片；若網頁沒有百分比，則沿用生成狀態與圖片就緒檢查。等待仍受 `--timeout` 限制。

回覆中的引用來源 favicon 與來源卡片不會被當成生成圖片，純文字回覆的圖片掃描會立即結束。未指定 `-i` 時，圖片下載失敗只會顯示警告，不影響回覆輸出與 `-o` 檔案寫入；指定 `-i` 時，圖片下載失敗仍會以非零狀態結束。

若已偵測到本次 ChatGPT 回覆的生圖進度或候選圖片，但回覆未能在期限內完成，工具會以非零狀態結束，且不會以空內容覆寫 `--output` 檔案；不會因為顯示 100% 就略過圖片就緒檢查。

ChatGPT 的多張生成圖片必須全部取得才會回報下載成功；部分圖片無法取得時會重試至逾時，而不會將部分成果誤報為完整下載。此規則僅適用於 ChatGPT，不變更 Gemini 與 Claude 的既有下載行為。

### 11. 切換模型與推理模式

使用 `--model` 在送出 prompt 前切換 provider 模型；使用 `--reasoning` 分別指定 provider 支援的推理模式。兩者可在同一次 ChatGPT 呼叫中併用；Gemini 的延伸思考只相容於 Pro 模型。

```bash
ask-bridge "證明這個數學問題。" --model "GPT-5.6 Sol" --reasoning high
ask-bridge "快速翻譯這段話。" --reasoning instant
ask-bridge --provider gemini "用幾句話介紹 Rust。" --model "3.6 Flash"
ask-bridge --provider gemini "證明這個數學問題。" --model "3.1 Pro" --reasoning extended
ask-bridge --provider claude "用幾句話介紹 Rust。" --model Sonnet
```

參數規則：

- **ChatGPT**：`--reasoning` 支援 `instant`、`medium`、`high`、`xhigh`，也接受 `即時`、`中`、`中等`、`高`、`極高` 等對應別名。新版 ChatGPT 的推理強度滑桿已移除「自動」，`auto` 只保留給仍提供該選項的介面。ask-bridge 一律使用 ChatGPT 的「對話（Chat）」模式；若撰寫器停在「工作（Work）」模式，會在送出前自動切回。
- **Gemini**：`--reasoning extended` 選擇 Extended Thinking；可省略 `--model`，或搭配實際存在的 Pro 模型。
- **Claude**：不支援 `--reasoning`；`--model` 的 Sonnet、Opus、Haiku 選擇流程維持不變。

模型比對只使用選單的主標籤，忽略副標題與 badge；比對仍不分大小寫與標點。工具不會把舊版模型名稱自動改選為其他版本。若主標籤不存在，錯誤會列出目前讀到的 provider 選項，並在送出 prompt 前中止。

舊用法如 `--model 高` 或 Gemini 的 `--model 延伸思考` 暫時仍可使用，但會顯示棄用警告；請改用 `--reasoning`。

### 12. 只開啟 provider

若只想快速開啟瀏覽器並進入所選 provider：

```bash
ask-bridge open
ask-bridge --provider gemini open
ask-bridge --provider claude open
```

### 13. 關閉瀏覽器 instance

若要關閉 `ask-bridge` 管理的 Chrome debug profile instance：

```bash
ask-bridge close
```

`close` 只會關閉使用 `~/.config/ask-bridge/chrome-profile` 且監聽 debug port `9223` 的 `ask-bridge` Chrome instance；若該 port 被非 `ask-bridge` Chrome 程序占用，會回報錯誤而不會關閉它。

### 14. 更新 ask-bridge

若要直接自動更新目前安裝的 `ask-bridge`，可直接執行：

```bash
ask-bridge update
```

此命令會依作業系統重新執行 README 建議的官方安裝命令（macOS / Linux 或 Windows）。

### 15. 批次執行多個工作（`batch`）

使用 `ask-bridge batch` 可一次提交多個彼此獨立的工作（issue #24）。每個
工作會在自己的瀏覽器分頁中執行，輸出到獨立目錄，互不影響：

```bash
ask-bridge batch --input jobs.jsonl --output-dir results/
cat jobs.jsonl | ask-bridge batch --output-dir results/
```

`jobs.jsonl` 為 JSONL 格式，一行一個工作物件：

```json
{"id":"job-1","provider":"chatgpt","prompt":"請只回覆 JOB-1"}
{"id":"job-2","provider":"gemini","prompt":"請摘要這份文件","files":["report.pdf"],"timeout":600}
```

欄位說明：`id`（可省略，自動產生；限 `[A-Za-z0-9._-]`、不可重複）、
`provider`（可省略，沿用全域設定）、`prompt`（必填）、`timeout`（秒，
預設 300）、`session`（接續既有對話）、`temporary`（臨時聊天，不可與
`session` 併用）、`model`／`reasoning`、`files`／`images` 附件、
`downloadImages`（預設 `true`）。每個工作固定在自己的新分頁執行，
`"new": false` 不被接受。

每個工作的產出位於 `<output-dir>/<job-id>/`，包含 `response.md`、
`metadata.json`（provider、分頁 ID、對話連結、終態與錯誤）與 `images/`
（下載的生成圖片）；檔案以暫存檔寫入後原子改名，不會留下半成品。執行
結束會輸出摘要（Total／Completed／Failed／Timed out／Recoverable／
Cancelled）；任一工作未完成時結束代碼為 1。加上 `--fail-fast` 可在第一
個未完成的工作後停止啟動後續工作（標記為 Cancelled）。

若工作在送出 prompt 時遇到瀏覽器連線中斷，送出結果不明確，該工作會被
標記為 `Recoverable` 並保留對話連結供人工恢復，**不會自動重送**，避免
重複提問。

`--concurrency N`（預設 `1`）控制同時執行的工作上限：各工作的瀏覽器
控制操作（選取分頁、送出、擷取）會序列化執行，但 provider 的回覆生成
在各自分頁中平行進行。結束摘要會另行回報實際觀察到的最大同時執行數
（max in-flight），不會超過設定的 N。帳號本身的並行上限由各 provider
決定；若遇到 provider 限流，請調低 `--concurrency`，工具不會因限流而
自動重送 prompt。工作送出後會先在自己的目錄留下 `in_progress` 狀態的
`metadata.json`（含分頁 ID 與對話連結），即使程序中途被結束，也能依
該檔案人工恢復已送出的工作。

兩個以上工作指向同一個既有對話（`session`）時，各工作雖有自己的分頁，
操作的仍是同一個遠端對話、回覆無法隔離，因此 `--concurrency` 大於 1
時會在預檢階段直接拒絕重複的對話目標；`--concurrency 1` 循序執行則
依輸入順序進行。工作結束後其分頁會自動關閉，僅 `Recoverable`（送出
結果不明確）的工作保留分頁供人工復原，保留數量以不明確送出的次數為
上限。

* * *

### 16. 使用 ChatGPT 訂閱額度 API 後端（SIWC）

**SIWC 是選用後端；未設定時仍使用原本的 Browser/CDP。** Sign in with ChatGPT 讓你透過 OAuth 授權，以 ChatGPT 方案額度呼叫公開 Responses API，不需要 API Key。適合單次文字提問、摘要與方案比較；需要網頁既有對話、附件或生圖時，請選擇瀏覽器後端。

#### 先理解 Provider、Backend 與額度

| 項目 | 說明 |
| --- | --- |
| `--provider chatgpt\|gemini\|claude` | 選擇 AI 服務。SIWC 目前僅適用 ChatGPT。 |
| `--backend browser\|siwc` | 選擇這次執行的傳輸方式，優先於設定檔。 |
| `config --chatgpt-backend siwc` | 儲存 ChatGPT 的預設後端；`--chatgpt-backend` 不是 `--backend` 的別名，僅用於 `config`。 |
| 額度來源 | 使用 ChatGPT Work／Codex 方案額度或 credits，不使用 API Key 計費來源。 |
| 登入狀態 | SIWC OAuth 憑證與瀏覽器 cookies 分開保存，網站已登入不代表 SIWC 已授權。 |

**SIWC 不是免費或無限 token，也不保證減少總 token 或費用。** 若你的主要 Coding Agent 與 SIWC 共用方案額度，兩者可能競爭同一份額度。可用模型、授權資格、額度與 credits 依帳號及 workspace 實際政策為準，不以方案名稱保證可用。

#### 快速上手：登入後送出第一個問題

先依前面的安裝章節安裝 `ask-bridge`，再執行：

```bash
# 1. 開啟系統瀏覽器，登入並授權 ChatGPT 方案使用
ask-bridge --provider chatgpt --backend siwc login

# 2. 授權完成、終端機確認登入成功後提問
ask-bridge --provider chatgpt --backend siwc "用三個重點解釋這個工具適合的使用情境"
```

首次登入會進行動態註冊；完成 OAuth 需要系統瀏覽器，後續提問直接透過 HTTP 串流，**不啟動 Chrome、不使用 CDP 或 MCP**。SIWC 提問本身不需要 Node.js／npx；目前安裝腳本仍有瀏覽器模式的相依檢查，請依安裝章節操作。

登入時請使用具方案使用權的帳號，完成頁面上的方案使用授權，再等候瀏覽器導回本機 callback。登入成功會嘗試列出最多 10 個可用模型及其 slug；若系統瀏覽器沒有自動開啟，可使用終端機顯示的授權網址。登入 callback 固定最多等待 300 秒，不受提問用的 `--timeout` 調整。

#### 提問、stdin、模型與輸出檔

```bash
# 把純文字文件內容作為上下文
cat report.md | ask-bridge --provider chatgpt --backend siwc "摘要重點，列出待確認事項"

# 也可只從 stdin 讀取完整問題
ask-bridge --provider chatgpt --backend siwc < prompt.txt

# 設定整體請求逾時，成功後將完整回覆寫入檔案
ask-bridge --provider chatgpt --backend siwc --timeout 120 -o answer.md "比較兩個技術方案"

# 將 MODEL_SLUG 換成登入時列出的實際 slug
ask-bridge --provider chatgpt --backend siwc --model "MODEL_SLUG" "比較兩個技術方案"
```

| 參數／輸入 | SIWC 行為 |
| --- | --- |
| prompt 加 stdin | 合併為「prompt、兩個換行、stdin」，一起送出；不會自動讀取其他檔案。 |
| `--model` | 每次向帳號模型目錄驗證；接受 slug 或完整顯示名稱，名稱比對不分英文大小寫。省略時使用目錄第一個模型，與網頁選單無關。不存在時列出可用 slug 並中止，不會改用其他模型。 |
| `--timeout` | 預設 300 秒，涵蓋 token 更新、模型目錄與推論串流；前置 HTTP 請求另有 30 秒上限，因此不保證一定等到整體上限。 |
| `-o`／`--output` | 回覆仍即時輸出到終端機；只有收到成功完成事件 `response.completed` 後，才寫入指定檔案。成功時會覆寫既有檔案，父目錄需先存在。 |

**終端機已印出部分文字不代表成功。** 額度錯誤、中斷或逾時可能在串流途中發生；這時程序以非零代碼結束，不執行 `-o` 寫檔，既有輸出檔也不會因本次推論失敗而被覆寫。直接以 `>` 重新導向 stdout 則可能留下不完整文字；需要確認完成的檔案時，請使用 `-o` 並檢查結束代碼。

#### 如何把目前工作的上下文傳入？

**把相關文字透過 stdin 傳入，再以 prompt 說明任務。** SIWC 不會自動取得目前工作目錄、Git 狀態、IDE 選取內容或 Coding Agent 的對話紀錄。你提供什麼，它才會收到什麼；`--file` 附件與將文字內容 pipe 進 stdin 是不同功能。

以下為 Bash／zsh 範例。多份資料請加上檔名或段落標籤，讓模型能分辨來源：

```bash
# 比較兩份文件，明確標示來源
{
  printf '\n--- a.md ---\n'
  cat a.md
  printf '\n--- b.md ---\n'
  cat b.md
} | ask-bridge --provider chatgpt --backend siwc "比較兩份文件的差異與矛盾"

# 把尚未 staged 的程式碼差異交給模型檢查
# 已 staged 的變更請改用 git diff --cached
# git diff 不包含尚未追蹤的檔案
git diff -- src/main.rs | ask-bridge --provider chatgpt --backend siwc "檢查這份 diff 的邏輯風險"

# 先保存測試輸出，再提供給模型分析
cargo test > test-output.txt 2>&1
cat test-output.txt | ask-bridge --provider chatgpt --backend siwc "分析失敗原因並建議修正方向"
```

Windows PowerShell 可用 `Get-Content -Raw` 傳入文字內容：

```powershell
Get-Content -Raw -Encoding utf8 report.md | ask-bridge --provider chatgpt --backend siwc "摘要重點"
```

目前每次呼叫都是獨立提問，工具不會自動保存或重送前一輪上下文，也不能用 `--session` 接續網頁對話。需要延續討論時，請自行選取上一輪的問題、回覆與必要背景，整理後再次傳入。例如：

```bash
# 第一輪成功後，才將新產生的摘要與原文傳入下一輪
if cat report.md | ask-bridge --provider chatgpt --backend siwc -o summary.md "摘要報告並列出風險"; then
  {
    printf '\n--- 上一輪問題 ---\n摘要報告並列出風險\n'
    printf '\n--- 上一輪回覆 ---\n'
    cat summary.md
    printf '\n--- 相關原文 ---\n'
    cat report.md
  } | ask-bridge --provider chatgpt --backend siwc -o follow-up.md "根據提供的上一輪摘要與原文，提出三項改善措施"
fi
```

這是手動提供文字背景，不是內建多輪會話。只傳入與任務相關的片段，並刪除密碼、token、個資等不應送出的內容；反覆附上全部歷史會增加輸入 token、耗時與額度使用，也可能超過模型上下文上限。PDF、Word 或圖片等二進位檔案不能直接當文字 pipe；請先轉成文字，或改用支援附件的瀏覽器模式。

#### 設定預設後端與切回瀏覽器

```bash
# 僅改 ChatGPT 的預設後端，保留其他設定
ask-bridge config --chatgpt-backend siwc
ask-bridge config

# 設定後可省略 --backend，但仍明確指定 ChatGPT
ask-bridge --provider chatgpt "摘要這項設計的風險"

# 本次改走瀏覽器，不改持久設定；若網站尚未登入，先執行下一行
ask-bridge --provider chatgpt --backend browser login
ask-bridge --provider chatgpt --backend browser "使用網頁功能回答這個問題"

# 恢復 ChatGPT 的預設瀏覽器後端
ask-bridge config --chatgpt-backend browser
```

後端優先序為 `--backend` > `config.json` 的 `chatgptBackend` > `browser`。設定 SIWC 不會改變預設 provider；若預設 provider 是 Gemini 或 Claude，仍需用 `--provider chatgpt` 才會使用 SIWC。Gemini／Claude 會忽略持久化的 ChatGPT 後端設定，但明確搭配 `--backend siwc` 會報錯。

#### Browser 與 SIWC 功能矩陣

| 功能 | Browser 後端 | SIWC 後端 |
| --- | --- | --- |
| 服務 | ChatGPT、Gemini、Claude | 僅 ChatGPT |
| 登入與連線 | 網站登入、Chrome／CDP／MCP | OAuth；提問使用 HTTP／SSE |
| 單次文字、stdin、`-o` | 支援 | 支援 |
| 模型選擇 | 網頁模型選單 | 帳號 API 模型目錄 |
| 既有對話 `--session`／`--session-id`／`--session-url` | 支援 | 拒絕，不讀取 `chatgpt.com/c/...` |
| `--new`／`--temporary` | 支援 | 拒絕；每次提問獨立，無網頁臨時聊天模式 |
| `--image`／`--file` | 依 provider 支援 | 本工具尚未實作，會拒絕 |
| 生圖與 `--image-output`／`-i` | 依網站功能支援 | 不支援，會拒絕圖片輸出選項 |
| `--reasoning` | 依 provider 支援 | 尚未實作，會拒絕 |
| `batch` | 支援 | 拒絕；若已設 SIWC 預設值，需明確使用 `--backend browser` |
| `open`／`get`／`close` 等瀏覽器指令 | 支援 | 拒絕，需明確使用 `--backend browser` |
| 額度來源 | 各網站產品與帳號限制 | ChatGPT Work／Codex 方案額度或 credits |

目前沒有 `--download-images` CLI 參數；`downloadImages` 是瀏覽器批次工作 JSON 的欄位。SIWC 不支援 `batch`，不能以該欄位啟用下載。

本工具的 SIWC MVP 僅提供單次純文字推論，不提供 file search、Code Interpreter、圖片生成或 hosted MCP/connectors 等工具。附件未支援是本工具目前的限制，不代表上游 SIWC API 一律不接受圖片或文件。這條 HTTP 路徑使用 `store: false`、`stream: true`，不使用 `previous_response_id` 接續會話；`store: false` 不代表服務端沒有任何資料保留。模型與 API 能力仍以 [OpenAI 模型與推論文件](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference)及 [Preview 限制](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations)為準。

#### 重新登入、登出與憑證

```bash
# 授權過期、被撤銷或更新失敗時，重新執行登入
ask-bridge --provider chatgpt --backend siwc login

# 清除本機保存的 SIWC 帳號與 token
ask-bridge --provider chatgpt --backend siwc logout
```

憑證保存於 `~/.config/ask-bridge/siwc-credentials.json`，採原子寫入；Unix 檔案權限為 `0600`，Windows 不適用此 Unix 權限表示法。工具會自動更新即將過期的 access token 並保存輪替後的 refresh token，不應將憑證檔分享、提交到 Git 或貼到問題回報中。

`logout` 清除本機帳號與 token，但保留固定的 host ID，因此憑證檔可能仍存在；它不會呼叫伺服器端撤銷 API、不會登出 ChatGPT 網站，也不會將預設後端改回 `browser`。下次使用 SIWC 需重新登入。

#### 常見問題與失敗處理

| 情況 | 處理方式 |
| --- | --- |
| `not signed in`／401／refresh 失敗 | 執行上方完整的 SIWC `login` 指令；網站登入狀態無法代替 OAuth 授權。 |
| 拒絕授權／缺少方案使用權 | 確認登入的帳號、workspace 與方案使用授權，再重新登入；僅有訂閱名稱不保證權限。 |
| 模型不存在／模型目錄為空 | 使用錯誤訊息列出的 slug；目錄為空時，確認帳號權限與服務可用狀態。 |
| 429／`subscription_sharing_usage_limit_exceeded`／`subscription_sharing_usage_unavailable` | 檢查方案用量、credits 或帳號可用狀態，依錯誤恢復；錯誤可能在串流中途才出現。 |
| 逾時／網路中斷／未收到完成事件 | 本次視為失敗，已顯示文字可能不完整；確認連線後自行決定是否重新送出。 |
| 選項與 SIWC 不相容 | 移除不支援選項，或明確選擇 `--backend browser` 並完成網站登入。 |
| `UnknownIssuer` | 請管理員確認所需 CA 已安裝到系統信任庫；不要停用 TLS 憑證驗證。 |

**失敗時不會自動切回 Browser/CDP，也不會動用 API Key 額度。** HTTPS 使用 rustls，合併內建 CA 與作業系統信任庫；錯誤會顯示底層連線原因，不包含請求 URL、headers 或 body。在企業代理／防毒 HTTPS 掃描等 TLS 檢查環境中，受信任的攔截方可看到連線內容，包含 token 交換與更新；請只在信任該環境時使用。

#### 驗證範圍與參考

依 [issue #40 結案紀錄](https://github.com/doggy8088/ask-bridge/issues/40#issuecomment-6956849874)，截至 2026-11-17：

- 已完成 Windows 真實環境 OAuth 登入與單次文字對話驗證。
- 尚未逐項進行真實環境驗證：refresh token 輪替與失效重登、登出後的撤銷效果、額度耗盡錯誤呈現、網路中斷恢復。
- 尚無效能／額度成本基準數據，不能據此宣稱一定更快或更省 token。

以上是該 issue 結案時的驗證紀錄，不表示所有帳號、平台與錯誤情境都已驗證；目前指令以 `ask-bridge --help` 為準。

* * *

## 運作原理

以下說明 Browser/CDP 後端；SIWC 在完成 OAuth 授權後，直接取得帳號模型目錄並以 HTTP／SSE 呼叫 Responses API，略過下列瀏覽器流程。

1. **瀏覽器初始化**：`ask-bridge` 會檢查 Chrome 是否正在監聽 debug port `9223`。若沒有，會以專屬 profile 目錄 `~/.config/ask-bridge/chrome-profile` 啟動 Google Chrome。
2. **MCP Bridge 設定**：啟動時會自動寫入 `~/.config/ask-bridge/mcp_servers.json`，預設設定 Chrome DevTools MCP server，使用 `chrome-devtools-mcp@latest` 與 `--browser-url=http://127.0.0.1:9223`。
3. **Client 呼叫**：`ask-bridge` 透過內建的 `doggy8088/mcp-cli` Rust library dependency 呼叫 MCP tools，例如 `list_pages`、`select_page`、`type_text` 與 `evaluate_script`，不依賴系統上的外部 `mcp-cli` 命令。
4. **狀態輪詢**：provider 產生回覆期間，工具會以 JavaScript 檢查送出與停止按鈕狀態，擷取回覆元素的文字內容，並輸出到 `stdout`。

## 相關文件

- [快速開始](docs/quick-start.md)
- [背景 Chrome 隱形技術與實作原理](docs/headless-techniques.md)

## 授權

MIT License。可自由使用、修改與散布。
