# Ask ChatGPT Rust 版

`ask` 是以 Rust 撰寫的輕量命令列工具，可透過真實 Chrome 瀏覽器自動操作 ChatGPT。它使用 Model Context Protocol MCP 與 Chrome DevTools Protocol CDP，並透過內建的 `doggy8088/mcp-cli` Rust library dependency 搭配 `chrome-devtools-mcp` 控制 Chrome、輸入 prompt、送出訊息，並將回覆即時串流輸出到終端機。

不同於一般 API client，`ask` 會在真實 Chrome 瀏覽器中執行，並使用持久化的專屬使用者 profile。這表示：

- 只需要手動登入一次，透過 `ask login` 完成。
- 可處理 CAPTCHA、Cloudflare，並像一般使用者一樣存取 ChatGPT 方案功能與自訂 GPT。
- Session cookies、登入狀態與對話紀錄會持久保存。

## 主要功能

- **100% Rust 核心**：快速、輕量，編譯後即可執行。
- **真實瀏覽器自動化**：直接控制監聽 `9223` port 的 Chrome debug profile。
- **持久登入狀態**：使用專屬本機 profile 目錄 `~/.config/ask-chatgpt/chrome-profile`，避免重複登入。
- **即時串流輸出**：ChatGPT 產生回覆時，逐字將內容輸出到終端機。
- **思考動畫**：等待 ChatGPT 回覆時，在終端機顯示旋轉 spinner，開始輸出內容後自動清除。
- **智慧分頁管理**：可重用既有 ChatGPT 分頁、聚焦分頁，或開啟新分頁，避免分頁過度增加。
- **Pipe 與 stdin 支援**：支援透過 standard input 傳入 prompt，例如 `cat report.txt | ask "summarize this"`。
- **預設安靜模式與 verbose 模式**：預設只輸出最終回覆；加上 `--verbose` 可顯示背景瀏覽器控制流程。
- **版本資訊**：使用 `-v` 或 `--version` 顯示目前版本號。

## 前置需求

執行此工具需要：

1. 已安裝 `node` 與 `npx`，用於自動啟動 `chrome-devtools-mcp` server。
2. 已安裝 Google Chrome。macOS 預設路徑通常是 `/Applications/Google Chrome.app`。若缺少 Chrome，且系統有 Homebrew，`make install` 會自動安裝。

不需要安裝全域 `mcp-cli` 執行檔。Rust binary 會透過 Cargo 從 `https://github.com/doggy8088/mcp-cli` 使用 `mcp-cli` 作為 dependency。

## 安裝與建置

### 1. 安裝工具

在專案目錄執行：

```bash
make install
```

此命令會安裝必要的 Chrome 瀏覽器、建置最佳化 binary，並建立 `~/.local/bin/ask` symlink。

請確認 `~/.local/bin` 已加入 shell 的 `PATH`。

### 2. 只建置不安裝

若只想建置 binary：

```bash
cargo build --release
```

編譯後的 binary 會位於 `target/release/ask-chatgpt`。

## 使用方式

### 1. 首次設定：登入 ChatGPT

送出 prompt 前，需要先登入 ChatGPT 帳號。執行：

```bash
ask login
```

此命令會：

- 使用專屬且持久化的 debug profile 啟動 Google Chrome。
- 開啟 `https://chatgpt.com/`。
- 等待你手動登入帳號。
- 在你回到終端機按 Enter 後，驗證登入狀態並保存 profile。

此流程通常只需要執行一次。

### 2. 直接提問

將 prompt 作為 argument 傳入：

```bash
ask "Rust struct 和 tuple 有什麼差異？"
```

執行後：

- Chrome 會開啟或聚焦 ChatGPT 分頁。
- Prompt 會自動輸入並送出。
- ChatGPT 回覆會即時串流輸出到終端機。

### 3. 開啟全新對話

預設情況下，`ask` 會重用既有 ChatGPT 分頁，以避免建立過多分頁。

若要開啟全新的 ChatGPT 對話，使用 `--new`：

```bash
ask "誰是保哥？" --new
```

此模式會開啟新的 ChatGPT 分頁，並清理先前的 ChatGPT 分頁。

### 4. Headless 模式

一般提問預設使用 headless Chrome，也就是 `--headless=true`。Chrome 會在背景執行，不會搶走焦點或跳出視窗。

若想觀察 Chrome 的操作過程，或需要手動檢查頁面狀態，可改用 headful 模式：

```bash
ask "誰是保哥？" --headless=false
```

`ask login` 與 `ask open` 這類 subcommand 會強制使用 headful 模式，方便你和瀏覽器 UI 互動。

### 5. Verbose 模式

預設情況下，`ask` 只輸出 ChatGPT 的最終回覆，隱藏背景控制訊息。

若要查看完整瀏覽器自動化流程，加入 `--verbose`：

```bash
ask "誰是保哥？" --verbose
```

Verbose 模式會顯示類似以下流程：

- 檢查已開啟的 Chrome 分頁。
- 聚焦輸入欄位。
- 輸入 prompt。
- 送出訊息。
- 等待 ChatGPT 回覆。

### 6. 顯示版本

使用 `-v` 或 `--version` 顯示目前版本號：

```bash
ask -v
```

### 7. Pipe 與 stdin

可透過 pipe 將文字或檔案內容傳入 `ask`：

```bash
echo "用一句話解釋 quantum computing" | ask
```

也可以讀取檔案內容：

```bash
cat src/main.rs | ask "這段 Rust code 有記憶體洩漏風險嗎？"
```

### 8. 只開啟 ChatGPT

若只想快速開啟瀏覽器並進入 ChatGPT：

```bash
ask open
```

## 運作原理

1. **瀏覽器初始化**：`ask` 會檢查 Chrome 是否正在監聽 debug port `9223`。若沒有，會以專屬 profile 目錄 `~/.config/ask-chatgpt/chrome-profile` 啟動 Google Chrome。
2. **MCP Bridge 設定**：啟動時會自動寫入 `~/.config/ask-chatgpt/mcp_servers.json`，預設設定 Chrome DevTools MCP server，使用 `chrome-devtools-mcp@latest` 與 `--browser-url=http://127.0.0.1:9223`。
3. **Client 呼叫**：`ask` 透過內建的 `doggy8088/mcp-cli` Rust library dependency 呼叫 MCP tools，例如 `list_pages`、`select_page`、`type_text` 與 `evaluate_script`，不依賴系統上的外部 `mcp-cli` 命令。
4. **狀態輪詢**：ChatGPT 產生回覆期間，工具會以 JavaScript 檢查送出與停止按鈕狀態，擷取回覆元素的文字內容，並將差異串流輸出到 `stdout`。

## 相關文件

- [快速開始](docs/quick-start.md)
- [背景 Chrome 隱形技術與實作原理](docs/headless-techniques.md)
- [English README](README.en.md)

## 授權

MIT License。可自由使用、修改與散布。
