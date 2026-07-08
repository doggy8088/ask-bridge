/* i18n — zh-TW (default) · zh-CN · ja · ko · en */
(function () {
  "use strict";

  const D = {
    "zh-TW": {
      "nav.skip": "跳至主內容", "nav.why": "為何而做", "nav.features": "功能",
      "nav.usage": "用法", "nav.how": "原理", "nav.install": "安裝",
      "hero.kicker": "給 AI Agent 用的研究橋接器",
      "hero.title": "把終端機，\n接上 ChatGPT 與 Gemini",
      "hero.sub": "ask-bridge 以 Rust 驅動真實 Chrome，自動把 prompt 送進網站型 AI，再把回覆取回你的終端機。讓主要 Coding Agent 專注在高價值工作，把探索性研究交給網站額度。",
      "hero.ctaInstall": "一鍵安裝", "hero.ctaSource": "查看原始碼",
      "hero.termResp": "Rust 的所有權模型：每個值有唯一所有者，離開作用域即自動釋放⋯（回覆自動取回終端機）",
      "why.kicker": "為何而做", "why.title": "不是取代，而是橋接",
      "why.lead": "開發過程中有大量探索性 AI 需求——查資料、整理文件、比較方案、摘要錯誤、產生初稿。這些任務未必需要主要 Coding Agent 親自做，也不值得消耗與改程式、跑測試相同的額度。",
      "why.p1.t": "額度分開計算", "why.p1.d": "網站型 AI 的使用額度與 Coding Agent 執行額度通常獨立。把研究工作委外，讓高價值額度留給程式碼。",
      "why.p2.t": "不離開工作流程", "why.p2.d": "不必切換瀏覽器、貼上 prompt、等待、再複製回來。一行指令完成終端機與網站型 AI 的往返。",
      "why.p3.t": "真實瀏覽器，持久登入", "why.p3.d": "在真實 Chrome 中執行，只需登入一次。可處理 CAPTCHA、Cloudflare，像一般使用者存取網站功能。",
      "features.kicker": "功能", "features.title": "一行指令，完整橋接",
      "features.f1.t": "多 Provider 切換", "features.f1.d": "以 --provider 在 ChatGPT 與 Gemini 之間切換，或在設定檔指定預設 provider。",
      "features.f2.t": "Pipe 與 stdin", "features.f2.d": "把檔案內容或管線文字直接餵進 prompt，無需離開終端機。",
      "features.f3.t": "圖片與文件附件", "features.f3.d": "--image 附上圖片、--file 附上 PDF／Word／CSV／程式碼等文件。",
      "features.f4.t": "模型切換", "features.f4.d": "送出前以 --model 切換 ChatGPT 的 GPT-5.4、o3 或 Gemini 的 3.1 Pro。",
      "features.f5.t": "智慧分頁 · 安靜預設", "features.f5.d": "重用、聚焦或開新分頁避免分頁氾濫；預設只輸出回覆，--verbose 顯示完整流程。",
      "usage.kicker": "快速開始", "usage.title": "三分鐘上手",
      "usage.s1.t": "1 · 登入", "usage.s1.d": "首次執行登入所選 provider，只需一次。",
      "usage.s2.t": "2 · 直接提問", "usage.s2.d": "把 prompt 作為參數傳入，回覆輸出到終端機。",
      "usage.s3.t": "3 · 管線與檔案", "usage.s3.d": "透過 pipe 餵入內容，或以 --file 附件上傳。",
      "usage.s4.t": "4 · 切換模型", "usage.s4.d": "送出前指定模型或思考強度。",
      "how.kicker": "運作原理", "how.title": "從指令到回覆，四個步驟",
      "how.s1.t": "瀏覽器初始化", "how.s1.d": "檢查 Chrome 是否監聽 debug port 9223，若無則以專屬 profile 目錄啟動背景 Chrome。",
      "how.s2.t": "MCP Bridge 設定", "how.s2.d": "自動寫入 mcp_servers.json，設定 chrome-devtools-mcp server 指向本機 9223。",
      "how.s3.t": "Client 呼叫", "how.s3.d": "透過內建 mcp-cli 呼叫 list_pages、select_page、type_text、evaluate_script 等工具。",
      "how.s4.t": "狀態輪詢", "how.s4.d": "以 JavaScript 檢查送出／停止按鈕狀態，擷取回覆元素文字並輸出到 stdout。",
      "install.kicker": "安裝", "install.title": "馬上開始",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "從原始碼", "install.t4": "Agent Skill",
      "cta.title": "讓你的 Agent 多一條研究通路", "cta.sub": "把探索性工作交給網站型 AI，把高價值額度留給真正改變專案的程式碼。",
      "footer.tagline": "給 AI Agent 用的外部研究橋接器",
      "footer.github": "GitHub", "footer.docs": "文件", "footer.issues": "問題回報",
      "footer.by": "由", "footer.crafted": "打造",
      "nav.toDark": "切換至深色模式", "nav.toLight": "切換至淺色模式",
      "nav.copy": "複製", "nav.copied": "已複製", "nav.langLabel": "選擇語言",
      "install.tabsLabel": "安裝方式",
      "doc.title": "ask-bridge — 把終端機接上 ChatGPT 與 Gemini",
      "doc.description": "ask-bridge 是以 Rust 撰寫的命令列研究橋接器，透過真實 Chrome 自動操作 ChatGPT 或 Gemini，把 prompt 送進網站、把回覆取回終端機。"
    },
    "zh-CN": {
      "nav.skip": "跳至主要内容", "nav.why": "为何而做", "nav.features": "功能",
      "nav.usage": "用法", "nav.how": "原理", "nav.install": "安装",
      "hero.kicker": "给 AI Agent 用的研究桥接器",
      "hero.title": "把终端机，\n接上 ChatGPT 与 Gemini",
      "hero.sub": "ask-bridge 以 Rust 驱动真实 Chrome，自动把 prompt 送进网站型 AI，再把回复取回你的终端机。让主要 Coding Agent 专注在高价值工作，把探索性研究交给网站额度。",
      "hero.ctaInstall": "一键安装", "hero.ctaSource": "查看源代码",
      "hero.termResp": "Rust 的所有权模型：每个值有唯一所有者，离开作用域即自动释放⋯（回复自动取回终端机）",
      "why.kicker": "为何而做", "why.title": "不是取代，而是桥接",
      "why.lead": "开发过程中有大量探索性 AI 需求——查资料、整理文档、比较方案、摘要错误、生成初稿。这些任务未必需要主要 Coding Agent 亲自做，也不值得消耗与改代码、跑测试相同的额度。",
      "why.p1.t": "额度分开计算", "why.p1.d": "网站型 AI 的使用额度与 Coding Agent 执行额度通常独立。把研究工作委外，让高价值额度留给代码。",
      "why.p2.t": "不离开工作流程", "why.p2.d": "不必切换浏览器、粘贴 prompt、等待、再复制回来。一行指令完成终端机与网站型 AI 的往返。",
      "why.p3.t": "真实浏览器，持久登录", "why.p3.d": "在真实 Chrome 中执行，只需登录一次。可处理 CAPTCHA、Cloudflare，像一般用户访问网站功能。",
      "features.kicker": "功能", "features.title": "一行指令，完整桥接",
      "features.f1.t": "多 Provider 切换", "features.f1.d": "以 --provider 在 ChatGPT 与 Gemini 之间切换，或在配置文件指定默认 provider。",
      "features.f2.t": "Pipe 与 stdin", "features.f2.d": "把文件内容或管道文字直接喂进 prompt，无需离开终端机。",
      "features.f3.t": "图片与文档附件", "features.f3.d": "--image 附上图片、--file 附上 PDF／Word／CSV／代码等文档。",
      "features.f4.t": "模型切换", "features.f4.d": "提交前以 --model 切换 ChatGPT 的 GPT-5.4、o3 或 Gemini 的 3.1 Pro。",
      "features.f5.t": "智能标签页 · 安静默认", "features.f5.d": "复用、聚焦或开新标签页避免泛滥；默认只输出回复，--verbose 显示完整流程。",
      "usage.kicker": "快速开始", "usage.title": "三分钟上手",
      "usage.s1.t": "1 · 登录", "usage.s1.d": "首次执行登录所选 provider，只需一次。",
      "usage.s2.t": "2 · 直接提问", "usage.s2.d": "把 prompt 作为参数传入，回复输出到终端机。",
      "usage.s3.t": "3 · 管道与文件", "usage.s3.d": "通过 pipe 喂入内容，或以 --file 附件上传。",
      "usage.s4.t": "4 · 切换模型", "usage.s4.d": "提交前指定模型或思考强度。",
      "how.kicker": "运作原理", "how.title": "从指令到回复，四个步骤",
      "how.s1.t": "浏览器初始化", "how.s1.d": "检查 Chrome 是否监听 debug port 9223，若无则以专属 profile 目录启动后台 Chrome。",
      "how.s2.t": "MCP Bridge 设置", "how.s2.d": "自动写入 mcp_servers.json，设置 chrome-devtools-mcp server 指向本机 9223。",
      "how.s3.t": "Client 调用", "how.s3.d": "通过内置 mcp-cli 调用 list_pages、select_page、type_text、evaluate_script 等工具。",
      "how.s4.t": "状态轮询", "how.s4.d": "以 JavaScript 检查提交／停止按钮状态，截取回复元素文字并输出到 stdout。",
      "install.kicker": "安装", "install.title": "马上开始",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "从源代码", "install.t4": "Agent Skill",
      "cta.title": "让你的 Agent 多一条研究通路", "cta.sub": "把探索性工作交给网站型 AI，把高价值额度留给真正改变项目的代码。",
      "footer.tagline": "给 AI Agent 用的外部研究桥接器",
      "footer.github": "GitHub", "footer.docs": "文档", "footer.issues": "问题反馈",
      "footer.by": "由", "footer.crafted": "打造",
      "nav.toDark": "切换至深色模式", "nav.toLight": "切换至浅色模式",
      "nav.copy": "复制", "nav.copied": "已复制", "nav.langLabel": "选择语言",
      "install.tabsLabel": "安装方式",
      "doc.title": "ask-bridge — 把终端机接上 ChatGPT 与 Gemini",
      "doc.description": "ask-bridge 是以 Rust 编写的命令列研究桥接器，透过真实 Chrome 自动操作 ChatGPT 或 Gemini，把 prompt 送进网站、把回复取回终端机。"
    },
    "ja": {
      "nav.skip": "メインへスキップ", "nav.why": "目的", "nav.features": "機能",
      "nav.usage": "使い方", "nav.how": "仕組み", "nav.install": "インストール",
      "hero.kicker": "AI Agent のための研究ブリッジ",
      "hero.title": "ターミナルを、\nChatGPT と Gemini に接続",
      "hero.sub": "ask-bridge は Rust で実際の Chrome を駆動し、プロンプトをウェブ型 AI に自動送信して、返答をターミナルへ持ち帰ります。メインの Coding Agent は高価値な作業に集中し、探索的な研究をウェブの利用枠に任せます。",
      "hero.ctaInstall": "ワンライン install", "hero.ctaSource": "ソースを見る",
      "hero.termResp": "Rust の所有権モデル：各値には唯一の所有者がおり、スコープを抜けると自動で解放されます⋯（返答は自動でターミナルに戻ります）",
      "why.kicker": "目的", "why.title": "置き換えではなく、橋渡し",
      "why.lead": "開発中には探索的な AI 需要が大量に生じます——調査、整理、比較、要約、下書き。これらはメインの Coding Agent が自らやる必要はないケースが多く、コード編集やテストと同じ枠を使う値打ちもありません。",
      "why.p1.t": "利用枠を分離", "why.p1.d": "ウェブ型 AI の利用枠と Coding Agent の実行枠は概ね独立。研究を委託し、高価値な枠はコードに残します。",
      "why.p2.t": "ワークフローから離れない", "why.p2.d": "ブラウザ切替・貼り付け・待機・コピー戻しをせず、一行でターミナルとウェブ型 AI の往復を完結します。",
      "why.p3.t": "実ブラウザ、永続ログイン", "why.p3.d": "実 Chrome で実行し、ログインは一度だけ。CAPTCHA や Cloudflare も一般ユーザーと同様に処理できます。",
      "features.kicker": "機能", "features.title": "一行で、完全なブリッジ",
      "features.f1.t": "マルチ Provider 切替", "features.f1.d": "--provider で ChatGPT と Gemini を切り替え、設定ファイルで既定 provider も指定可能。",
      "features.f2.t": "Pipe と stdin", "features.f2.d": "ファイル内容やパイプのテキストを直接プロンプトに流し、ターミナルから離れません。",
      "features.f3.t": "画像・ファイル添付", "features.f3.d": "--image で画像、--file で PDF／Word／CSV／コード等を添付。",
      "features.f4.t": "モデル切替", "features.f4.d": "送信前に --model で ChatGPT の GPT-5.4・o3、Gemini の 3.1 Pro などを指定。",
      "features.f5.t": "スマートタブ・静音 既定", "features.f5.d": "タブの再利用・フォーカス・新規開封で煩雑化を防ぎ、既定は返答のみ、--verbose で全過程を表示。",
      "usage.kicker": "クイックスタート", "usage.title": "3 分で始める",
      "usage.s1.t": "1 · ログイン", "usage.s1.d": "初回は選んだ provider にログイン。一度だけ。",
      "usage.s2.t": "2 · 直接質問", "usage.s2.d": "プロンプトを引数で渡すと、返答がターミナルに出力されます。",
      "usage.s3.t": "3 · パイプとファイル", "usage.s3.d": "pipe で内容を流し込むか、--file で添付します。",
      "usage.s4.t": "4 · モデル切替", "usage.s4.d": "送信前にモデルや思考強度を指定します。",
      "how.kicker": "仕組み", "how.title": "コマンドから返答まで、4 ステップ",
      "how.s1.t": "ブラウザ初期化", "how.s1.d": "Chrome が debug port 9223 を監聴しているか確認し、無ければ専用 profile でバックグラウンド Chrome を起動。",
      "how.s2.t": "MCP Bridge 設定", "how.s2.d": "mcp_servers.json を自動書き込み、chrome-devtools-mcp server を localhost:9223 に向けます。",
      "how.s3.t": "Client 呼び出し", "how.s3.d": "内蔵 mcp-cli で list_pages・select_page・type_text・evaluate_script などを呼び出します。",
      "how.s4.t": "状態ポーリング", "how.s4.d": "JavaScript で送信／停止ボタン状態を確認し、返答要素のテキストを stdout へ出力。",
      "install.kicker": "インストール", "install.title": "今すぐ始める",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "ソースから", "install.t4": "Agent Skill",
      "cta.title": "Agent にもう一本、研究の道を", "cta.sub": "探索的な作業はウェブ型 AI に、高価値な枠はプロジェクトを動かすコードに。",
      "footer.tagline": "AI Agent のための外部研究ブリッジ",
      "footer.github": "GitHub", "footer.docs": "ドキュメント", "footer.issues": "Issue",
      "footer.by": "制作：", "footer.crafted": "",
      "nav.toDark": "ダークモードへ切り替え", "nav.toLight": "ライトモードへ切り替え",
      "nav.copy": "コピー", "nav.copied": "コピー済み", "nav.langLabel": "言語を選択",
      "install.tabsLabel": "インストール方法",
      "doc.title": "ask-bridge — ターミナルを ChatGPT と Gemini に接続",
      "doc.description": "ask-bridge は Rust 製の CLI 研究ブリッジ。実 Chrome を自動操作し、ChatGPT または Gemini に prompt を送り、返答をターミナルに取り戻します。"
    },
    "ko": {
      "nav.skip": "본문으로 건너뛰기", "nav.why": "목적", "nav.features": "기능",
      "nav.usage": "사용법", "nav.how": "원리", "nav.install": "설치",
      "hero.kicker": "AI Agent 를 위한 연구 브릿지",
      "hero.title": "터미널을,\nChatGPT 와 Gemini 에 연결",
      "hero.sub": "ask-bridge 는 Rust 로 실제 Chrome 을 구동해 프로ンプ트를 웹형 AI 에 자동 전송하고, 답변을 터미널로 가져옵니다. 메인 Coding Agent 는 고가치 작업에 집중하고, 탐색적 연구는 웹 사용량에 맡깁니다.",
      "hero.ctaInstall": "원클릭 설치", "hero.ctaSource": "소스 보기",
      "hero.termResp": "Rust 의 소유권 모델: 각 값은 유일한 소유자를 가지며, 스코프를 벗어나면 자동 해제됩니다⋯(답변이 자동으로 터미널로 돌아옵니다)",
      "why.kicker": "목적", "why.title": "대체가 아니라 연결",
      "why.lead": "개발 중에는 탐색적 AI 수요가 많습니다——조사, 정리, 비교, 요약, 초안. 이런 작업은 메인 Coding Agent 가 직접 할 필요가 없고, 코드 수정·테스트와 같은 한도를 쓸 만큼의 가치도 아닐 때가 많습니다.",
      "why.p1.t": "사용량 분리", "why.p1.d": "웹형 AI 사용량과 Coding Agent 실행량은 보통 독립적. 연구를 위임하고 고가치 한도는 코드에 남깁니다.",
      "why.p2.t": "워크플로를 떠나지 않기", "why.p2.d": "브라우저 전환·붙여넣기·대기·복사 돌아오기 없이, 한 줄로 터미널과 웹형 AI 의 왕복을 끝냅니다.",
      "why.p3.t": "실제 브라우저, 영구 로그인", "why.p3.d": "실제 Chrome 에서 실행, 로그인은 한 번만. CAPTCHA 와 Cloudflare 도 일반 사용자처럼 처리합니다.",
      "features.kicker": "기능", "features.title": "한 줄로, 완전한 브릿지",
      "features.f1.t": "멀티 Provider 전환", "features.f1.d": "--provider 로 ChatGPT 와 Gemini 를 전환, 설정 파일로 기본 provider 지정.",
      "features.f2.t": "Pipe 와 stdin", "features.f2.d": "파일 내용이나 파이프 텍스트를 프로ンプ트에 직접 넣고 터미널을 떠나지 않습니다.",
      "features.f3.t": "이미지·파일 첨부", "features.f3.d": "--image 로 이미지, --file 로 PDF／Word／CSV／코드 등 첨부.",
      "features.f4.t": "모델 전환", "features.f4.d": "전송 전 --model 로 ChatGPT 의 GPT-5.4·o3, Gemini 의 3.1 Pro 지정.",
      "features.f5.t": "스마트 탭 · 조용한 기본", "features.f5.d": "탭 재사용·포커스·신규 열기로 탭 난립을 막고, 기본은 답변만, --verbose 로 전 과정 표시.",
      "usage.kicker": "빠른 시작", "usage.title": "3 분 시작",
      "usage.s1.t": "1 · 로그인", "usage.s1.d": "처음에 선택한 provider 에 로그인. 한 번만.",
      "usage.s2.t": "2 · 직접 질문", "usage.s2.d": "프롬프트를 인수로 넘기면 답변이 터미널에 출력됩니다.",
      "usage.s3.t": "3 · 파이프와 파일", "usage.s3.d": "pipe 로 내용을 넣거나 --file 로 첨부합니다.",
      "usage.s4.t": "4 · 모델 전환", "usage.s4.d": "전송 전 모델이나 사고 강도를 지정합니다.",
      "how.kicker": "원리", "how.title": "명령에서 답변까지, 4 단계",
      "how.s1.t": "브라우저 초기화", "how.s1.d": "Chrome 이 debug port 9223 을 듣는지 확인, 없으면 전용 profile 로 백그라운드 Chrome 시작.",
      "how.s2.t": "MCP Bridge 설정", "how.s2.d": "mcp_servers.json 을 자동 작성, chrome-devtools-mcp server 를 localhost:9223 으로 지정.",
      "how.s3.t": "Client 호출", "how.s3.d": "내장 mcp-cli 로 list_pages·select_page·type_text·evaluate_script 등 호출.",
      "how.s4.t": "상태 폴링", "how.s4.d": "JavaScript 로 전송／정지 버튼 상태를 확인, 답변 요소 텍스트를 stdout 으로 출력.",
      "install.kicker": "설치", "install.title": "지금 시작",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "소스에서", "install.t4": "Agent Skill",
      "cta.title": "Agent 에 연구 길 하나 더", "cta.sub": "탐색적 작업은 웹형 AI 에, 고가치 한도는 프로젝트를 움직이는 코드에.",
      "footer.tagline": "AI Agent 를 위한 외부 연구 브릿지",
      "footer.github": "GitHub", "footer.docs": "문서", "footer.issues": "이슈",
      "footer.by": "제작：", "footer.crafted": "",
      "nav.toDark": "다크 모드로 전환", "nav.toLight": "라이트 모드로 전환",
      "nav.copy": "복사", "nav.copied": "복사됨", "nav.langLabel": "언어 선택",
      "install.tabsLabel": "설치 방법",
      "doc.title": "ask-bridge — 터미널을 ChatGPT 와 Gemini 에 연결",
      "doc.description": "ask-bridge 는 Rust 로 작성된 CLI 연구 브릿지입니다. 실제 Chrome 을 자동 조작해 ChatGPT 또는 Gemini 에 prompt 를 보내고 답변을 터미널로 가져옵니다."
    },
    "en": {
      "nav.skip": "Skip to content", "nav.why": "Why", "nav.features": "Features",
      "nav.usage": "Usage", "nav.how": "How", "nav.install": "Install",
      "hero.kicker": "A research bridge for AI agents",
      "hero.title": "Wire your terminal \nto ChatGPT and Gemini",
      "hero.sub": "ask-bridge drives a real Chrome browser in Rust, ships your prompt to a web-based AI, and brings the answer back to your terminal. Let your main coding agent focus on high-value work, and route exploratory research to your web quota.",
      "hero.ctaInstall": "One-line install", "hero.ctaSource": "View source",
      "hero.termResp": "Rust's ownership model: every value has a single owner and is freed automatically when it goes out of scope… (the reply is returned to your terminal)",
      "why.kicker": "Why", "why.title": "A bridge, not a replacement",
      "why.lead": "Development is full of exploratory AI needs: research, summarizing, comparing options, digesting errors, drafting. These rarely need your main coding agent, and rarely justify spending the same budget reserved for editing code and running tests.",
      "why.p1.t": "Separate quotas", "why.p1.d": "Web-AI usage quotas and coding-agent execution budgets are usually independent. Delegate research, save the high-value budget for code.",
      "why.p2.t": "Stay in your workflow", "why.p2.d": "No switching to a browser, pasting a prompt, waiting, copying back. One command round-trips the terminal and the web AI.",
      "why.p3.t": "Real browser, persistent login", "why.p3.d": "Runs in a real Chrome, log in once. Handles CAPTCHA and Cloudflare like any normal user.",
      "features.kicker": "Features", "features.title": "One command, the full bridge",
      "features.f1.t": "Multi-provider switch", "features.f1.d": "Switch ChatGPT and Gemini with --provider, or set a default in the config file.",
      "features.f2.t": "Pipe & stdin", "features.f2.d": "Feed file contents or piped text straight into the prompt without leaving the terminal.",
      "features.f3.t": "Image & file attachments", "features.f3.d": "Attach images with --image and PDF/Word/CSV/code with --file.",
      "features.f4.t": "Model switching", "features.f4.d": "Switch to ChatGPT's GPT-5.4, o3, or Gemini's 3.1 Pro before submitting.",
      "features.f5.t": "Smart tabs · quiet by default", "features.f5.d": "Reuse, focus, or open new tabs to avoid clutter; defaults to the reply only, --verbose shows the full flow.",
      "usage.kicker": "Quick start", "usage.title": "Up in three minutes",
      "usage.s1.t": "1 · Log in", "usage.s1.d": "Log in to your chosen provider the first time. Once.",
      "usage.s2.t": "2 · Ask directly", "usage.s2.d": "Pass the prompt as an argument; the reply prints to your terminal.",
      "usage.s3.t": "3 · Pipe & files", "usage.s3.d": "Feed content via pipe, or upload with --file.",
      "usage.s4.t": "4 · Switch model", "usage.s4.d": "Pick a model or thinking level before submitting.",
      "how.kicker": "How it works", "how.title": "From command to reply, four steps",
      "how.s1.t": "Browser init", "how.s1.d": "Checks whether Chrome is listening on debug port 9223; if not, launches a background Chrome with a dedicated profile.",
      "how.s2.t": "MCP bridge config", "how.s2.d": "Writes mcp_servers.json automatically, pointing chrome-devtools-mcp at localhost:9223.",
      "how.s3.t": "Client calls", "how.s3.d": "Uses the embedded mcp-cli to call list_pages, select_page, type_text, evaluate_script, and more.",
      "how.s4.t": "State polling", "how.s4.d": "JavaScript checks the send/stop button states and streams the reply text to stdout.",
      "install.kicker": "Install", "install.title": "Get started now",
      "install.t1": "macOS / Linux", "install.t2": "Windows", "install.t3": "From source", "install.t4": "Agent Skill",
      "cta.title": "Give your agent another research path", "cta.sub": "Route exploratory work to web AI, and keep the high-value budget for the code that actually moves the project.",
      "footer.tagline": "An external research bridge for AI agents",
      "footer.github": "GitHub", "footer.docs": "Docs", "footer.issues": "Issues",
      "footer.by": "Built by ", "footer.crafted": "",
      "nav.toDark": "Switch to dark mode", "nav.toLight": "Switch to light mode",
      "nav.copy": "Copy", "nav.copied": "Copied", "nav.langLabel": "Choose language",
      "install.tabsLabel": "Install method",
      "doc.title": "ask-bridge — Wire your terminal to ChatGPT and Gemini",
      "doc.description": "ask-bridge is a Rust CLI research bridge that drives real Chrome to automate ChatGPT or Gemini, sending prompts in and bringing replies back to your terminal."
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