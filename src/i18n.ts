import i18n from "i18next";
import { initReactI18next } from "react-i18next";

const englishTranslations = {
  "インデックス": "Indexes",
  "MCPサーバー": "MCP server",
  "設定": "Settings",
  "ダークモードを切り替え": "Toggle dark mode",
  "言語": "Language",
  "エラー": "Error",
  "確認": "Confirm",
  "fileSource": "File source",
  "fileExtractor": "File extractor",
  "embedder": "Embedder",
  "vectorStore": "Vector store",
  "各種設定を閲覧・変更します": "View and change application settings",
  "設定を読み込んでいます…": "Loading settings…",
  "設定を取得できませんでした。": "Failed to load settings.",
  "ローカルデータ": "Local data",
  "SQLite データベース": "SQLite database",
  "ログディレクトリ": "Log directory",
  "プラグイン": "Plugins",
  "{{count}} 件": "{{count}} items",
  "{{count}} 件_one": "{{count}} item",
  "{{count}} 件_other": "{{count}} items",
  "アプリで利用している依存ライブラリのライセンス情報を確認できます。":
    "View license information for libraries used by the application.",
  "ライセンス一覧を見る": "View licenses",
  "インデックスを読み込んでいます…": "Loading indexes…",
  "インデックスを取得できませんでした。": "Failed to load indexes.",
  "インデックスを取得できませんでした: {{error}}":
    "Failed to load indexes: {{error}}",
  "インデックスがありません。先にインデックスを作成してください。":
    "No indexes found. Create an index first.",
  "ファイルソースと検索設定を管理します":
    "Manage file sources and search settings",
  "＋ 新しいインデックス": "+ New index",
  "並び替え": "Sort by",
  "名前": "Name",
  "最終実行日時": "Last run",
  "{{count}} 件のインデックス_one": "{{count}} index",
  "{{count}} 件のインデックス_other": "{{count}} indexes",
  "インデックス名を変更しました": "Index name updated.",
  "インデックスの説明を変更しました": "Index description updated.",
  "ベクトル化プラグインを変更しました": "Embedder plugin updated.",
  "ベクトルストアの変更は索引をクリアする必要があります、よろしいですか？":
    "Changing the vector store requires clearing the index. Continue?",
  "ベクトルストアを変更しました": "Vector store updated.",
  "基本情報": "Basic information",
  "インデックス名": "Index name",
  "情報がありません": "No information",
  "インデックスの詳細情報": "Index description",
  "インデックス定義を削除します、よろしいですか？":
    "Delete this index definition?",
  "インデックス定義を削除": "Delete index definition",
  "索引をクリアします、よろしいですか？": "Clear the index?",
  "索引をクリア": "Clear index",
  "スケジュールを保存しました": "Schedule saved.",
  "定期実行スケジュールを削除します、よろしいですか？":
    "Delete the scheduled run?",
  "スケジュールを削除しました": "Schedule deleted.",
  "ファイルソース：ファイルの取得元です":
    "File source: where files are retrieved from",
  "ファイルソースを保存しました": "File source saved.",
  "ファイル抽出：ファイルを抽出します": "File extractor: extracts file content",
  "ファイル抽出を変更しました": "File extractor updated.",
  "ベクトル化：ファイルを検索可能に変換します":
    "Embedder: converts files into searchable vectors",
  "ベクトルストア：生成したベクトルを保存し、検索に使います":
    "Vector store: stores generated vectors for search",
  "インデクシングを実行します、よろしいですか？":
    "Start indexing now?",
  "今すぐ実行": "Run now",
  "インデックス内のファイルを閲覧し、メタデータを確認します":
    "Browse files in an index and inspect their metadata",
  "インデックスを選択": "Select an index",
  "説明はありません": "No description",
  "キーワード検索": "Keyword search",
  "意味検索": "Semantic search",
  "テキスト": "Text",
  "画像": "Image",
  "検索範囲": "Search scope",
  "このフォルダ ({{path}})": "This folder ({{path}})",
  "インデックス全体": "Entire index",
  "画像を選択": "Choose image",
  "選択した画像を解除": "Remove selected image",
  "画像を選択してください": "Choose an image",
  "検索語": "Search query",
  "本文から探すキーワードを入力": "Enter keywords to find in file contents",
  "例: 契約更新の条件について書かれた文書":
    "e.g. A document describing the terms for contract renewal",
  "画像: {{name}}": "Image: {{name}}",
  "{{mode}}： 「{{query}}」": "{{mode}}: “{{query}}”",
  "{{kind}}: {{name}}": "{{kind}}: {{name}}",
  "検索": "Search",
  "本文に入力したキーワードを含む箇所を検索します。":
    "Find matches for the entered keywords in file contents.",
  "画像の内容に近いファイルを検索します。":
    "Find files with content similar to the image.",
  "言葉が完全に一致しなくても、内容が近い箇所を検索します。":
    "Find semantically similar content even when the wording differs.",
  "対象:": "In:",
  "一覧に戻る": "Back to file list",
  "上の階層へ": "Go to parent folder",
  "検索中…": "Searching…",
  "検索しています…": "Searching…",
  "検索できませんでした: {{error}}": "Search failed: {{error}}",
  "該当するファイルはありません": "No matching files found.",
  "ファイル一覧を読み込んでいます…": "Loading files…",
  "ファイル一覧を取得できませんでした: {{error}}":
    "Failed to load files: {{error}}",
  "最終更新日": "Last modified",
  "サイズ": "Size",
  "種類": "Type",
  "このフォルダは空です": "This folder is empty.",
  "ファイル情報": "File details",
  "ファイル情報を表示: {{name}}": "Show file details: {{name}}",
  "フォルダ": "Folder",
  "ファイル": "File",
  "昇順": "ascending",
  "降順": "descending",
  "{{label}}で並べ替え{{direction}}": "Sort by {{label}}{{direction}}",
  "メタデータを読み込んでいます…": "Loading metadata…",
  "メタデータを取得できませんでした: {{error}}":
    "Failed to load metadata: {{error}}",
  "コンテンツタイプ": "Content type",
  "更新日時": "Last modified",
  "不明": "Unknown",
  "ファイル URI": "File URI",
  "ファイルハッシュ": "File hash",
  "ダウンロード": "Download",
  "ファイルを選択するとメタデータが表示されます。":
    "Select a file to view its metadata.",
  "不明なエラー": "Unknown error",
  "ファイルをダウンロードしました": "File downloaded.",
  "ファイルをダウンロードできませんでした: {{error}}":
    "Failed to download file: {{error}}",
  "画像ファイルを選択してください。": "Please select an image file.",
  "画像ファイルを読み込めませんでした。": "Could not read the image file.",
  "タスク": "Tasks",
  "現在実行中のインデックス処理を表示します":
    "View currently running indexing jobs",
  "実行中 {{count}} 件_one": "{{count}} task running",
  "実行中 {{count}} 件_other": "{{count}} tasks running",
  "実行中のタスクを読み込んでいます…": "Loading active tasks…",
  "タスクを取得できませんでした。": "Failed to load tasks.",
  "現在実行中のタスクはありません。": "There are no active tasks.",
  "実行中": "Running",
  "待機中": "Queued",
  "開始:": "Started:",
  "進捗情報なし": "No progress information",
  "MCP から公開するインデックスとローカルサーバーを管理します":
    "Manage indexes exposed through MCP and the local server",
  "MCP サーバーの状態を読み込んでいます…": "Loading MCP server status…",
  "MCP サーバーの情報を取得できませんでした。":
    "Failed to load MCP server information.",
  "稼働中": "Running",
  "停止中": "Stopped",
  "起動:": "Started:",
  "公開中のインデックス": "Exposed indexes",
  "ステータス": "Status",
  "サーバーを停止": "Stop server",
  "サーバーを起動": "Start server",
  "待受ポート": "Listening port",
  "MCP クライアントが接続するローカルポートを選択します。":
    "Choose the local port for MCP clients to connect to.",
  "ポート": "Port",
  "公開するインデックス": "Indexes to expose",
  "有効なインデックスだけを MCP から検索できます":
    "Only enabled indexes can be searched through MCP",
  "{{name}} を MCP から公開": "Expose {{name}} through MCP",
  "公開中": "Exposed",
  "非公開": "Hidden",
  "オープンソースライセンス": "Open-source licenses",
  "アプリで利用している npm パッケージと Rust クレートのライセンス表記です。":
    "License notices for npm packages and Rust crates used by the application.",
  "ライセンス一覧を読み込んでいます…": "Loading licenses…",
  "ライセンス一覧を取得できませんでした。":
    "Failed to load licenses.",
  "依存ライブラリを検索": "Search dependencies",
  "名前、バージョン、ライセンス": "Name, version, or license",
  "{{shown}} / {{total}} 件": "{{shown}} of {{total}} dependencies",
  "該当するライブラリはありません。": "No matching dependencies found.",
  "ライセンス一覧の取得に失敗しました: {{status}}":
    "Failed to load licenses: {{status}}",
  "ライセンス本文をパッケージから取得できませんでした（{{license}}）。":
    "Could not retrieve the license text from the package ({{license}}).",
  "OAuth設定": "OAuth settings",
  "ファイルソースのためのOAuth認証（PKCE）を設定します":
    "Configure OAuth authentication (PKCE) for file sources",
  "＋ 新しいOAuth": "+ New OAuth client",
  "空のOAuthを追加しました": "Added an empty OAuth client.",
  "OAuthを更新しました": "OAuth client updated.",
  "OAuthを削除しました": "OAuth client deleted.",
  "クライアントID": "Client ID",
  "クライアントシークレット": "Client secret",
  "認可エンドポイント": "Authorization endpoint",
  "トークンエンドポイント": "Token endpoint",
  "リフレッシュエンドポイント": "Refresh endpoint",
  "失効エンドポイント": "Revocation endpoint",
  "スコープ": "Scopes",
  "保存": "Save",
  "削除": "Delete",
  "中断": "Cancel",
  "テスト": "Test",
  "OAuthを削除します、よろしいですか？": "Delete this OAuth client?",
  "認証に成功しました": "Authentication succeeded.",
  "OAuthクライアント": "OAuth client",
  "認証を中断": "Cancel authentication",
  "OAuth認証": "OAuth authentication",
  "ファイルソースのテストに成功しました":
    "File source test succeeded.",
  "OAuth認証に成功しました": "OAuth authentication succeeded.",
  "通知": "Notification",
  "保存できませんでした。": "Failed to save.",
  "キャンセル": "Cancel",
  "変更": "Edit",
  "定期実行スケジュール": "Recurring schedule",
  "時間おき": "Hourly",
  "日おき": "Daily",
  "週おき": "Weekly",
  "月おき": "Monthly",
  "年おき": "Yearly",
  "開始日": "Start date",
  "終了日": "End date",
  "頻度": "Frequency",
  "非起動時に予定を過ぎた場合、次回起動時に処理を開始する":
    "If a run is missed while the app is closed, start it when the app next opens",
  "RFC5545 RRULE 直接入力": "Enter RFC5545 RRULE directly",
  "リセット": "Reset",
  "実行予定日時プレビュー": "Upcoming run preview",
  "定期実行は未設定です": "No recurring schedule is configured",
  "空のインデクサを追加しました": "Added an empty index.",
} as const;

const japaneseTranslations = {
  fileSource: "ファイルソース",
  fileExtractor: "ファイル抽出",
  embedder: "ベクトル化",
  vectorStore: "ベクトルストア",
} as const;

const storedLanguage = localStorage.getItem("anydex.language");
const initialLanguage =
  storedLanguage === "ja" || storedLanguage === "en"
    ? storedLanguage
    : navigator.language.toLowerCase().startsWith("ja")
      ? "ja"
      : "en";

void i18n.use(initReactI18next).init({
  resources: {
    ja: { translation: japaneseTranslations },
    en: { translation: englishTranslations },
  },
  lng: initialLanguage,
  fallbackLng: "ja",
  keySeparator: false,
  nsSeparator: false,
  interpolation: { escapeValue: false },
});

i18n.on("languageChanged", (language) => {
  localStorage.setItem("anydex.language", language);
  document.documentElement.lang = language;
});

document.documentElement.lang = initialLanguage;

export default i18n;
