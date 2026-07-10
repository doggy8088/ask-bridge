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

不同於一般 API client，`ask-bridge` 會在真實 Chrome 瀏覽器中執行，並使用持久化的專屬使用者 profile。這表示：

- 只需要手動登入一次，透過 `ask-bridge login` 完成。
- 可處理 CAPTCHA、Cloudflare，並像一般使用者一樣存取所選 provider 的網頁功能。
- Session cookies、登入狀態與對話紀錄會持久保存。

## 主要功能

- **100% Rust 核心**：快速、輕量，編譯後即可執行。
- **多 provider 支援**：使用 `--provider chatgpt|gemini|claude` 選擇 ChatGPT、Gemini 或 Claude。
- **全域 provider 設定**：可在 `~/.config/ask-bridge/config.json` 指定預設 provider，CLI 的 `--provider` 會覆蓋設定檔。
- **真實瀏覽器自動化**：直接控制監聽 `9223` port 的 Chrome debug profile。
- **持久登入狀態**：使用專屬本機 profile 目錄 `~/.config/ask-bridge/chrome-profile`，避免重複登入。
- **回覆輸出**：所選 provider 產生回覆時，將內容輸出到終端機。
- **思考動畫**：等待 provider 回覆時，在終端機顯示旋轉 spinner，開始輸出內容後自動清除。
- **智慧分頁管理**：可重用既有 provider 分頁、聚焦分頁，或開啟新分頁，避免分頁過度增加。
- **Pipe 與 stdin 支援**：支援透過 standard input 傳入 prompt，例如 `cat report.txt | ask-bridge "summarize this"`。
- **圖片與文件上傳**：可透過 `--image` 附上圖片（支援 ChatGPT 與 Claude），或透過 `--file` 附上文件（PDF、Word、Excel、純文字、Markdown、JSON 等皆可），一次可指定多個檔案；Gemini 目前支援 `--file`，不支援 `--image` 圖片輸入。
- **模型切換**：使用 `--model` 在送出 prompt 前自動切換 provider 模型（如 ChatGPT 的 `GPT-5.4`、`o3`，Gemini 的 `3.5 Flash`、`3.1 Pro`，或 Claude 的 `Sonnet`、`Opus`）。
- **預設安靜模式與 verbose 模式**：預設只輸出最終回覆；加上 `--verbose` 可顯示背景瀏覽器控制流程。
- **版本資訊**：使用 `-v` 或 `--version` 顯示目前版本號。

## 前置需求

執行此工具需要：

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

送出 prompt 前，需要先登入所選 provider。未設定全域 provider 時，ChatGPT 為預設：

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
- 在你回到終端機按 Enter 後，驗證登入狀態並保存 profile。

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

此模式會開啟新的所選 provider 分頁，並清理先前同一 provider 的分頁。

### 4. Headless 模式

一般提問預設使用 headless Chrome，也就是 `--headless=true`。Chrome 會在背景執行，不會搶走焦點或跳出視窗。

若想觀察 Chrome 的操作過程，或需要手動檢查頁面狀態，可改用 headful 模式：

```bash
ask-bridge "誰是保哥？" --headless=false
```

`ask-bridge login` 會強制使用 headful 模式，方便你和瀏覽器 UI 互動；其他 subcommand 若要可見瀏覽器，請明確加上 `--headless=false`。

### 5. Verbose 模式

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

### 6. 顯示版本

使用 `-v` 或 `--version` 顯示目前版本號：

```bash
ask-bridge -v
```

### 7. Pipe 與 stdin

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

### 8. 附上圖片或文件

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

#### 顯示上傳結果

provider 回覆後，可使用 `-i` / `--image-output` 指定生成圖片的下載路徑（資料夾或檔案路徑）。

### 9. 切換模型

使用 `--model` 在送出 prompt 前自動切換 provider 的模型。比對時不分大小寫與標點符號（`-`、`.`、空格 等）。

```bash
ask-bridge "用幾句話介紹 Rust。" --model GPT-5.4
ask-bridge "證明這個數學問題。" --model o3
ask-bridge "快速翻譯這段話。" --model 即時
ask-bridge --provider gemini "用幾句話介紹 Rust。" --model "3.5 Flash"
ask-bridge --provider gemini "用幾句話介紹 Rust。" --model "3.1 Pro"
ask-bridge --provider claude "用幾句話介紹 Rust。" --model Sonnet
ask-bridge --provider claude "證明這個數學問題。" --model Opus
```

可用的模型名稱（視帳號權限與 provider UI 而定）：

- **ChatGPT 模型**：`GPT-5.5`、`GPT-5.4`、`GPT-5.3`、`o3`
- **ChatGPT 思考強度**：`智慧`、`即時`、`中等`、`高`、`超高`、`專業`
- **Gemini 模式**：`3.5 Flash`、`3.1 Flash-Lite`、`3.1 Pro`
- **Claude 模型**：`Sonnet`、`Opus`、`Haiku`（實際名稱依 claude.ai 選單與帳號方案而定）

> 若指定的名稱在選單中找不到，`ask-bridge` 會回報 `Model switch failed: error: model not found in menu` 並中止，不會送出 prompt。

### 10. 只開啟 provider

若只想快速開啟瀏覽器並進入所選 provider：

```bash
ask-bridge open
ask-bridge --provider gemini open
ask-bridge --provider claude open
```

### 11. 關閉瀏覽器 instance

若要關閉 `ask-bridge` 管理的 Chrome debug profile instance：

```bash
ask-bridge close
```

`close` 只會關閉使用 `~/.config/ask-bridge/chrome-profile` 且監聽 debug port `9223` 的 `ask-bridge` Chrome instance；若該 port 被非 `ask-bridge` Chrome 程序占用，會回報錯誤而不會關閉它。

## 運作原理

1. **瀏覽器初始化**：`ask-bridge` 會檢查 Chrome 是否正在監聽 debug port `9223`。若沒有，會以專屬 profile 目錄 `~/.config/ask-bridge/chrome-profile` 啟動 Google Chrome。
2. **MCP Bridge 設定**：啟動時會自動寫入 `~/.config/ask-bridge/mcp_servers.json`，預設設定 Chrome DevTools MCP server，使用 `chrome-devtools-mcp@latest` 與 `--browser-url=http://127.0.0.1:9223`。
3. **Client 呼叫**：`ask-bridge` 透過內建的 `doggy8088/mcp-cli` Rust library dependency 呼叫 MCP tools，例如 `list_pages`、`select_page`、`type_text` 與 `evaluate_script`，不依賴系統上的外部 `mcp-cli` 命令。
4. **狀態輪詢**：provider 產生回覆期間，工具會以 JavaScript 檢查送出與停止按鈕狀態，擷取回覆元素的文字內容，並輸出到 `stdout`。

## 相關文件

- [快速開始](docs/quick-start.md)
- [背景 Chrome 隱形技術與實作原理](docs/headless-techniques.md)

## 授權

MIT License。可自由使用、修改與散布。
