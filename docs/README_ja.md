# DevOps ツールセット

DevOps ワークフローの効率化のために設計された、堅牢な Rust 製 CLI ツールセットです。

[English](../README.md) | [繁體中文](README_zh-TW.md) | [简体中文](README_zh-CN.md)

![Screenshot](image/01.png)

## 機能一覧

| 機能 | 説明 |
|------|------|
| Terraform クリーンアップ | `.terraform`、`.terragrunt-cache` およびロックファイルを削除 |
| AI アシスタントアップグレード | Claude Code、Codex、Gemini CLI を一括更新 |
| パッケージ管理 | nvm、pnpm、Rust、Go、kubectl、k9s、tmux、vim などをインストール/更新 |
| MCP 管理 | Claude/Codex/Gemini の MCP サーバーを管理 |
| セキュリティスキャン | gitleaks、trufflehog、git-secrets、trivy、semgrep を実行 |
| プロンプト生成 | 4 ステップ LLM ワークフロー、進捗追跡対応 |
| スキルインストーラー | AI CLI 拡張機能をインストール（Claude/Codex/Gemini） |
| Rust ビルド | クロスプラットフォーム実行ファイル（cargo/cross、glibc/musl 選択可） |
| コンテナビルド | Docker/Buildah マルチアーキビルド（x86、arm64、armv7、Jetson） |
| Kubeconfig 管理 | tmux ウィンドウ分離の kubeconfig |

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

Codex のアップグレードは `bun install -g @openai/codex` を使用します。

### 3. パッケージのインストール/更新（macOS/Linux）
対話式のチェックリストでインストール・削除・更新を行います：
- `nvm`（最新 Node.js をインストール）
- `pnpm`
- `Rust`（rustup 経由）
- `Go`（最新公式アーカイブ/pkg を取得）
- `Terraform`
- `kubectl`
- `kubectx`
- `k9s`
- `git`
- `uv`（最新 Python をインストール）
- `tmux`（TPM と tmux.conf をセットアップ）
- `vim`（vim-plug と molokai 設定）
- `ffmpeg`（Linux はビルドスクリプト、macOS は Homebrew）

### 4. MCP ツール管理
Claude、Codex、Gemini CLI 用の MCP サーバーを管理します：

| MCP ツール | 説明 |
|------------|------|
| `sequential-thinking` | シーケンシャル・シンキング |
| `context7` | ドキュメント検索 |
| `chrome-devtools` | Chrome 開発者ツール |
| `kubernetes` | K8s 管理 |
| `tailwindcss` | Tailwind CSS（CSS フレームワーク）|
| `arxiv-mcp-server` | arXiv 学術論文検索・ダウンロード |
| `github` | GitHub 統合 |
| `cloudflare-*` | Cloudflare MCP（Docs/Workers/Observability など） |

**環境変数**（オプション - MCP管理機能を使用する場合のみ必要、ビルド時に `.env` で設定）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`：GitHub パーソナルアクセストークン（必須）
- `GITHUB_MCP_MODE`：`docker`（デフォルト、Copilot サブスクリプション不要）または `remote`（GitHub Copilot サブスクリプションが必要）
- `GITHUB_HOST`：GitHub Enterprise ホスト（オプション、デフォルトは `github.com`）
- `GITHUB_TOOLSETS`：ツールセット（オプション、例：`repos,issues,pull_requests,actions`）
- `CONTEXT7_API_KEY`
- `enable_cloudflare_mcp`（`true` で Cloudflare MCP を有効化）
- `ARXIV_STORAGE_PATH`（arXiv 論文の保存パス、デフォルトは `~/.arxiv-papers`）

Codex の MCP インストールでは、`CONTEXT7_API_KEY`、`GITHUB_PERSONAL_ACCESS_TOKEN`、`GITHUB_HOST` のビルド時の値を `~/.codex/config.toml` に書き込むため、実行時の環境変数は不要です。
Cloudflare MCP は OAuth の対話ログインを使用するため、CLI に表示される URL で認可を完了してください。WSL の場合は `wslview <URL>` を使用できます。

### 5. プロジェクトセキュリティスキャナー
現在の Git リポジトリを厳格モードで素早くスキャンします：
- `gitleaks`（Git 履歴 + ワーキングツリー）
- `trufflehog`（Git 履歴 + ワーキングツリー）
- `git-secrets`（Git 履歴 + ワーキングツリー）
- `trivy`（ワーキングツリーの SCA + Misconfig）
- `semgrep`（ワーキングツリーの SAST）

自動インストールは一般的なパッケージマネージャー、Trivy のインストールスクリプト、Semgrep の pipx/venv を優先し、見つからない場合は GitHub Release から取得します（`curl`/`wget` と `tar`/`unzip` が必要です）。
ワーキングツリーのスキャンは Git で追跡済みかつ `.gitignore` で除外されていないファイルのみ対象で、各スキャンの生ログを出力します。

### 6. プロンプトジェネレーター（LLM）
LLM ワークフロー用の 4 ステッププロンプトを生成・実行します：
- **生成**：YAML/JSON 仕様ファイルからプロンプトファイルを作成
- **実行**：Claude/Codex/Gemini CLI で対話式に実行（全件または指定機能を選択）
- **ステータス**：機能の実行進捗とステータスを確認
- **検証**：仕様ファイル形式を検証
- **YAML Prompt**：YAML 仕様 Prompt を生成（内蔵テンプレート）

4 ステップワークフロー：
1. P1：要件、実装、デプロイ
2. P2：検証環境での E2E 検証
3. P3：リファクタリングと最適化
4. P4：検証環境での E2E 回帰テスト

各機能の進捗を追跡し、セッション管理により中断後の再開が可能です。

**Rust Build ヒント**
- `*-unknown-linux-gnu`（glibc）：主流ディストロ向け；動的リンクでバイナリは小さめだがシステム glibc に依存。
- `*-unknown-linux-musl`（musl、多くは静的）：Alpine/最小イメージ向け；単一バイナリ配布に便利。
- `i686-*` はレガシー 32bit x86、`powerpc64le-*` は OpenPOWER/IBM Cloud 向け、`wasm32-unknown-unknown` はブラウザ/wasm ランタイム向け（no std）。

### 7. スキルインストーラー
AI CLI ツールの拡張機能をインストール・管理します：

| CLI | 拡張形式 | インストールパス |
|-----|---------|-----------------|
| Claude Code | Plugins + Skills | `~/.claude/plugins/`、`~/.claude/skills/` |
| OpenAI Codex | Skills (SKILL.md) | `~/.codex/skills/` |
| Google Gemini | Extensions (TOML) | `~/.gemini/extensions/` |

**利用可能な拡張機能：**
- `ralph-wiggum` - AI エージェントループ（Claude/Gemini）
- `security-guidance` - セキュリティベストプラクティス（Claude/Gemini）
- `frontend-design` - フロントエンドインターフェースデザイン（全 CLI）
- `code-review` - コードレビューアシスタント（全 CLI）
- `pr-review-toolkit` - PR レビューツール（全 CLI）
- `commit-commands` - Git Commit ヘルパー（全 CLI）
- `writing-rules` - ライティングスタイルルール（全 CLI）

**注意：** Gemini は異なる拡張形式を使用します。インストーラーは Claude プラグインを Gemini ネイティブの TOML 形式に自動変換し、`extension-enablement.json` に登録します。

詳細は [docs/SKILL_INSTALLER.md](SKILL_INSTALLER.md) 開発ガイドを参照してください。

### 8. コンテナイメージビルダー
Docker または Buildah でマルチアーキテクチャコンテナイメージをビルドします：
- **ビルドエンジン**：Docker (buildx) または Buildah（デーモンレス OCI ビルダー）
- **マルチアーキテクチャサポート**：
  - x86_64 / amd64（Intel/AMD 64 ビット）
  - arm64 / aarch64（Apple Silicon、AWS Graviton）
  - armv7 / arm/v7（Raspberry Pi 2/3）
  - Jetson Nano（NVIDIA Jetson Nano aarch64）
- **Dockerfile スキャナー**：Dockerfile、Containerfile、およびバリアント（Dockerfile.dev など）を自動検出
- **レジストリプッシュ**：コンテナレジストリへのオプションプッシュ
- **クイック選択**：最近使用したイメージ名、タグ、レジストリを記憶して素早く再利用

## インストール

### インストールスクリプト経由 (推奨 - Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### 手動インストール

[Releases](https://github.com/DennySORA/Ops-Tools/releases) ページから、お使いのシステムに対応するバージョンをダウンロードしてください。

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
3. システムパッケージのインストール/更新（macOS/Linux）
4. Rust プロジェクトとツールチェーンのアップグレード
5. 複数プラットフォーム向けに Rust バイナリをビルド（glibc/musl、cargo/cross 選択可）
6. セキュリティスキャン (Gitleaks/TruffleHog/Git-Secrets/Trivy/Semgrep)
7. MCP ツールの管理 (Claude/Codex/Gemini)
8. プロンプトジェネレーター（LLM 4 ステップワークフロー）
9. コンテナイメージビルダー（Docker/Buildah マルチアーキ）
10. Kubeconfig 管理（tmux ウィンドウ分離）
11. 言語設定（英語/繁体字中国語/簡体字中国語/日本語）

起動時に言語選択が表示され、後からメニューで切り替えできます。
言語設定は `~/.config/ops-tools/config.toml`（Linux）、`~/Library/Application Support/ops-tools/config.toml`（macOS）、`%APPDATA%\\ops-tools\\config.toml`（Windows）に保存されます。

## 貢献

Pull Request や Issue の作成は大歓迎です！

## ライセンス

MIT License
