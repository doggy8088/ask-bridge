/* i18n — zh-TW (default) · zh-CN · ja · ko · en */
(function () {
  "use strict";

  const D = {
    "zh-TW": {
      "nav.skip": "跳至主內容", "nav.why": "為何而做", "nav.features": "功能",
      "nav.usage": "用法", "nav.how": "原理", "nav.install": "安裝",
      "hero.kicker": "給 AI Agent 用的研究橋接器",
      "hero.title": "把終端機，\n接上 ChatGPT 與 Gemini",
      "hero.sub": "ask-bridge 預設透過真實 Chrome 操作網站型 AI；ChatGPT 也可選用 SIWC，以方案額度直接呼叫 API。把問題與工作上下文送出，再將回覆取回終端機。",
      "hero.ctaInstall": "一鍵安裝", "hero.ctaSource": "查看原始碼",
      "hero.termResp": "Rust 的所有權模型：每個值有唯一所有者，離開作用域即自動釋放⋯（回覆自動取回終端機）",
      "why.kicker": "為何而做", "why.title": "不是取代，而是橋接",
      "why.lead": "開發過程中有大量探索性 AI 需求——查資料、整理文件、比較方案、摘要錯誤、產生初稿。這些任務未必需要主要 Coding Agent 親自做，也不值得消耗與改程式、跑測試相同的額度。",
      "why.p1.t": "看清額度來源", "why.p1.d": "瀏覽器模式使用網站額度；SIWC 使用 ChatGPT Work／Codex 方案額度或 credits，可能與主要 Coding Agent 共用，並非免費或無限 token。",
      "why.p2.t": "不離開工作流程", "why.p2.d": "不必切換瀏覽器、貼上 prompt、等待、再複製回來。一行指令完成終端機與網站型 AI 的往返。",
      "why.p3.t": "真實瀏覽器，持久登入", "why.p3.d": "在真實 Chrome 中執行，只需登入一次。可處理 CAPTCHA、Cloudflare，像一般使用者存取網站功能。",
      "features.kicker": "功能", "features.title": "一行指令，完整橋接",
      "features.f1.t": "多 Provider 切換", "features.f1.d": "以 --provider 在 ChatGPT 與 Gemini 之間切換，或在設定檔指定預設 provider。",
      "features.f2.t": "Pipe 與 stdin", "features.f2.d": "把檔案內容或管線文字直接餵進 prompt，無需離開終端機。",
      "features.f3.t": "圖片與文件附件", "features.f3.d": "--image 附上圖片、--file 附上 PDF／Word／CSV／程式碼等文件。",
      "features.f4.t": "模型切換", "features.f4.d": "送出前以 --model 切換 ChatGPT 的 GPT-5.4、o3 或 Gemini 的 3.1 Pro。",
      "features.f5.t": "智慧分頁 · 安靜預設", "features.f5.d": "重用、聚焦或開新分頁避免分頁氾濫；預設只輸出回覆，--verbose 顯示完整流程。",
      "usage.kicker": "快速開始", "usage.title": "瀏覽器模式快速上手",
      "usage.s1.t": "1 · 登入", "usage.s1.d": "首次執行登入所選 provider，只需一次。",
      "usage.s2.t": "2 · 直接提問", "usage.s2.d": "把 prompt 作為參數傳入，回覆輸出到終端機。",
      "usage.s3.t": "3 · 管線與檔案", "usage.s3.d": "透過 pipe 餵入內容，或以 --file 附件上傳。",
      "usage.s4.t": "4 · 切換模型", "usage.s4.d": "送出前指定模型或思考強度。",
      "how.kicker": "運作原理", "how.title": "瀏覽器模式的四個步驟",
      "how.s1.t": "瀏覽器初始化", "how.s1.d": "檢查 Chrome 是否監聽 debug port 9223，若無則以專屬 profile 目錄啟動背景 Chrome。",
      "how.s2.t": "MCP Bridge 設定", "how.s2.d": "自動寫入 mcp_servers.json，設定 chrome-devtools-mcp server 指向本機 9223。",
      "how.s3.t": "Client 呼叫", "how.s3.d": "透過內建 mcp-cli 呼叫 list_pages、select_page、type_text、evaluate_script 等工具。",
      "how.s4.t": "狀態輪詢", "how.s4.d": "以 JavaScript 檢查送出／停止按鈕狀態，擷取回覆元素文字並輸出到 stdout。",
      "install.kicker": "安裝", "install.title": "馬上開始",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "從原始碼", "install.t4": "Agent Skill",
      "install.req.title": "前置需求",
      "install.req.lead": "需要 Node.js 20.19.0 LTS 以上或更新的 LTS 版本，且 node 與 npx 必須可在目前 shell 的 PATH 中執行。ask-bridge 會透過 npx 啟動 chrome-devtools-mcp@latest；Node.js 版本過舊時，MCP server 可能在 initialize 階段直接退出。",
      "install.req.mac.title": "macOS",
      "install.req.mac.body": "可使用 Homebrew 或 nvm 安裝 Node.js LTS。若使用 nvm，請確認執行 ask-bridge 的同一個 shell 已載入 nvm，且 node -v 顯示 v20.19.0 以上。Chrome 預設偵測 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome。",
      "install.req.win.title": "Windows",
      "install.req.win.body": "可使用官方安裝程式、winget 或 nvm-windows 安裝 Node.js LTS。安裝後請重新開啟 PowerShell，並確認 node -v 與 npx -v 可執行。Chrome 會偵測 Program Files、Program Files (x86) 與 %LOCALAPPDATA% 底下的標準安裝路徑。",
      "install.req.linux.title": "Linux",
      "install.req.linux.body": "發行版內建套件庫可能提供較舊的 Node.js；建議使用 NodeSource、nvm 或官方 LTS 來源。請安裝 Google Chrome Stable，並確認 google-chrome 或 google-chrome-stable 位於 PATH；Snap、Flatpak 或 Chromium 安裝方式可能不符合預設偵測邏輯。",
      "cta.title": "讓你的 Agent 多一條研究通路", "cta.sub": "把探索性工作交給網站型 AI，把高價值額度留給真正改變專案的程式碼。",
      "footer.tagline": "給 AI Agent 用的外部研究橋接器",
      "footer.github": "GitHub", "footer.docs": "文件", "footer.issues": "問題回報",
      "footer.by": "由", "footer.crafted": "打造",
      "nav.toDark": "切換至深色模式", "nav.toLight": "切換至淺色模式",
      "nav.copy": "複製", "nav.copied": "已複製", "nav.langLabel": "選擇語言",
      "install.tabsLabel": "安裝方式",
      "doc.title": "ask-bridge — 把終端機接上 ChatGPT 與 Gemini",
      "doc.description": "ask-bridge 是 Rust 命令列研究橋接器，支援 Chrome 網站自動化，或以 Sign in with ChatGPT（SIWC）方案額度直接串流回答。",
      "siwc.title": "ChatGPT 方案，直接接上終端機",
      "siwc.intro": "Sign in with ChatGPT（SIWC）是選用後端。OAuth 登入會開啟系統瀏覽器；之後提問直接使用 HTTP 串流，不啟動 Chrome，也不需要 API Key。未設定時仍使用 Browser/CDP。",
      "siwc.login.t": "1 · 登入並授權",
      "siwc.login.d": "完成 ChatGPT 方案使用授權，等候終端機確認成功。網站登入與 SIWC 憑證互相獨立；可用資格依帳號與 workspace 為準。",
      "siwc.ask.t": "2 · 提問並保存回覆",
      "siwc.ask.d": "回覆即時輸出；只有成功完成後才寫入 -o，成功時會覆寫舊檔。--timeout 涵蓋 token 更新、模型目錄與推論，預設 300 秒。",
      "siwc.context.t": "3 · 傳入工作上下文",
      "siwc.context.d": "prompt 與 stdin 會一起送出。不會自動讀取目前目錄、IDE 或 Agent 對話；多份資料請先加上來源標籤。PowerShell 可用 Get-Content -Raw -Encoding utf8 report.md 取代 cat report.md。",
      "siwc.config.t": "4 · 設成預設，或切回瀏覽器",
      "siwc.config.d": "--provider 選服務，--backend 選本次傳輸方式。config --chatgpt-backend 僅儲存 ChatGPT 的預設後端，不會更改 provider；--backend 可單次覆蓋。",
      "siwc.model": "模型以帳號 API 目錄為準；登入後會嘗試列出模型與 slug。用 --model \"MODEL_SLUG\" 指定實際 slug，省略時使用目錄第一個模型，不沿用網頁選單。",
      "siwc.matrix": "選擇適合的後端",
      "siwc.feature": "功能",
      "siwc.browser": "Browser/CDP",
      "siwc.api": "SIWC",
      "siwc.row.text": "文字、stdin、輸出檔",
      "siwc.yes": "支援",
      "siwc.row.session": "網頁對話、臨時聊天",
      "siwc.web": "依網站功能",
      "siwc.noSession": "不支援；每次提問獨立",
      "siwc.row.tools": "附件、生圖、batch、--reasoning",
      "siwc.provider": "依 provider／功能支援",
      "siwc.noTools": "目前不支援，會明確拒絕",
      "siwc.row.cost": "額度來源",
      "siwc.webCost": "網站產品與帳號限制",
      "siwc.planCost": "ChatGPT Work／Codex 額度或 credits",
      "siwc.history": "需要接續討論？自行把上一輪問題、回覆與必要背景整理成文字，再透過 stdin 傳入。沒有內建多輪歷史，也不能用 --session 接續 chatgpt.com 對話。只提供相關片段，避免敏感資料與過長歷史。",
      "siwc.failure": "部分文字不代表成功。額度錯誤或中斷可能在串流途中發生；失敗時以非零代碼結束、不執行 -o 寫檔，也不會自動改走瀏覽器或 API Key。",
      "siwc.security": "登入失效時重新執行 SIWC login；logout 清除本機帳號與 token，保留 host ID，不撤銷伺服器授權、不登出網站、不改預設後端。Unix 憑證權限為 0600；TLS 檢查環境的受信任攔截方可看到 token 交換內容。",
      "siwc.docs": "完整中文指南",
      "siwc.docsEn": "English guide",
      "install.req.siwc": "以下是瀏覽器模式的需求。SIWC 提問本身不需要 Node.js／npx 或 Chrome；OAuth 仍需要系統瀏覽器，目前安裝腳本也仍檢查瀏覽器相依。",
      "siwc.switch": "單次使用網站功能：以 --backend browser 登入並提問，不改持久設定。",
      "siwc.reset": "恢復預設瀏覽器後端；登出 SIWC 需另外執行 logout。"
    },
    "zh-CN": {
      "nav.skip": "跳至主要内容", "nav.why": "为何而做", "nav.features": "功能",
      "nav.usage": "用法", "nav.how": "原理", "nav.install": "安装",
      "hero.kicker": "给 AI Agent 用的研究桥接器",
      "hero.title": "把终端机，\n接上 ChatGPT 与 Gemini",
      "hero.sub": "ask-bridge 默认通过真实 Chrome 操作网站型 AI；ChatGPT 也可选用 SIWC，以方案额度直接调用 API。发送问题与工作上下文，再把回复带回终端。",
      "hero.ctaInstall": "一键安装", "hero.ctaSource": "查看源代码",
      "hero.termResp": "Rust 的所有权模型：每个值有唯一所有者，离开作用域即自动释放⋯（回复自动取回终端机）",
      "why.kicker": "为何而做", "why.title": "不是取代，而是桥接",
      "why.lead": "开发过程中有大量探索性 AI 需求——查资料、整理文档、比较方案、摘要错误、生成初稿。这些任务未必需要主要 Coding Agent 亲自做，也不值得消耗与改代码、跑测试相同的额度。",
      "why.p1.t": "了解额度来源", "why.p1.d": "浏览器模式使用网站额度；SIWC 使用 ChatGPT Work／Codex 方案额度或 credits，可能与主要 Coding Agent 共用，并非免费或无限 token。",
      "why.p2.t": "不离开工作流程", "why.p2.d": "不必切换浏览器、粘贴 prompt、等待、再复制回来。一行指令完成终端机与网站型 AI 的往返。",
      "why.p3.t": "真实浏览器，持久登录", "why.p3.d": "在真实 Chrome 中执行，只需登录一次。可处理 CAPTCHA、Cloudflare，像一般用户访问网站功能。",
      "features.kicker": "功能", "features.title": "一行指令，完整桥接",
      "features.f1.t": "多 Provider 切换", "features.f1.d": "以 --provider 在 ChatGPT 与 Gemini 之间切换，或在配置文件指定默认 provider。",
      "features.f2.t": "Pipe 与 stdin", "features.f2.d": "把文件内容或管道文字直接喂进 prompt，无需离开终端机。",
      "features.f3.t": "图片与文档附件", "features.f3.d": "--image 附上图片、--file 附上 PDF／Word／CSV／代码等文档。",
      "features.f4.t": "模型切换", "features.f4.d": "提交前以 --model 切换 ChatGPT 的 GPT-5.4、o3 或 Gemini 的 3.1 Pro。",
      "features.f5.t": "智能标签页 · 安静默认", "features.f5.d": "复用、聚焦或开新标签页避免泛滥；默认只输出回复，--verbose 显示完整流程。",
      "usage.kicker": "快速开始", "usage.title": "浏览器模式快速上手",
      "usage.s1.t": "1 · 登录", "usage.s1.d": "首次执行登录所选 provider，只需一次。",
      "usage.s2.t": "2 · 直接提问", "usage.s2.d": "把 prompt 作为参数传入，回复输出到终端机。",
      "usage.s3.t": "3 · 管道与文件", "usage.s3.d": "通过 pipe 喂入内容，或以 --file 附件上传。",
      "usage.s4.t": "4 · 切换模型", "usage.s4.d": "提交前指定模型或思考强度。",
      "how.kicker": "运作原理", "how.title": "浏览器模式的四个步骤",
      "how.s1.t": "浏览器初始化", "how.s1.d": "检查 Chrome 是否监听 debug port 9223，若无则以专属 profile 目录启动后台 Chrome。",
      "how.s2.t": "MCP Bridge 设置", "how.s2.d": "自动写入 mcp_servers.json，设置 chrome-devtools-mcp server 指向本机 9223。",
      "how.s3.t": "Client 调用", "how.s3.d": "通过内置 mcp-cli 调用 list_pages、select_page、type_text、evaluate_script 等工具。",
      "how.s4.t": "状态轮询", "how.s4.d": "以 JavaScript 检查提交／停止按钮状态，截取回复元素文字并输出到 stdout。",
      "install.kicker": "安装", "install.title": "马上开始",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "从源代码", "install.t4": "Agent Skill",
      "install.req.title": "前置要求",
      "install.req.lead": "需要 Node.js 20.19.0 LTS 以上或更新的 LTS 版本，并且 node 与 npx 必须能在当前 shell 的 PATH 中执行。ask-bridge 会通过 npx 启动 chrome-devtools-mcp@latest；Node.js 版本过旧时，MCP server 可能在 initialize 阶段直接退出。",
      "install.req.mac.title": "macOS",
      "install.req.mac.body": "可使用 Homebrew 或 nvm 安装 Node.js LTS。若使用 nvm，请确认运行 ask-bridge 的同一个 shell 已加载 nvm，且 node -v 显示 v20.19.0 以上。Chrome 默认检测 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome。",
      "install.req.win.title": "Windows",
      "install.req.win.body": "可使用官方安装程序、winget 或 nvm-windows 安装 Node.js LTS。安装后请重新打开 PowerShell，并确认 node -v 与 npx -v 可执行。Chrome 会检测 Program Files、Program Files (x86) 与 %LOCALAPPDATA% 下的标准安装路径。",
      "install.req.linux.title": "Linux",
      "install.req.linux.body": "发行版内置软件源可能提供较旧的 Node.js；建议使用 NodeSource、nvm 或官方 LTS 来源。请安装 Google Chrome Stable，并确认 google-chrome 或 google-chrome-stable 位于 PATH；Snap、Flatpak 或 Chromium 安装方式可能不符合默认检测逻辑。",
      "cta.title": "让你的 Agent 多一条研究通路", "cta.sub": "把探索性工作交给网站型 AI，把高价值额度留给真正改变项目的代码。",
      "footer.tagline": "给 AI Agent 用的外部研究桥接器",
      "footer.github": "GitHub", "footer.docs": "文档", "footer.issues": "问题反馈",
      "footer.by": "由", "footer.crafted": "打造",
      "nav.toDark": "切换至深色模式", "nav.toLight": "切换至浅色模式",
      "nav.copy": "复制", "nav.copied": "已复制", "nav.langLabel": "选择语言",
      "install.tabsLabel": "安装方式",
      "doc.title": "ask-bridge — 把终端机接上 ChatGPT 与 Gemini",
      "doc.description": "ask-bridge 是 Rust 命令行研究桥接器，支持 Chrome 网站自动化，或以 Sign in with ChatGPT（SIWC）方案额度直接流式回答。",
      "siwc.title": "在终端使用 ChatGPT 方案",
      "siwc.intro": "Sign in with ChatGPT（SIWC）是可选后端。OAuth 登录会打开系统浏览器；之后提问直接使用 HTTP 流式传输，无需启动 Chrome 或 API Key。默认仍为 Browser/CDP。",
      "siwc.login.t": "1 · 登录并授权",
      "siwc.login.d": "完成 ChatGPT 方案使用授权，等待终端确认。网站登录与 SIWC 凭据独立；资格以账号和 workspace 为准。",
      "siwc.ask.t": "2 · 提问并保存回复",
      "siwc.ask.d": "回复实时输出；-o 仅在成功完成后写入并覆盖旧文件。--timeout 涵盖 token 更新、模型目录及推理，默认 300 秒。",
      "siwc.context.t": "3 · 传入工作上下文",
      "siwc.context.d": "prompt 与 stdin 一起发送，不会自动读取目录、IDE 或 Agent 对话。多份资料请标注来源。PowerShell 可用 Get-Content -Raw -Encoding utf8 report.md 替代 cat report.md。",
      "siwc.config.t": "4 · 设置默认或切回浏览器",
      "siwc.config.d": "--provider 选服务，--backend 选本次传输方式。config --chatgpt-backend 只保存 ChatGPT 默认后端，不改变 provider；--backend 可临时覆盖。",
      "siwc.model": "模型以账号 API 目录为准；登录会尝试列出名称和 slug。用 --model \"MODEL_SLUG\" 指定实际 slug；省略时选目录首个模型，不沿用网页菜单。",
      "siwc.matrix": "选择适合的后端",
      "siwc.feature": "功能",
      "siwc.browser": "Browser/CDP",
      "siwc.api": "SIWC",
      "siwc.row.text": "文字、stdin、输出文件",
      "siwc.yes": "支持",
      "siwc.row.session": "网页对话、临时聊天",
      "siwc.web": "取决于网站",
      "siwc.noSession": "不支持；每次提问独立",
      "siwc.row.tools": "附件、生图、batch、--reasoning",
      "siwc.provider": "取决于 provider／功能",
      "siwc.noTools": "尚未实现，会明确拒绝",
      "siwc.row.cost": "额度来源",
      "siwc.webCost": "网站产品和账号限制",
      "siwc.planCost": "ChatGPT Work／Codex 额度或 credits",
      "siwc.history": "需要继续讨论时，自行把上一轮问题、回复和必要背景整理成文字，通过 stdin 再次传入。没有内置多轮历史，--session 不能继续 chatgpt.com 对话。只提供相关片段，避免敏感资料和过长历史。",
      "siwc.failure": "部分文字不代表成功。额度错误或中断可能在流式传输中途发生；失败以非零代码退出、不执行 -o 写入，不自动切换到浏览器或 API Key。",
      "siwc.security": "授权失效时重新运行 SIWC login。logout 清除本机账号和 token，保留 host ID；不撤销服务器授权、不退出网站、不改默认后端。Unix 凭据权限为 0600；受信任的 TLS 拦截方可看到 token 交换内容。",
      "siwc.docs": "完整中文指南",
      "siwc.docsEn": "English guide",
      "install.req.siwc": "以下是浏览器模式的要求。SIWC 提问本身无需 Node.js／npx 或 Chrome；OAuth 需要系统浏览器，目前安装脚本仍检查浏览器依赖。",
      "siwc.switch": "本次使用网站功能：以 --backend browser 登录并提问，不改变持久设置。",
      "siwc.reset": "恢复默认浏览器后端；退出 SIWC 需另外运行 logout。"
    },
    "ja": {
      "nav.skip": "メインへスキップ", "nav.why": "目的", "nav.features": "機能",
      "nav.usage": "使い方", "nav.how": "仕組み", "nav.install": "インストール",
      "hero.kicker": "AI Agent のための研究ブリッジ",
      "hero.title": "ターミナルを、\nChatGPT と Gemini に接続",
      "hero.sub": "ask-bridge は標準で Chrome を使ってウェブ AI を操作します。ChatGPT では SIWC を選び、プランの利用枠で API に直接質問することもできます。質問と作業の背景を送り、回答をターミナルに取得します。",
      "hero.ctaInstall": "ワンライン install", "hero.ctaSource": "ソースを見る",
      "hero.termResp": "Rust の所有権モデル：各値には唯一の所有者がおり、スコープを抜けると自動で解放されます⋯（返答は自動でターミナルに戻ります）",
      "why.kicker": "目的", "why.title": "置き換えではなく、橋渡し",
      "why.lead": "開発中には探索的な AI 需要が大量に生じます——調査、整理、比較、要約、下書き。これらはメインの Coding Agent が自らやる必要はないケースが多く、コード編集やテストと同じ枠を使う値打ちもありません。",
      "why.p1.t": "利用枠の出所を確認", "why.p1.d": "ブラウザ方式はウェブの利用枠、SIWC は ChatGPT Work/Codex の利用枠や credits を使います。メインの Coding Agent と共有する場合があり、無料・無制限ではありません。",
      "why.p2.t": "ワークフローから離れない", "why.p2.d": "ブラウザ切替・貼り付け・待機・コピー戻しをせず、一行でターミナルとウェブ型 AI の往復を完結します。",
      "why.p3.t": "実ブラウザ、永続ログイン", "why.p3.d": "実 Chrome で実行し、ログインは一度だけ。CAPTCHA や Cloudflare も一般ユーザーと同様に処理できます。",
      "features.kicker": "機能", "features.title": "一行で、完全なブリッジ",
      "features.f1.t": "マルチ Provider 切替", "features.f1.d": "--provider で ChatGPT と Gemini を切り替え、設定ファイルで既定 provider も指定可能。",
      "features.f2.t": "Pipe と stdin", "features.f2.d": "ファイル内容やパイプのテキストを直接プロンプトに流し、ターミナルから離れません。",
      "features.f3.t": "画像・ファイル添付", "features.f3.d": "--image で画像、--file で PDF／Word／CSV／コード等を添付。",
      "features.f4.t": "モデル切替", "features.f4.d": "送信前に --model で ChatGPT の GPT-5.4・o3、Gemini の 3.1 Pro などを指定。",
      "features.f5.t": "スマートタブ・静音 既定", "features.f5.d": "タブの再利用・フォーカス・新規開封で煩雑化を防ぎ、既定は返答のみ、--verbose で全過程を表示。",
      "usage.kicker": "クイックスタート", "usage.title": "ブラウザ方式のクイックスタート",
      "usage.s1.t": "1 · ログイン", "usage.s1.d": "初回は選んだ provider にログイン。一度だけ。",
      "usage.s2.t": "2 · 直接質問", "usage.s2.d": "プロンプトを引数で渡すと、返答がターミナルに出力されます。",
      "usage.s3.t": "3 · パイプとファイル", "usage.s3.d": "pipe で内容を流し込むか、--file で添付します。",
      "usage.s4.t": "4 · モデル切替", "usage.s4.d": "送信前にモデルや思考強度を指定します。",
      "how.kicker": "仕組み", "how.title": "ブラウザ方式の4ステップ",
      "how.s1.t": "ブラウザ初期化", "how.s1.d": "Chrome が debug port 9223 を監聴しているか確認し、無ければ専用 profile でバックグラウンド Chrome を起動。",
      "how.s2.t": "MCP Bridge 設定", "how.s2.d": "mcp_servers.json を自動書き込み、chrome-devtools-mcp server を localhost:9223 に向けます。",
      "how.s3.t": "Client 呼び出し", "how.s3.d": "内蔵 mcp-cli で list_pages・select_page・type_text・evaluate_script などを呼び出します。",
      "how.s4.t": "状態ポーリング", "how.s4.d": "JavaScript で送信／停止ボタン状態を確認し、返答要素のテキストを stdout へ出力。",
      "install.kicker": "インストール", "install.title": "今すぐ始める",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "ソースから", "install.t4": "Agent Skill",
      "install.req.title": "前提条件",
      "install.req.lead": "Node.js 20.19.0 LTS 以上、またはより新しい LTS が必要です。node と npx は現在の shell の PATH から実行できる必要があります。ask-bridge は npx 経由で chrome-devtools-mcp@latest を起動します。Node.js が古い場合、MCP server は initialize 段階で終了することがあります。",
      "install.req.mac.title": "macOS",
      "install.req.mac.body": "Node.js LTS は Homebrew または nvm でインストールできます。nvm を使う場合は、ask-bridge を実行する同じ shell で nvm が読み込まれ、node -v が v20.19.0 以上を示すことを確認してください。Chrome は既定で /Applications/Google Chrome.app/Contents/MacOS/Google Chrome を検出します。",
      "install.req.win.title": "Windows",
      "install.req.win.body": "Node.js LTS は公式 installer、winget、または nvm-windows でインストールできます。インストール後は PowerShell を開き直し、node -v と npx -v が実行できることを確認してください。Chrome は Program Files、Program Files (x86)、%LOCALAPPDATA% の標準パスから検出されます。",
      "install.req.linux.title": "Linux",
      "install.req.linux.body": "ディストリビューションの package repository は古い Node.js を提供することがあります。NodeSource、nvm、または公式 LTS source を推奨します。Google Chrome Stable をインストールし、google-chrome または google-chrome-stable が PATH にあることを確認してください。Snap、Flatpak、Chromium は既定の検出ロジックに一致しない場合があります。",
      "cta.title": "Agent にもう一本、研究の道を", "cta.sub": "探索的な作業はウェブ型 AI に、高価値な枠はプロジェクトを動かすコードに。",
      "footer.tagline": "AI Agent のための外部研究ブリッジ",
      "footer.github": "GitHub", "footer.docs": "ドキュメント", "footer.issues": "Issue",
      "footer.by": "制作：", "footer.crafted": "",
      "nav.toDark": "ダークモードへ切り替え", "nav.toLight": "ライトモードへ切り替え",
      "nav.copy": "コピー", "nav.copied": "コピー済み", "nav.langLabel": "言語を選択",
      "install.tabsLabel": "インストール方法",
      "doc.title": "ask-bridge — ターミナルを ChatGPT と Gemini に接続",
      "doc.description": "Rust CLI の ask-bridge。Chrome によるウェブ AI の自動操作、または Sign in with ChatGPT（SIWC）のプラン利用枠で回答をストリーミング取得。",
      "siwc.title": "ChatGPT プランをターミナルから",
      "siwc.intro": "Sign in with ChatGPT（SIWC）は任意のバックエンドです。OAuth はシステムブラウザで認証し、以後の質問は Chrome を起動せず HTTP ストリーミングで送信します。API キーは不要。既定は Browser/CDP です。",
      "siwc.login.t": "1 · ログインと認可",
      "siwc.login.d": "ChatGPT プランの利用を許可し、ターミナルの成功表示を待ちます。ウェブへのログインとは別の認証です。利用資格はアカウントと workspace に依存します。",
      "siwc.ask.t": "2 · 質問と回答の保存",
      "siwc.ask.d": "回答は順次表示され、成功後にのみ -o へ保存されます。成功時は既存ファイルを上書きします。--timeout は更新・モデル取得・推論を含み、既定は300秒です。",
      "siwc.context.t": "3 · 作業の背景を渡す",
      "siwc.context.d": "prompt と stdin をまとめて送ります。ディレクトリ、IDE、Agent の会話は自動取得しません。資料には出典を付けてください。PowerShell では cat report.md の代わりに Get-Content -Raw -Encoding utf8 report.md を使えます。",
      "siwc.config.t": "4 · 既定の設定と切り替え",
      "siwc.config.d": "--provider はサービス、--backend は今回の通信方式です。config --chatgpt-backend は ChatGPT のバックエンドのみ保存し、provider を変更しません。--backend で一時的に上書きできます。",
      "siwc.model": "モデルはアカウントの API カタログで確認します。ログイン時に名前と slug の表示を試みます。--model \"MODEL_SLUG\" を実際の slug に置き換えてください。省略時は最初のモデルを使い、ウェブの選択は引き継ぎません。",
      "siwc.matrix": "用途に合ったバックエンド",
      "siwc.feature": "機能",
      "siwc.browser": "Browser/CDP",
      "siwc.api": "SIWC",
      "siwc.row.text": "テキスト・stdin・ファイル出力",
      "siwc.yes": "対応",
      "siwc.row.session": "ウェブの会話・一時チャット",
      "siwc.web": "ウェブ側の機能に依存",
      "siwc.noSession": "非対応。毎回独立した質問",
      "siwc.row.tools": "添付・画像生成・batch・--reasoning",
      "siwc.provider": "provider／機能に依存",
      "siwc.noTools": "未実装。明示的に拒否",
      "siwc.row.cost": "利用枠",
      "siwc.webCost": "ウェブ製品・アカウントの制限",
      "siwc.planCost": "ChatGPT Work/Codex の利用枠・credits",
      "siwc.history": "議論を続けるには、前の質問・回答・必要な背景をテキストにまとめ、stdin で再送してください。複数ターンの履歴管理はなく、--session で chatgpt.com の会話を再開できません。秘密情報と長すぎる履歴を避け、関連部分だけ送ります。",
      "siwc.failure": "途中のテキスト表示は成功を意味しません。利用枠のエラーや切断は途中でも発生します。失敗時は非ゼロで終了し、-o に書き込まず、ブラウザや API キー課金へ自動切り替えもしません。",
      "siwc.security": "認証が失効したら SIWC login を再実行します。logout はローカルのアカウントと token を削除し、host ID を保持します。サーバーの認可取消・ウェブログアウト・既定変更は行いません。Unix 権限は0600です。信頼された TLS 検査者は token 交換を閲覧できます。",
      "siwc.docs": "繁体字中国語ガイド",
      "siwc.docsEn": "English guide",
      "install.req.siwc": "以下はブラウザ方式の要件です。SIWC の質問には Node.js/npx や Chrome は不要ですが、OAuth にはシステムブラウザが必要です。現行インストーラーはブラウザ依存関係も確認します。",
      "siwc.switch": "今回だけウェブ機能を利用：--backend browser でログインして質問します。保存済み設定は変更しません。",
      "siwc.reset": "既定をブラウザに戻します。SIWC のログアウトには別途 logout が必要です。"
    },
    "ko": {
      "nav.skip": "본문으로 건너뛰기", "nav.why": "목적", "nav.features": "기능",
      "nav.usage": "사용법", "nav.how": "원리", "nav.install": "설치",
      "hero.kicker": "AI Agent 를 위한 연구 브릿지",
      "hero.title": "터미널을,\nChatGPT 와 Gemini 에 연결",
      "hero.sub": "ask-bridge는 기본적으로 Chrome으로 웹 AI를 조작합니다. ChatGPT는 SIWC를 선택해 플랜 사용량으로 API에 직접 요청할 수도 있습니다. 질문과 작업 맥락을 보내고 답변을 터미널로 가져옵니다.",
      "hero.ctaInstall": "원클릭 설치", "hero.ctaSource": "소스 보기",
      "hero.termResp": "Rust 의 소유권 모델: 각 값은 유일한 소유자를 가지며, 스코프를 벗어나면 자동 해제됩니다⋯(답변이 자동으로 터미널로 돌아옵니다)",
      "why.kicker": "목적", "why.title": "대체가 아니라 연결",
      "why.lead": "개발 중에는 탐색적 AI 수요가 많습니다——조사, 정리, 비교, 요약, 초안. 이런 작업은 메인 Coding Agent 가 직접 할 필요가 없고, 코드 수정·테스트와 같은 한도를 쓸 만큼의 가치도 아닐 때가 많습니다.",
      "why.p1.t": "사용량 출처 확인", "why.p1.d": "브라우저 모드는 웹 사용량을, SIWC는 ChatGPT Work/Codex 사용량 또는 credits를 씁니다. 주 Coding Agent와 한도를 공유할 수 있으며 무료나 무제한이 아닙니다.",
      "why.p2.t": "워크플로를 떠나지 않기", "why.p2.d": "브라우저 전환·붙여넣기·대기·복사 돌아오기 없이, 한 줄로 터미널과 웹형 AI 의 왕복을 끝냅니다.",
      "why.p3.t": "실제 브라우저, 영구 로그인", "why.p3.d": "실제 Chrome 에서 실행, 로그인은 한 번만. CAPTCHA 와 Cloudflare 도 일반 사용자처럼 처리합니다.",
      "features.kicker": "기능", "features.title": "한 줄로, 완전한 브릿지",
      "features.f1.t": "멀티 Provider 전환", "features.f1.d": "--provider 로 ChatGPT 와 Gemini 를 전환, 설정 파일로 기본 provider 지정.",
      "features.f2.t": "Pipe 와 stdin", "features.f2.d": "파일 내용이나 파이프 텍스트를 프로ンプ트에 직접 넣고 터미널을 떠나지 않습니다.",
      "features.f3.t": "이미지·파일 첨부", "features.f3.d": "--image 로 이미지, --file 로 PDF／Word／CSV／코드 등 첨부.",
      "features.f4.t": "모델 전환", "features.f4.d": "전송 전 --model 로 ChatGPT 의 GPT-5.4·o3, Gemini 의 3.1 Pro 지정.",
      "features.f5.t": "스마트 탭 · 조용한 기본", "features.f5.d": "탭 재사용·포커스·신규 열기로 탭 난립을 막고, 기본은 답변만, --verbose 로 전 과정 표시.",
      "usage.kicker": "빠른 시작", "usage.title": "브라우저 모드 빠른 시작",
      "usage.s1.t": "1 · 로그인", "usage.s1.d": "처음에 선택한 provider 에 로그인. 한 번만.",
      "usage.s2.t": "2 · 직접 질문", "usage.s2.d": "프롬프트를 인수로 넘기면 답변이 터미널에 출력됩니다.",
      "usage.s3.t": "3 · 파이프와 파일", "usage.s3.d": "pipe 로 내용을 넣거나 --file 로 첨부합니다.",
      "usage.s4.t": "4 · 모델 전환", "usage.s4.d": "전송 전 모델이나 사고 강도를 지정합니다.",
      "how.kicker": "원리", "how.title": "브라우저 모드의 네 단계",
      "how.s1.t": "브라우저 초기화", "how.s1.d": "Chrome 이 debug port 9223 을 듣는지 확인, 없으면 전용 profile 로 백그라운드 Chrome 시작.",
      "how.s2.t": "MCP Bridge 설정", "how.s2.d": "mcp_servers.json 을 자동 작성, chrome-devtools-mcp server 를 localhost:9223 으로 지정.",
      "how.s3.t": "Client 호출", "how.s3.d": "내장 mcp-cli 로 list_pages·select_page·type_text·evaluate_script 등 호출.",
      "how.s4.t": "상태 폴링", "how.s4.d": "JavaScript 로 전송／정지 버튼 상태를 확인, 답변 요소 텍스트를 stdout 으로 출력.",
      "install.kicker": "설치", "install.title": "지금 시작",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "소스에서", "install.t4": "Agent Skill",
      "install.req.title": "필수 조건",
      "install.req.lead": "Node.js 20.19.0 LTS 이상 또는 더 새로운 LTS 가 필요하며, node 와 npx 는 현재 shell 의 PATH 에서 실행 가능해야 합니다. ask-bridge 는 npx 로 chrome-devtools-mcp@latest 를 시작합니다. Node.js 가 너무 오래되면 MCP server 가 initialize 단계에서 종료될 수 있습니다.",
      "install.req.mac.title": "macOS",
      "install.req.mac.body": "Node.js LTS 는 Homebrew 또는 nvm 으로 설치할 수 있습니다. nvm 을 사용한다면 ask-bridge 를 실행하는 같은 shell 에서 nvm 이 로드되었고 node -v 가 v20.19.0 이상인지 확인하세요. Chrome 은 기본적으로 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome 에서 감지됩니다.",
      "install.req.win.title": "Windows",
      "install.req.win.body": "Node.js LTS 는 공식 설치 프로그램, winget 또는 nvm-windows 로 설치할 수 있습니다. 설치 후 PowerShell 을 다시 열고 node -v 와 npx -v 가 실행되는지 확인하세요. Chrome 은 Program Files, Program Files (x86), %LOCALAPPDATA% 의 표준 경로에서 감지됩니다.",
      "install.req.linux.title": "Linux",
      "install.req.linux.body": "배포판 package repository 는 오래된 Node.js 를 제공할 수 있습니다. NodeSource, nvm 또는 공식 LTS source 사용을 권장합니다. Google Chrome Stable 을 설치하고 google-chrome 또는 google-chrome-stable 이 PATH 에 있는지 확인하세요. Snap, Flatpak 또는 Chromium 설치는 기본 감지 로직과 맞지 않을 수 있습니다.",
      "cta.title": "Agent 에 연구 길 하나 더", "cta.sub": "탐색적 작업은 웹형 AI 에, 고가치 한도는 프로젝트를 움직이는 코드에.",
      "footer.tagline": "AI Agent 를 위한 외부 연구 브릿지",
      "footer.github": "GitHub", "footer.docs": "문서", "footer.issues": "이슈",
      "footer.by": "제작：", "footer.crafted": "",
      "nav.toDark": "다크 모드로 전환", "nav.toLight": "라이트 모드로 전환",
      "nav.copy": "복사", "nav.copied": "복사됨", "nav.langLabel": "언어 선택",
      "install.tabsLabel": "설치 방법",
      "doc.title": "ask-bridge — 터미널을 ChatGPT 와 Gemini 에 연결",
      "doc.description": "Rust CLI ask-bridge: Chrome 웹 AI 자동화 또는 Sign in with ChatGPT(SIWC) 플랜 사용량으로 답변을 스트리밍합니다.",
      "siwc.title": "터미널에서 ChatGPT 플랜 사용",
      "siwc.intro": "Sign in with ChatGPT(SIWC)는 선택적 백엔드입니다. OAuth 로그인은 시스템 브라우저를 열고, 이후 질문은 Chrome 실행이나 API 키 없이 HTTP로 스트리밍합니다. 기본값은 Browser/CDP입니다.",
      "siwc.login.t": "1 · 로그인 및 승인",
      "siwc.login.d": "ChatGPT 플랜 사용을 승인하고 터미널의 성공 확인을 기다립니다. 웹 로그인과 SIWC 자격 증명은 별개입니다. 사용 자격은 계정과 workspace에 따라 다릅니다.",
      "siwc.ask.t": "2 · 질문하고 답변 저장",
      "siwc.ask.d": "답변은 실시간으로 표시됩니다. -o는 성공 후에만 기록하며 기존 파일을 덮어씁니다. --timeout은 토큰 갱신, 모델 조회, 추론을 포함하며 기본 300초입니다.",
      "siwc.context.t": "3 · 작업 맥락 전달",
      "siwc.context.d": "prompt와 stdin을 함께 보냅니다. 디렉터리, IDE, Agent 대화는 자동으로 읽지 않습니다. 여러 자료에는 출처를 붙이세요. PowerShell에서는 cat report.md 대신 Get-Content -Raw -Encoding utf8 report.md를 쓸 수 있습니다.",
      "siwc.config.t": "4 · 기본값 저장 또는 브라우저 전환",
      "siwc.config.d": "--provider는 서비스, --backend는 이번 실행의 전송 방식을 선택합니다. config --chatgpt-backend는 ChatGPT 백엔드만 저장하며 provider는 바꾸지 않습니다. --backend로 한 번만 재정의할 수 있습니다.",
      "siwc.model": "모델은 계정 API 목록을 따릅니다. 로그인 시 이름과 slug 조회를 시도합니다. --model \"MODEL_SLUG\"를 실제 slug로 바꾸세요. 생략하면 목록의 첫 모델을 사용하며 웹 메뉴 선택과 별개입니다.",
      "siwc.matrix": "작업에 맞는 백엔드",
      "siwc.feature": "기능",
      "siwc.browser": "Browser/CDP",
      "siwc.api": "SIWC",
      "siwc.row.text": "텍스트, stdin, 파일 출력",
      "siwc.yes": "지원",
      "siwc.row.session": "웹 대화, 임시 채팅",
      "siwc.web": "웹 기능에 따라 다름",
      "siwc.noSession": "미지원; 요청마다 독립",
      "siwc.row.tools": "첨부, 이미지 생성, batch, --reasoning",
      "siwc.provider": "provider/기능에 따라 다름",
      "siwc.noTools": "미구현; 명시적으로 거부",
      "siwc.row.cost": "사용량 출처",
      "siwc.webCost": "웹 제품 및 계정 한도",
      "siwc.planCost": "ChatGPT Work/Codex 한도 또는 credits",
      "siwc.history": "논의를 이어가려면 이전 질문, 답변, 필요한 배경을 텍스트로 정리해 stdin으로 다시 보내세요. 내장 다중 턴 기록은 없으며 --session으로 chatgpt.com 대화를 이어갈 수 없습니다. 비밀 정보와 과도한 기록을 피하고 관련 부분만 보내세요.",
      "siwc.failure": "일부 텍스트가 표시되어도 성공한 것은 아닙니다. 스트리밍 중 한도 오류나 중단이 발생할 수 있습니다. 실패하면 0이 아닌 코드로 종료하고 -o에 기록하지 않으며 브라우저나 API 키 결제로 자동 전환하지 않습니다.",
      "siwc.security": "인증이 만료되면 SIWC login을 다시 실행합니다. logout은 로컬 계정과 토큰을 지우고 host ID는 유지합니다. 서버 승인 철회, 웹 로그아웃, 기본 백엔드 변경은 하지 않습니다. Unix 권한은 0600입니다. 신뢰된 TLS 검사자는 토큰 교환을 볼 수 있습니다.",
      "siwc.docs": "번체 중국어 가이드",
      "siwc.docsEn": "English guide",
      "install.req.siwc": "아래는 브라우저 모드 요구 사항입니다. SIWC 질문 자체는 Node.js/npx나 Chrome이 필요 없지만 OAuth에는 시스템 브라우저가 필요하며 현재 설치 스크립트는 브라우저 의존성도 검사합니다.",
      "siwc.switch": "이번에만 웹 기능 사용: --backend browser로 로그인하고 질문합니다. 저장한 설정은 바뀌지 않습니다.",
      "siwc.reset": "기본 백엔드를 브라우저로 복원합니다. SIWC 로그아웃은 별도로 logout을 실행하세요."
    },
    "en": {
      "nav.skip": "Skip to content", "nav.why": "Why", "nav.features": "Features",
      "nav.usage": "Usage", "nav.how": "How", "nav.install": "Install",
      "hero.kicker": "A research bridge for AI agents",
      "hero.title": "Wire your terminal \nto ChatGPT and Gemini",
      "hero.sub": "ask-bridge automates web AI through Chrome by default. ChatGPT also offers an optional SIWC backend for direct API requests using your plan. Send questions and working context, then bring answers back to your terminal.",
      "hero.ctaInstall": "One-line install", "hero.ctaSource": "View source",
      "hero.termResp": "Rust's ownership model: every value has a single owner and is freed automatically when it goes out of scope… (the reply is returned to your terminal)",
      "why.kicker": "Why", "why.title": "A bridge, not a replacement",
      "why.lead": "Development is full of exploratory AI needs: research, summarizing, comparing options, digesting errors, drafting. These rarely need your main coding agent, and rarely justify spending the same budget reserved for editing code and running tests.",
      "why.p1.t": "Know your usage source", "why.p1.d": "Browser mode uses website allowances. SIWC uses ChatGPT Work/Codex plan usage or credits, which may be shared with your primary Coding Agent. It is not free or unlimited tokens.",
      "why.p2.t": "Stay in your workflow", "why.p2.d": "No switching to a browser, pasting a prompt, waiting, copying back. One command round-trips the terminal and the web AI.",
      "why.p3.t": "Real browser, persistent login", "why.p3.d": "Runs in a real Chrome, log in once. Handles CAPTCHA and Cloudflare like any normal user.",
      "features.kicker": "Features", "features.title": "One command, the full bridge",
      "features.f1.t": "Multi-provider switch", "features.f1.d": "Switch ChatGPT and Gemini with --provider, or set a default in the config file.",
      "features.f2.t": "Pipe & stdin", "features.f2.d": "Feed file contents or piped text straight into the prompt without leaving the terminal.",
      "features.f3.t": "Image & file attachments", "features.f3.d": "Attach images with --image and PDF/Word/CSV/code with --file.",
      "features.f4.t": "Model switching", "features.f4.d": "Switch to ChatGPT's GPT-5.4, o3, or Gemini's 3.1 Pro before submitting.",
      "features.f5.t": "Smart tabs · quiet by default", "features.f5.d": "Reuse, focus, or open new tabs to avoid clutter; defaults to the reply only, --verbose shows the full flow.",
      "usage.kicker": "Quick start", "usage.title": "Browser mode quick start",
      "usage.s1.t": "1 · Log in", "usage.s1.d": "Log in to your chosen provider the first time. Once.",
      "usage.s2.t": "2 · Ask directly", "usage.s2.d": "Pass the prompt as an argument; the reply prints to your terminal.",
      "usage.s3.t": "3 · Pipe & files", "usage.s3.d": "Feed content via pipe, or upload with --file.",
      "usage.s4.t": "4 · Switch model", "usage.s4.d": "Pick a model or thinking level before submitting.",
      "how.kicker": "How it works", "how.title": "Four steps in browser mode",
      "how.s1.t": "Browser init", "how.s1.d": "Checks whether Chrome is listening on debug port 9223; if not, launches a background Chrome with a dedicated profile.",
      "how.s2.t": "MCP bridge config", "how.s2.d": "Writes mcp_servers.json automatically, pointing chrome-devtools-mcp at localhost:9223.",
      "how.s3.t": "Client calls", "how.s3.d": "Uses the embedded mcp-cli to call list_pages, select_page, type_text, evaluate_script, and more.",
      "how.s4.t": "State polling", "how.s4.d": "JavaScript checks the send/stop button states and streams the reply text to stdout.",
      "install.kicker": "Install", "install.title": "Get started now",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "From source", "install.t4": "Agent Skill",
      "install.req.title": "Prerequisites",
      "install.req.lead": "You need Node.js 20.19.0 LTS or a newer LTS, and both node and npx must be available in the current shell's PATH. ask-bridge starts chrome-devtools-mcp@latest through npx; older Node.js versions can cause the MCP server to exit during initialize.",
      "install.req.mac.title": "macOS",
      "install.req.mac.body": "Install a Node.js LTS release with Homebrew or nvm. If you use nvm, make sure the same shell that runs ask-bridge has loaded nvm and that node -v reports v20.19.0 or newer. Chrome is detected at /Applications/Google Chrome.app/Contents/MacOS/Google Chrome by default.",
      "install.req.win.title": "Windows",
      "install.req.win.body": "Install a Node.js LTS release with the official installer, winget, or nvm-windows. Reopen PowerShell after installation, then verify that node -v and npx -v work. Chrome is detected from the standard Program Files, Program Files (x86), and %LOCALAPPDATA% paths.",
      "install.req.linux.title": "Linux",
      "install.req.linux.body": "Distribution package repositories may provide an older Node.js version. Prefer NodeSource, nvm, or an official LTS source. Install Google Chrome Stable and make sure google-chrome or google-chrome-stable is available in PATH; Snap, Flatpak, or Chromium installs may not match the default detection logic.",
      "cta.title": "Give your agent another research path", "cta.sub": "Route exploratory work to web AI, and keep the high-value budget for the code that actually moves the project.",
      "footer.tagline": "An external research bridge for AI agents",
      "footer.github": "GitHub", "footer.docs": "Docs", "footer.issues": "Issues",
      "footer.by": "Built by ", "footer.crafted": "",
      "nav.toDark": "Switch to dark mode", "nav.toLight": "Switch to light mode",
      "nav.copy": "Copy", "nav.copied": "Copied", "nav.langLabel": "Choose language",
      "install.tabsLabel": "Install method",
      "doc.title": "ask-bridge — Wire your terminal to ChatGPT and Gemini",
      "doc.description": "ask-bridge is a Rust CLI research bridge: automate web AI through Chrome or stream answers directly with Sign in with ChatGPT (SIWC) plan usage.",
      "siwc.title": "Your ChatGPT plan, in your terminal",
      "siwc.intro": "Sign in with ChatGPT (SIWC) is an optional backend. OAuth opens your system browser; subsequent questions use HTTP streaming without launching Chrome or requiring an API key. Browser/CDP remains the default.",
      "siwc.login.t": "1 · Sign in and authorize",
      "siwc.login.d": "Authorize ChatGPT plan usage and wait for terminal confirmation. Website login and SIWC credentials are separate; eligibility depends on your account and workspace.",
      "siwc.ask.t": "2 · Ask and save an answer",
      "siwc.ask.d": "Text streams live. -o writes only after successful completion and overwrites an existing file on success. --timeout covers refresh, model lookup, and inference; the default is 300 seconds.",
      "siwc.context.t": "3 · Supply your working context",
      "siwc.context.d": "The prompt and stdin are sent together. No directory, IDE, or Agent conversation is read automatically. Label multiple sources. In PowerShell, replace cat report.md with Get-Content -Raw -Encoding utf8 report.md.",
      "siwc.config.t": "4 · Save a default or use the browser",
      "siwc.config.d": "--provider selects the service; --backend selects this invocation’s transport. config --chatgpt-backend saves only the ChatGPT backend, without changing the provider. --backend overrides it for one invocation.",
      "siwc.model": "Models come from your account’s API catalog. Sign-in attempts to list names and slugs. Use --model \"MODEL_SLUG\" with an actual slug; omitting it selects the first catalog entry, independently of the web model menu.",
      "siwc.matrix": "Choose the backend for your task",
      "siwc.feature": "Feature",
      "siwc.browser": "Browser/CDP",
      "siwc.api": "SIWC",
      "siwc.row.text": "Text, stdin, output files",
      "siwc.yes": "Supported",
      "siwc.row.session": "Web conversations, temporary chats",
      "siwc.web": "Website-dependent",
      "siwc.noSession": "Unsupported; each request is independent",
      "siwc.row.tools": "Attachments, images, batch, --reasoning",
      "siwc.provider": "Provider/feature-dependent",
      "siwc.noTools": "Not implemented; explicitly rejected",
      "siwc.row.cost": "Usage source",
      "siwc.webCost": "Website product/account limits",
      "siwc.planCost": "ChatGPT Work/Codex allowance or credits",
      "siwc.history": "Continuing a discussion? Include the previous question, answer, and relevant background as text through stdin. There is no built-in multi-turn history, and --session cannot resume chatgpt.com conversations. Send relevant excerpts, avoiding secrets and excessive history.",
      "siwc.failure": "Partial text does not mean success. Quota errors or interruptions can arrive mid-stream. Failures exit nonzero without performing the -o write, and never automatically switch to the browser or API-key billing.",
      "siwc.security": "Run SIWC login again if authorization expires. logout clears local accounts and tokens but preserves the host ID; it does not revoke server authorization, sign out of the website, or reset the backend. Unix credentials use 0600 permissions. Trusted TLS interceptors can see token exchanges.",
      "siwc.docs": "完整中文指南",
      "siwc.docsEn": "English guide",
      "install.req.siwc": "The requirements below apply to browser mode. SIWC questions do not need Node.js/npx or Chrome, but OAuth needs a system browser and the current installation scripts still check browser dependencies.",
      "siwc.switch": "Use website features once: sign in and ask with --backend browser without changing your saved preference.",
      "siwc.reset": "Restore the default browser backend. To sign out of SIWC, run logout separately."
    }
  };

  const LABELS = {
    "zh-TW": "繁體中文", "zh-CN": "简体中文", "ja": "日本語", "ko": "한국어", "en": "English"
  };
  const ORDER = ["zh-TW", "zh-CN", "ja", "ko", "en"];
  const STORE = "ab-lang";
  let current = "zh-TW";

  function apply(lang) {
    if (!D[lang]) lang = "zh-TW";
    current = lang;
    const dict = D[lang];
    document.documentElement.lang = lang;
    document.querySelectorAll("[data-i18n]").forEach((el) => {
      const k = el.getAttribute("data-i18n");
      const v = dict[k];
      if (v == null) return;
      if (k === "hero.title") {
        el.innerHTML = v.replace(/\n/g, "<br/>");
      } else {
        el.textContent = v;
      }
    });
    document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
      const v = dict[el.getAttribute("data-i18n-aria")];
      if (v == null) return;
      el.setAttribute("aria-label", v);
      if (el.tagName === "BUTTON") el.setAttribute("title", v);
    });
    const lbl = document.getElementById("langLabel");
    if (lbl) lbl.textContent = LABELS[lang];
    const menu = document.getElementById("langMenu");
    if (menu) menu.querySelectorAll("button[data-lang]").forEach((b) => {
      const sel = b.getAttribute("data-lang") === lang;
      if (sel) b.setAttribute("aria-current", "true"); else b.removeAttribute("aria-current");
    });
    if (dict["doc.title"]) document.title = dict["doc.title"];
    if (dict["doc.description"]) {
      const m = document.querySelector('meta[name="description"]');
      if (m) m.setAttribute("content", dict["doc.description"]);
    }
    const themeBtn = document.getElementById("themeBtn");
    if (themeBtn) {
      const pressed = themeBtn.getAttribute("aria-pressed") === "true";
      const key = pressed ? "nav.toDark" : "nav.toLight";
      if (dict[key]) { themeBtn.setAttribute("aria-label", dict[key]); themeBtn.setAttribute("title", dict[key]); }
    }
  }

  function t(key) {
    return (D[current] || D["zh-TW"])[key];
  }

  function fromUrl() {
    try {
      const p = new URLSearchParams(window.location.search);
      const l = p.get("lang");
      if (l && D[l]) return l;
    } catch (e) {}
    return null;
  }

  function init() {
    let lang = fromUrl() || "zh-TW";
    try { lang = localStorage.getItem(STORE) || lang; } catch (e) {}
    if (!D[lang]) lang = "zh-TW";
    apply(lang);

    const btn = document.getElementById("langBtn");
    const menu = document.getElementById("langMenu");
    if (btn && menu) {
      // switch from the no-JS `hidden` default to class-driven animated control
      menu.removeAttribute("hidden");
      const close = () => { menu.classList.remove("open"); btn.setAttribute("aria-expanded", "false"); };
      const open = () => { menu.classList.add("open"); btn.setAttribute("aria-expanded", "true"); };
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        if (menu.classList.contains("open")) close(); else open();
      });
      menu.addEventListener("click", (e) => {
        const b = e.target.closest("button[data-lang]");
        if (!b) return;
        const lang = b.getAttribute("data-lang");
        try { localStorage.setItem(STORE, lang); } catch (err) {}
        apply(lang);
        close();
        btn.focus();
      });
      document.addEventListener("click", close);
      document.addEventListener("keydown", (e) => { if (e.key === "Escape") close(); });
    }
  }

  window.i18n = { init, apply, t, get current() { return current; }, order: ORDER, labels: LABELS };
})();
