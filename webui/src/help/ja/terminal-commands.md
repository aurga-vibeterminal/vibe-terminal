クイックコマンドは、プロジェクトのビルド/テスト/デプロイコマンドをターミナルの右クリックメニューにピン留めします。ワンクリックで実行でき、どこからでも利用可能です。コマンドはリポジトリごとにリポジトリルートの `.vterm/commands.json` で管理されます。

## commands.json の作成

ターミナル内で右クリック → `Commands`:

- セッションのリポジトリにまだ commands.json がない場合、メニューには `Generate commands.json` という単一エントリが表示されます。クリックすると、サーバーがプロジェクトタイプ(npm / Cargo / .NET / Xcode / Gradle-Android / 汎用)を検出し、控えめなスターターテンプレートを書き込みます。既存の空でないファイルは**上書きされません**;
- ファイルの場所:Git リポジトリのルートの `.vterm/commands.json`(非 Git フォルダではカレントディレクトリ)。

## 調整

生成されたテンプレートには、検出されたプロジェクトタイプの定番コマンドのみが含まれます。コードエージェント(このサーバーの MCP に接続されたエディタエージェントなど)にプロジェクトを読ませてファイルを調整させましょう:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

トップレベル配列も受け付けられます。`env` はオプションでコマンド実行前に変数を設定します。旧 `value` キーも引き続き読み込まれます。オプションの `os` フィールドは、エントリを `vterm-server` が動作している OS に限定します。Windows には `"win"`、Linux には `"linux"`、macOS には `"osx"` を指定します。`os` を省略した場合、`"all"` を設定した場合、または認識できない値の場合は、エントリはすべてのプラットフォームで利用できます。サーバーは CWD と commands.json の変更を監視してブロードキャストします。メニューを開き直せば新しいコマンドが現れます。

## 実行

右クリック → `Commands` → エントリをクリックすると、そのターミナルで実行されます(オプションの env が先に適用されます)。セッションでエージェントアプリが実行中の間はメニューは利用できないため、コマンド文字列がアプリの UI 内に入り込むことはありません。

## 例:Wi-Fi 経由の Android ビルド&インストール

Gradle-Android プロジェクトでの完全なウォークスルー:ワイヤレス ADB のセットアップとビルド&インストールのフローを右クリックメニューにピン留めし、ワンクリックでアプリをリビルドします。

**1. スマホをワイヤレス ADB に切り替える。** これら 2 つのコマンドはデバイスごとに 1 回入力します。スマホが USB 接続中に `adb tcpip 5555`、その後スマホのアドレスで `adb connect`:

![ターミナル — adb tcpip 5555 と adb connect で Android スマホをワイヤレス ADB に切り替え](images/terminal-commands-adb-setup.gif)

**2. コードエージェントにコマンドを書かせる。** プロジェクトのコードエージェント(ここでは Claude Code)をターミナルで開き、ビルド&インストールコマンドの追加を依頼します。プロジェクト構成を読み、`.vterm/commands.json` にエントリを書き込みます:

![ターミナル — Claude Code が commands.json にビルド&インストール APK コマンドを編集](images/terminal-commands-agent-edit.mp4)

**3. メニューからビルド&インストール。** ターミナルを分割してビルド出力を専用ペインにし、右クリック → `Commands` → エントリ — Gradle が APK をアセンブルし、Wi-Fi 経由でスマホにインストールします:

![ターミナル — ペインを分割し右クリックメニューから Android ビルド&インストールコマンドを実行](images/terminal-commands-split-run.mp4)
