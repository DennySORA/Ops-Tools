# DevOps ツールセット

DevOps ワークフローの効率化のために設計された、堅牢な Rust 製 CLI ツールセットです。

[English](../README.md) | [繁體中文](README_zh-TW.md) | [简体中文](README_zh-CN.md)

## 機能

### 1. Terraform/Terragrunt キャッシュクリーナー
Terraform および Terragrunt によって生成されたキャッシュファイルをインテリジェントに削除します：
- `.terragrunt-cache` ディレクトリ
- `.terraform` ディレクトリ
- `.terraform.lock.hcl` ファイル
- 重複するサブパスを自動的に除外し、冗長な削除を回避します。

### 2. AI コードアシスタントアップグレーダー
人気のある AI コードアシスタントツールを一括アップグレードします：
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex)
- `Google Gemini CLI` (@google/gemini-cli)

### 3. 高リスクパッケージスキャナー
プロジェクト内のパッケージ依存関係を高性能で並列スキャンします：
- 任意のパッケージ名検索をサポート。
- `node_modules`、`.git`、`target` などのディレクトリを無視。
- バイナリファイルを自動的にスキップ。
- 一致するファイルパスと行番号を表示。

### 4. MCP ツール管理
Claude、Codex、Gemini CLI 用の MCP サーバーを管理します：

| MCP ツール | 説明 |
|------------|------|
| `sequential-thinking` | シーケンシャル・シンキング |
| `context7` | ドキュメント検索 |
| `chrome-devtools` | Chrome 開発者ツール |
| `kubernetes` | K8s 管理 |
| `github` | GitHub 統合 |

**必要な環境変数**（ビルド時に `.env` で設定）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `GITHUB_HOST`
- `CONTEXT7_API_KEY`

## インストール

### インストールスクリプト経由 (推奨 - Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Tool-Package/main/install.sh | bash
```

### 手動インストール

[Releases](https://github.com/DennySORA/Tool-Package/releases) ページから、お使いのシステムに対応するバージョンをダウンロードしてください。

### ソースコードからのビルド

```bash
# ビルド
cargo build --release

# 環境変数の設定 (オプション、MCP 管理機能を使用する場合)
cp .env.example .env
# .env を編集して認証情報を入力
```

## 使用方法

スクリプト経由でインストールした場合は、直接実行できます：

```bash
ops-tools
```

ソースコードからビルドした場合：

```bash
cargo run
# または
./target/release/tools
```

機能メニューを選択してください：
1. Terraform/Terragrunt キャッシュファイルのクリーニング
2. AI コードアシスタントツールのアップグレード
3. 高リスクパッケージのスキャン (セキュリティチェック)
4. MCP ツールの管理 (Claude/Codex/Gemini)

## 貢献

Pull Request や Issue の作成は大歓迎です！

## ライセンス

MIT License
