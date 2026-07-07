# 快速開始

本文件說明如何安裝 `ask-bridge`，並透過 Chrome 自動操作 ChatGPT 或 Gemini。未設定全域 provider 時預設為 ChatGPT，可用 `--provider gemini` 或全域設定檔切換。

## 前置需求

- macOS，且已安裝 Cargo。
- Shell 可使用 Node.js 與 `npx`。
- 若希望 `make install` 在缺少 Google Chrome 時自動安裝 Chrome，需先安裝 Homebrew。

不需要安裝全域 `mcp-cli` 執行檔。本專案會透過 Cargo 從 `https://github.com/doggy8088/mcp-cli` 使用 `mcp-cli` 作為 Rust dependency。

## 安裝

執行：

```sh
make install
```

此命令會：

- 在缺少 Google Chrome 時，透過 Homebrew 安裝 Chrome。
- 建置 release binary。
- 建立 `~/.local/bin/ask-bridge` symlink，指向 release binary。
- 建立 `ask` alias，供既有使用者相容使用。

請確認 `~/.local/bin` 已加入你的 `PATH`。正式 CLI 命令為 `ask-bridge`；`ask` 只是 alias。

## 首次登入

執行：

```sh
ask-bridge login
```

Chrome 會使用專屬 profile 開啟，profile 路徑為：

```text
~/.config/ask-bridge/chrome-profile
```

在瀏覽器視窗登入 ChatGPT 後，回到終端機按 Enter。

若要登入 Gemini：

```sh
ask-bridge --provider gemini login
```

在瀏覽器視窗登入 Gemini 後，回到終端機按 Enter。

## 全域 provider 設定

若希望未指定 `--provider` 時預設使用 Gemini，可執行：

```sh
ask-bridge config --provider gemini
```

若要改回 ChatGPT：

```sh
ask-bridge config --provider chatgpt
```

可用 `ask-bridge config` 檢視目前設定。

`--provider` 會覆蓋全域設定檔，因此可針對單次命令暫時切換 provider。

## 提問

執行：

```sh
ask-bridge "用一段話解釋 Rust ownership。"
ask-bridge --provider gemini "用一段話解釋 Rust ownership。"
```

一般提問預設會使用 headless Chrome，並把所選 provider 的回覆輸出到終端機。

## 使用可見瀏覽器

執行：

```sh
ask-bridge "請示範一個簡短 Markdown 表格。" --headless=false
```

這會在自動化執行期間保持 Chrome 視窗可見。

## 開啟新的 provider 對話

執行：

```sh
ask-bridge "開始一個關於 async Rust 的新主題。" --new
```

`--new` 會開啟新的所選 provider 對話，而不是重用既有的同 provider 分頁。

## 透過 pipe 傳入內容

執行：

```sh
cat README.md | ask-bridge "摘要這份文件。"
```

若未提供 prompt argument，`ask-bridge` 會從 standard input 讀取內容。

若同時提供 prompt argument，`ask-bridge` 會先將 prompt 輸出，並在後方接上兩個換行後再接上標準輸入內容。

## 附上圖片或文件

`ask-bridge` 支援把本機檔案當作附件直接上傳給所選 provider，不必透過 pipe 把內容塞進 prompt。Gemini 目前支援 `--file` 文件附件；`--image` 圖片輸入目前僅支援 ChatGPT。

### 附上圖片

使用 `--image`（可重複指定）。此功能目前僅支援 ChatGPT；搭配 `--provider gemini` 使用會立即回報錯誤。

```sh
ask-bridge "請描述這張圖片。" --image screenshot.png
ask-bridge "比較這兩張圖。" --image v1.png --image v2.png
```

### 附上文件

使用 `--file`（可重複指定）附上 PDF、Word、Excel、純文字、Markdown、CSV、JSON、程式碼等文件：

```sh
ask-bridge "請摘要這份 PDF。" --file report.pdf
ask-bridge "這份 CSV 有幾筆資料？" --file data.csv
ask-bridge "幫我檢查這段程式碼。" --file src/main.rs
```

也可以同時附上圖片與文件：

```sh
ask-bridge "對照這張設計圖與規格文件，指出不一致處。" --image design.png --file spec.docx
```

## 切換模型

使用 `--model` 在送出 prompt 前自動切換 provider 模型（不分大小寫與標點）：

```sh
ask-bridge "用幾句話介紹 Rust。" --model GPT-5.4
ask-bridge "證明這個數學問題。" --model o3
ask-bridge "快速翻譯這段話。" --model 即時
ask-bridge --provider gemini "用幾句話介紹 Rust。" --model "3.5 Flash"
```

可用名稱（視帳號權限與 provider UI）：

- **ChatGPT 模型**：`GPT-5.5`、`GPT-5.4`、`GPT-5.3`、`o3`
- **ChatGPT 思考強度**：`智慧`、`即時`、`中等`、`高`、`超高`、`專業`
- **Gemini 模式**：`3.5 Flash`、`3.1 Flash-Lite`、`3.1 Pro`

## 關閉瀏覽器 instance

執行：

```sh
ask-bridge close
```

`close` 只會關閉使用本專案專屬 Chrome profile 且監聽 debug port `9223` 的瀏覽器 instance；如果沒有執行中的 `ask-bridge` Chrome，會直接回報沒有 instance 正在執行。

## MCP 行為

啟動時，`ask-bridge` 會讀取全域設定檔：

```text
~/.config/ask-bridge/config.json
```

接著會把 MCP 設定寫入：

```text
~/.config/ask-bridge/mcp_servers.json
```

預設 server 是 Chrome DevTools MCP：

```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": [
        "-y",
        "chrome-devtools-mcp@latest",
        "--browser-url=http://127.0.0.1:9223"
      ]
    }
  }
}
```

Rust binary 會透過內建的 `doggy8088/mcp-cli` library dependency 呼叫 Chrome DevTools MCP，不會 shell out 到系統上的 `mcp-cli` 命令。

## 疑難排解

若 Chrome 無法啟動，請確認 Chrome 是否存在於：

```text
/Applications/Google Chrome.app/Contents/MacOS/Google Chrome
```

若 Chrome 存在但 `ask-bridge` 無法連線，請關閉此工具先前建立的 Chrome instance 後重試：

```sh
ask-bridge login
```

若缺少 `npx`，請先安裝 Node.js，再重新執行：

```sh
make install
```
