# Ops-Tools

DevOps ワークフローの効率化のために設計された、堅牢な Rust 製 CLI ツールセットです。

[English](../README.md) | [繁體中文](README_zh-TW.md) | [简体中文](README_zh-CN.md)

![Screenshot](image/01.png)

## 機能一覧

| カテゴリ | 機能 | 説明 |
|---------|------|------|
| アップグレード | システム更新 | クロスプラットフォーム保守（Linux APT / macOS Homebrew + ツール） |
| アップグレード | AI ツール更新 | Claude Code、Codex、Gemini CLI を一括更新 |
| アップグレード | Rust 更新 | Rust ツールチェーン + Cargo ツールのアップグレード |
| アップグレード | パッケージ管理 | nvm、pnpm、Rust、Go、kubectl、k9s、tmux などをインストール/更新 |
| ビルド | Rust ビルド | クロスプラットフォーム Rust バイナリ（cargo/cross、30+ ターゲット） |
| ビルド | コンテナビルド | Docker/Buildah マルチアーキビルド（x86、arm64、armv7、Jetson） |
| ビルド | CUDA ML ビルド | ML パッケージをソースからビルド（PyTorch、Flash Attention、xFormers） |
| AI | MCP 管理 | Claude/Codex/Gemini の MCP サーバーを管理 |
| AI | スキルインストーラー | AI CLI 拡張機能をインストール（Claude/Codex/Gemini） |
| AI | プロンプト生成 | 4 ステップ LLM ワークフロー、進捗追跡対応 |
| インフラ | Terraform クリーンアップ | `.terraform`、`.terragrunt-cache` およびロックファイルを削除 |
| インフラ | Kubeconfig 管理 | tmux ウィンドウ分離の kubeconfig |
| セキュリティ | セキュリティスキャン | gitleaks、trufflehog、git-secrets、trivy、semgrep を実行 |

## メニュー構造

インタラクティブメニューは 5 つのカテゴリに分類され、使用頻度に基づいてスマートソートされます：

```
よく使う（使用頻度順）
  システム更新、AI ツール更新、...

カテゴリ
  ビルド          — Rust ビルド、コンテナビルド、CUDA ML ビルド
  AI              — MCP 管理、スキルインストーラー、プロンプト生成
  アップグレード  — システム更新、AI ツール更新、Rust 更新、パッケージ管理
  インフラ        — Terraform クリーンアップ、Kubeconfig 管理
  セキュリティ    — セキュリティスキャン

設定          — 言語、よく使うアイテム数、ピン管理
```

ピン留めしたアイテムは最上部に表示されます。よく使うアイテムは使用頻度で自動ソートされます。

## 機能

### システム更新
プラットフォームを考慮したクロスプラットフォーム保守:
- **モード**：フル更新、スキャンのみ、クリーンアップ、検証、バックアップ
- **プロファイル**：デフォルト（フル）、セーフ（再起動なし、控えめ）、アグレッシブ（徹底的）
- **Linux ワークフロー**：APT アップグレード、NVIDIA/WSL ホストでの CUDA Toolkit runfile 更新、DGX カーネル/ドライバー、Snap/Flatpak/Docker、ツール更新（nvm、bun、deno、pipx、conda、pnpm、Rust、uv）、キャッシュクリーンアップ、検証、再起動判断
- **macOS ワークフロー**：Homebrew の更新/アップグレード、保守的な `softwareupdate`、ツール更新、キャッシュクリーンアップ、検証、バックアップスナップショット
- **CUDA 自動検出**：NVIDIA runfile インデックス、`nvidia-smi`、`nvcc`、`dpkg` から最新版 runfile、GPU アーキテクチャ、WSL CUDA シグナル、ドライバー/カーネルパッケージを自動検出
- **プラットフォーム検出**：実行時に Linux と macOS を自動判別し、未対応ステップは安全にスキップ
- **設定**：`update.toml` または `~/.config/update/config.toml`（`update.example.toml` を参照）
- ドライランモードで変更をプレビュー

### AI ツール更新
AI コードアシスタントの一括アップグレード：
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex) — ローカルリポジトリからのソースビルド対応
- `Google Gemini CLI` (@google/gemini-cli)

### パッケージ管理（macOS / Linux）
対話式チェックリストでインストール・削除・更新：
- `nvm`（最新 Node.js）、`pnpm`、`Rust`（rustup 経由）、`Go`（最新公式アーカイブ）
- `Terraform`、`kubectl`、`kubectx`、`k9s`、`git`、`uv`（最新 Python）
- `tmux`（TPM + tmux.conf）、`vim`（vim-plug + molokai）
- `ffmpeg`（Linux はビルドスクリプト、macOS は Homebrew）

### Rust 更新
Rust ツールチェーンと Cargo ツールのアップグレード：
- rustc、cargo、rustup のバージョン確認
- 不足している Cargo ツールをインストール（cargo-edit、cargo-update、cargo-outdated、cargo-audit）
- 6 ステップアップグレード：rustup self-update、rustup update、cargo install-update、cargo upgrade、cargo outdated、cargo audit

### CUDA ML ビルド
GPU に合わせて CUDA 対応 ML パッケージをソースからビルド：
- **パッケージ**：PyTorch、TorchVision、TorchAudio、Flash Attention、xFormers、FlashInfer、BitsAndBytes、ExLlamaV2、AutoGPTQ、AutoAWQ、llama-cpp-python、CTranslate2、TensorRT、Transformer Engine、DeepSpeed、vLLM、CuPy、Unsloth
- **モード**：ソースビルド、キャッシュからインストール、ステータス、クリーン
- CUDA バージョン、GPU アーキテクチャ、ビルド最適化（ccache、Ninja、clang、mold）を自動検出
- 隔離ビルド環境：`~/.ml-packages/`

### MCP 管理
Claude、Codex、Gemini CLI 用の MCP サーバー管理：

| MCP ツール | 説明 |
|------------|------|
| `sequential-thinking` | シーケンシャル・シンキング |
| `context7` | ドキュメント検索 |
| `chrome-devtools` | Chrome 開発者ツール |
| `kubernetes` | K8s 管理 |
| `tailwindcss` | Tailwind CSS |
| `arxiv-mcp-server` | arXiv 論文検索・ダウンロード |
| `github` | GitHub 統合 |
| `cloudflare-*` | Cloudflare MCP サーバー |

**オプション MCP 認証情報**（ビルド時に `.env` で設定）：
1. `cp .env.example .env`
2. 必要な値を入力
3. `cargo build --release` でビルド

利用可能なオプション：
- **Context7**：`CONTEXT7_API_KEY` を設定
- **GitHub**：`GITHUB_PERSONAL_ACCESS_TOKEN`（必須）、オプションで `GITHUB_MCP_MODE`、`GITHUB_HOST`、`GITHUB_TOOLSETS`
- **Cloudflare**：`enable_cloudflare_mcp=true` を設定（インストール時 OAuth）
- **arXiv**：`ARXIV_STORAGE_PATH` を設定（デフォルト `~/.arxiv-papers`）

### スキルインストーラー
AI CLI ツールの拡張機能をインストール：

| CLI | 拡張形式 | インストールパス |
|-----|---------|-----------------|
| Claude Code | Plugins + Skills | `~/.claude/plugins/`、`~/.claude/skills/` |
| OpenAI Codex | Skills (SKILL.md) | `~/.codex/skills/` |
| Google Gemini | Extensions (TOML) | `~/.gemini/extensions/` |

利用可能な拡張機能：ralph-wiggum、security-guidance、frontend-design、code-review、pr-review-toolkit、commit-commands、writing-rules、claude-mem、loop-runner など。

詳細は [docs/SKILL_INSTALLER.md](SKILL_INSTALLER.md) 開発ガイドを参照。

### LLM プロンプトジェネレーター
LLM ワークフロー用の 4 ステッププロンプトを生成・実行：
- **コマンド**：生成、実行、ステータス、検証、YAML Prompt
- **4 ステップ**：P1（実装・デプロイ）、P2（E2E 検証）、P3（リファクタリング・最適化）、P4（回帰テスト）
- 進捗追跡、中断後の再開に対応

### Rust ビルダー
クロスプラットフォーム Rust バイナリのビルド：
- **エンジン**：cargo（ネイティブ）または cross（コンテナ化クロスコンパイル）
- **30+ ターゲット**：x86_64-gnu、x86_64-musl、aarch64、i686、powerpc64le、wasm32 など
- 不足している rustup ターゲットを自動インストール

### コンテナビルダー
マルチアーキテクチャコンテナイメージのビルド：
- **エンジン**：Docker (buildx) または Buildah（デーモンレス）
- **アーキテクチャ**：x86_64、arm64、armv7、Jetson Nano
- Dockerfile/Containerfile バリアントを自動検出
- レジストリプッシュ、よく使う設定を記憶

### Terraform クリーナー
Terraform/Terragrunt キャッシュのスマートクリーンアップ：
- `.terragrunt-cache`、`.terraform`、`.terraform.lock.hcl`
- 重複パスを自動除外

### Kubeconfig マネージャー
tmux ウィンドウ分離の kubeconfig で安全なマルチクラスター操作：
- セットアップ、クリーンアップ、リスト、全クリーンアップ
- 誤って別のクラスターに切り替えることを防止

### セキュリティスキャナー
Git リポジトリを厳格モードでスキャン：
- `gitleaks`、`trufflehog`、`git-secrets`（履歴 + ワーキングツリー）
- `trivy`（SCA + misconfig）、`semgrep`（SAST）
- 組み込みサプライチェーンヒューリスティックで、サブフォルダー内の npm、Python、Rust パッケージファイルを再帰的に検出
- npm install scripts、リモート/ローカル依存関係、lockfile 不足、Python lockfile の URL/index ソース、Rust 代替 registry、git/path 依存関係、integrity/checksum 不足を検出
- 自動インストール、Git 追跡ファイルと ignore されていない未追跡ファイルをスキャンし、`.gitignore` を尊重

## インストール

### インストールスクリプト（Linux / macOS）

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### 手動ダウンロード

[Releases](https://github.com/DennySORA/Ops-Tools/releases) ページからプリビルドバイナリをダウンロード：
- Linux x86_64
- macOS x86_64 / arm64 (Apple Silicon)
- Windows x86_64

### ソースからビルド

```bash
cargo build --release
./target/release/tools

# オプション：MCP 認証情報を設定
cp .env.example .env
# .env を編集し、再ビルド
```

## 多言語対応

4 言語対応 — 初回起動時に選択、設定から変更可能：

- English
- 繁體中文
- 简体中文
- 日本語

言語設定の保存先：
- Linux：`~/.config/ops-tools/config.toml`
- macOS：`~/Library/Application Support/ops-tools/config.toml`
- Windows：`%APPDATA%\ops-tools\config.toml`

## 貢献

Pull Request や Issue の作成は大歓迎です！

詳細は [CONTRIBUTING.md](../CONTRIBUTING.md) 開発ガイドを参照。

## ライセンス

MIT License
