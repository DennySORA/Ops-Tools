use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const NVM_INSTALL_SCRIPT: &str = "https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh";
const PNPM_INSTALL_SCRIPT: &str = "https://get.pnpm.io/install.sh";
const RUSTUP_INSTALL_SCRIPT: &str = "https://sh.rustup.rs";
const UV_INSTALL_SCRIPT: &str = "https://astral.sh/uv/install.sh";

const TMUX_CONF_CONTENT: &str = r#"# prefix setting
set -g prefix C-a
unbind C-b
bind C-a send-prefix

set-option -g default-shell /bin/zsh

set -g status-right '#{prefix_highlight} | %a %Y-%m-%d %H:%M'

set-option -g allow-rename off

bind-key s setw synchronize-panes \; display-message "Synchronize Panes is now #{?pane_synchronized,on,off}"

bind-key v select-layout even-vertical

# display things in 256 colors
set -g default-terminal "screen-256color"

# mouse on
set -g mouse on

# save context
set -g @resurrect-capture-pane-contents 'on'
set -g @resurrect-processes ':all:'

### tmux plugin manager

set -g @plugin 'tmux-plugins/tpm'
set -g @plugin 'tmux-plugins/tmux-sensible'
set -g @plugin 'tmux-plugins/tmux-resurrect'
set -g @plugin 'tmux-plugins/tmux-prefix-highlight'
set -g @plugin 'tmux-plugins/tmux-yank'

# Initialize TMUX plugin manager (keep this line at the very bottom of tmux.conf)
run -b '~/.tmux/plugins/tpm/tpm'
"#;

const VIMRC_CONTENT: &str = r#"" =========================
"  插件管理（vim-plug）
" =========================
" 安裝 vim-plug 後，:PlugInstall 以安裝下列外掛
call plug#begin('~/.vim/plugged')

" 介面與狀態列
Plug 'vim-airline/vim-airline'          " 輕量狀態列/標籤列
" colorscheme molokai 需要主題外掛（若你的 Vim 無內建）
" 如需主題外掛請取消下一行註解
" Plug 'tomasr/molokai'

" 編輯輔助
Plug 'tpope/vim-surround'               " surrond 快速包覆/替換括號、引號等
Plug 'tpope/vim-commentary'             " gcc 進行行註解，gc{motion} 區塊註解
Plug 'tpope/vim-repeat'                 " 讓 . 可重複更多外掛動作
Plug 'michaeljsmith/vim-indent-object'  " 以縮排層級作為 text object

" 檔案/樹狀瀏覽
Plug 'preservim/nerdtree'               " 檔案樹

" 模糊搜尋與全域搜尋
Plug 'junegunn/fzf'                     " fzf 核心（需系統安裝 fzf）
Plug 'junegunn/fzf.vim'                 " fzf 與 Vim 的整合
" 注意：:Rg 需系統安裝 ripgrep

call plug#end()

" =========================
"  外觀與語法
" =========================
syntax on                               " 啟用語法高亮
colorscheme molokai                     " 主題（若無外掛，將回退或報錯）
set number                              " 顯示行號
set cursorline                          " 突顯游標所在行
set showmatch                           " 顯示成對括號
set laststatus=2                        " 永遠顯示狀態列（Vim 8 預設 2）
" 若你的終端支援 24-bit 色彩，可開啟下行以更佳色彩（可選）
" set termguicolors

" =========================
"  編碼與剪貼簿
" =========================
set encoding=utf-8                      " 內部編碼使用 UTF-8
set fileencodings=utf-8,latin1          " 嘗試偵測檔案編碼的順序
set clipboard=unnamedplus               " 使用系統剪貼簿

" =========================
"  編輯體驗與縮排
" =========================
set autoindent                          " 依前一行自動縮排
set tabstop=4                           " <Tab> 寬度 4
set shiftwidth=4                        " 自動縮排寬度 4
set expandtab                           " 輸入 <Tab> 以空格取代
set showcmd                             " 命令列即時顯示未完成按鍵序列
set wildmenu                            " 命令列補全選單
set pastetoggle=<F2>                    " 插入模式按 F2 進入/離開 paste 模式

" =========================
"  搜尋
" =========================
set ignorecase                          " 預設大小寫不敏感
set smartcase                           " 搜尋字串含大寫時，自動轉為大小寫敏感
set incsearch                           " 即時增量顯示搜尋結果
set hlsearch                            " 高亮搜尋結果

" =========================
"  Airline 設定
" =========================
let g:airline#extensions#tabline#enabled = 1   " 顯示 buffer/分頁列

" =========================
"  NERDTree 與 fzf 快捷鍵
" =========================
" 注意：未自訂 <Leader> 時，預設是反斜線 \ ，可自行改：let mapleader = ","
nnoremap <leader>s :NERDTreeToggle<CR>  " 切換檔案樹
nnoremap <leader>l :Lines<CR>           " 以 fzf 搜索「目前檔案」行
nnoremap <leader>g :Rg<CR>              " 以 ripgrep 全域搜尋（需 ripgrep）

" =========================
"  一般快捷鍵與游標移動
" =========================
nnoremap ll $                           " ll 跳到行尾
nnoremap jj ^                           " jj 跳到行首（第一個非空白字元）
nnoremap <leader>r :set relativenumber!<CR> " 切換相對行號，利於跳轉

" =========================
"  行複製並清理縮排空白（進階巨集）
" =========================
" <Leader><CR>：複製當前行到下一行，並清理行首多餘空白，停在行首
" 動作分解：
" 1) mz          記錄當前位置標記 z
" 2) Do<Esc>     複製本行到下一行（D 為刪到行尾，o 新增一行；你的版本用 Do 代表 copy 行到下一行）
" 3) p           再貼上，確保有一份副本
" 4) :silent s/^\(\s*\)\s\+/\1/e   清理多餘縮排空白（保留最左側縮排）
" 5) ^           移到行首第一個非空白
nnoremap <leader><CR> mzDo<Esc>p:silent s/^\(\s*\)\s\+/\1/e<CR>^

" =========================
"  視覺模式配色
" =========================
" 終端模式下高亮選取區背景為黃，前景保持預設
highlight Visual ctermfg=NONE ctermbg=yellow guibg=yellow
"#;

const FFMPEG_BUILD_SCRIPT: &str = r#"#!/usr/bin/env bash
set -euxo pipefail

PREFIX="${HOME}/.ffbuild"
CUDA_PATH="/usr/local/cuda"
ENABLE_NVENC=1

missing_tools=()
for tool in gcc make cmake git nasm yasm pkg-config; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    missing_tools+=("$tool")
  fi
done

if [ ${#missing_tools[@]} -gt 0 ]; then
  echo "Missing required tools: ${missing_tools[*]}"
  if command -v apt >/dev/null 2>&1 && command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    sudo apt update
    sudo apt install -y build-essential pkg-config git yasm nasm cmake libnuma-dev libmp3lame-dev
  else
    echo "[error] Please install missing tools: ${missing_tools[*]}"
    echo "On Ubuntu/Debian: sudo apt install build-essential pkg-config git yasm nasm cmake libnuma-dev libmp3lame-dev"
    exit 1
  fi
fi

mkdir -p "$PREFIX/src" && cd "$PREFIX/src"

if [ "${ENABLE_NVENC}" -eq 1 ]; then
  if [ ! -d nv-codec-headers ]; then
    git clone https://github.com/FFmpeg/nv-codec-headers.git
  else
    git -C nv-codec-headers pull --ff-only || true
  fi
  cd nv-codec-headers
  make distclean || true
  make -j"$(nproc)"
  make install PREFIX="$PREFIX"
  cd ..
fi

if [ ! -d x264 ]; then
  git clone --depth 1 https://code.videolan.org/videolan/x264.git
else
  git -C x264 pull --ff-only || true
fi
cd x264
make distclean || true
./configure --prefix="$PREFIX" --enable-static --enable-pic
make -j"$(nproc)" && make install
cd ..

if [ ! -d x265 ]; then
  git clone --depth 1 https://github.com/videolan/x265.git x265
else
  git -C x265 pull --ff-only || true
fi
cd x265/build/linux

rm -rf CMakeCache.txt CMakeFiles/ || true
cmake -G "Unix Makefiles" \
  -DENABLE_SHARED=OFF \
  -DENABLE_PIC=ON \
  -DHIGH_BIT_DEPTH=ON \
  -DMAIN10=ON \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DENABLE_CLI=OFF \
  ../../source

make -j"$(nproc)" && make install
cd ../../..

export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export LD_LIBRARY_PATH="$PREFIX/lib:${LD_LIBRARY_PATH:-}"

echo "Creating x265.pc file..."
X265_VERSION=$(strings "$PREFIX/lib/libx265.a" 2>/dev/null | grep "x265.*[0-9]\+\.[0-9]" | head -1 | grep -o '[0-9]\+\.[0-9]\+' || echo "3.5")

cat > "$PREFIX/lib/pkgconfig/x265.pc" <<EOF
prefix=$PREFIX
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: x265
Description: H.265/HEVC video encoder library
Version: $X265_VERSION
Requires:
Conflicts:
Libs: -L\${libdir} -lx265 -lstdc++ -lm -lpthread -ldl -lnuma
Cflags: -I\${includedir}
EOF

pkg-config --modversion x264
pkg-config --modversion x265
echo "x265 libs: $(pkg-config --libs x265)"

if [ ! -d ffmpeg ]; then
  git clone --depth 1 https://github.com/FFmpeg/FFmpeg.git ffmpeg
else
  git -C ffmpeg pull --ff-only || true
fi

cd ffmpeg
make distclean || true

export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
echo "PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
pkg-config --exists x264 && echo "x264 found via pkg-config" || echo "x264 NOT found via pkg-config"
pkg-config --exists x265 && echo "x265 found via pkg-config" || echo "x265 NOT found via pkg-config"

COMMON_CFG=(
  --prefix="$PREFIX"
  --pkg-config-flags="--static"
  --extra-cflags="-I$PREFIX/include -I$CUDA_PATH/include"
  --extra-ldflags="-L$PREFIX/lib -L$CUDA_PATH/lib64 -Wl,-rpath,$PREFIX/lib:$CUDA_PATH/lib64"
  --extra-libs="-lpthread -lm -ldl"
  --enable-gpl
  --enable-libx264 --enable-libx265
  --enable-libmp3lame
  --enable-nonfree
  --enable-static
  --disable-shared
  --pkg-config="pkg-config --static"
)

CFG=("${COMMON_CFG[@]}")
if [ "${ENABLE_NVENC}" -eq 1 ]; then
  if [ -d "$CUDA_PATH" ] && [ -f "$CUDA_PATH/include/cuda.h" ] && command -v nvcc >/dev/null 2>&1 && command -v clang >/dev/null 2>&1; then
    CFG+=(--enable-nvenc --enable-cuda-llvm)
    echo "CUDA and Clang found, enabling NVENC and CUDA-LLVM"
  elif [ -d "$CUDA_PATH" ] && [ -f "$CUDA_PATH/include/cuda.h" ] && command -v nvcc >/dev/null 2>&1; then
    CFG+=(--enable-nvenc)
    echo "CUDA found (no Clang), enabling NVENC only"
  else
    echo "CUDA not found at $CUDA_PATH (or nvcc missing), disabling NVENC features"
    ENABLE_NVENC=0
  fi
fi

./configure "${CFG[@]}"
make -j"$(nproc)"
make install

mkdir -p "$HOME/.local/bin"
ln -sf "$PREFIX/bin/ffmpeg" "$HOME/.local/bin/ffmpeg"
ln -sf "$PREFIX/bin/ffprobe" "$HOME/.local/bin/ffprobe"

"$PREFIX/bin/ffmpeg" -hide_banner -encoders | egrep 'libx265|flac' || true
"$PREFIX/bin/ffmpeg" -hide_banner -pix_fmts | grep -E '^.*yuv420p10le' || true

echo "Build finished. ffmpeg at: $PREFIX/bin/ffmpeg"
echo "Build finished. ffprobe at: $PREFIX/bin/ffprobe"
echo "If 'ffmpeg' or 'ffprobe' not found, add to PATH: export PATH=\"$HOME/.local/bin:\$PATH\""
"#;

#[derive(Clone, Copy, Debug)]
pub enum SupportedOs {
    Linux,
    Macos,
}

impl SupportedOs {
    pub fn detect() -> Option<Self> {
        match env::consts::OS {
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::Macos),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Macos => "macOS",
        }
    }

    fn go_os(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "darwin",
        }
    }

    fn kubectl_os(self) -> &'static str {
        self.go_os()
    }
}

#[derive(Clone, Copy, Debug)]
enum PackageManager {
    Brew,
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
}

impl PackageManager {
    fn detect(os: SupportedOs) -> Option<Self> {
        match os {
            SupportedOs::Macos => {
                if is_command_available("brew").is_some() {
                    Some(Self::Brew)
                } else {
                    None
                }
            }
            SupportedOs::Linux => {
                if is_command_available("apt-get").is_some() {
                    Some(Self::Apt)
                } else if is_command_available("dnf").is_some() {
                    Some(Self::Dnf)
                } else if is_command_available("yum").is_some() {
                    Some(Self::Yum)
                } else if is_command_available("pacman").is_some() {
                    Some(Self::Pacman)
                } else if is_command_available("zypper").is_some() {
                    Some(Self::Zypper)
                } else if is_command_available("apk").is_some() {
                    Some(Self::Apk)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageAction {
    Install,
    Update,
    Remove,
}

impl PackageAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Install => i18n::t(keys::PACKAGE_MANAGER_ACTION_INSTALL),
            Self::Update => i18n::t(keys::PACKAGE_MANAGER_ACTION_UPDATE),
            Self::Remove => i18n::t(keys::PACKAGE_MANAGER_ACTION_REMOVE),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageId {
    Nvm,
    Pnpm,
    Rust,
    Go,
    Terraform,
    Kubectl,
    Kubectx,
    K9s,
    Git,
    Uv,
    Tmux,
    Vim,
    Ffmpeg,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageDefinition {
    pub id: PackageId,
    pub name: &'static str,
}

pub struct ActionContext {
    os: SupportedOs,
    package_manager: Option<PackageManager>,
    sudo_available: bool,
    home_dir: PathBuf,
    temp_dir: PathBuf,
    apt_updated: bool,
    pacman_synced: bool,
    hashicorp_repo_ready: bool,
}

impl ActionContext {
    pub fn new(os: SupportedOs) -> Self {
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let temp_dir = env::temp_dir();
        let package_manager = PackageManager::detect(os);
        let sudo_available = is_command_available("sudo").is_some();

        Self {
            os,
            package_manager,
            sudo_available,
            home_dir,
            temp_dir,
            apt_updated: false,
            pacman_synced: false,
            hashicorp_repo_ready: false,
        }
    }

    fn require_package_manager(&self) -> Result<PackageManager> {
        self.package_manager.ok_or_else(|| OperationError::Command {
            command: "package-manager".to_string(),
            message: crate::tr!(keys::PACKAGE_MANAGER_MISSING_PM, os = self.os.label()),
        })
    }
}

pub fn package_definitions() -> Vec<PackageDefinition> {
    vec![
        PackageDefinition {
            id: PackageId::Nvm,
            name: "nvm",
        },
        PackageDefinition {
            id: PackageId::Pnpm,
            name: "pnpm",
        },
        PackageDefinition {
            id: PackageId::Rust,
            name: "Rust",
        },
        PackageDefinition {
            id: PackageId::Go,
            name: "Go",
        },
        PackageDefinition {
            id: PackageId::Terraform,
            name: "Terraform",
        },
        PackageDefinition {
            id: PackageId::Kubectl,
            name: "kubectl",
        },
        PackageDefinition {
            id: PackageId::Kubectx,
            name: "kubectx",
        },
        PackageDefinition {
            id: PackageId::K9s,
            name: "k9s",
        },
        PackageDefinition {
            id: PackageId::Git,
            name: "git",
        },
        PackageDefinition {
            id: PackageId::Uv,
            name: "uv",
        },
        PackageDefinition {
            id: PackageId::Tmux,
            name: "tmux",
        },
        PackageDefinition {
            id: PackageId::Vim,
            name: "vim",
        },
        PackageDefinition {
            id: PackageId::Ffmpeg,
            name: "ffmpeg",
        },
    ]
}

pub fn ensure_curl(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("curl").is_some() {
        return Ok(());
    }
    install_with_manager(ctx, "curl")
}

pub fn update_curl(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("curl").is_none() {
        return ensure_curl(ctx);
    }
    update_with_manager(ctx, "curl")
}

pub fn is_installed(package: PackageId, ctx: &ActionContext) -> bool {
    match package {
        PackageId::Nvm => nvm_dir(ctx).join("nvm.sh").is_file(),
        PackageId::Pnpm => is_command_available("pnpm").is_some(),
        PackageId::Rust => is_command_available("rustup").is_some(),
        PackageId::Go => is_command_available("go").is_some(),
        PackageId::Terraform => is_command_available("terraform").is_some(),
        PackageId::Kubectl => is_command_available("kubectl").is_some(),
        PackageId::Kubectx => is_command_available("kubectx").is_some(),
        PackageId::K9s => is_command_available("k9s").is_some(),
        PackageId::Git => is_command_available("git").is_some(),
        PackageId::Uv => is_command_available("uv").is_some(),
        PackageId::Tmux => is_command_available("tmux").is_some(),
        PackageId::Vim => is_command_available("vim").is_some(),
        PackageId::Ffmpeg => is_command_available("ffmpeg").is_some(),
    }
}

pub fn apply_action(
    action: PackageAction,
    package: PackageId,
    ctx: &mut ActionContext,
) -> Result<()> {
    match action {
        PackageAction::Install => install_package(package, ctx),
        PackageAction::Update => update_package(package, ctx),
        PackageAction::Remove => remove_package(package, ctx),
    }
}

fn install_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => install_nvm(ctx),
        PackageId::Pnpm => install_pnpm(ctx),
        PackageId::Rust => install_rust(ctx),
        PackageId::Go => install_go(ctx),
        PackageId::Terraform => install_terraform(ctx),
        PackageId::Kubectl => install_kubectl(ctx),
        PackageId::Kubectx => install_kubectx(ctx),
        PackageId::K9s => install_k9s(ctx),
        PackageId::Git => install_git(ctx),
        PackageId::Uv => install_uv(ctx),
        PackageId::Tmux => install_tmux(ctx),
        PackageId::Vim => install_vim(ctx),
        PackageId::Ffmpeg => install_ffmpeg(ctx),
    }
}

fn update_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => update_nvm(ctx),
        PackageId::Pnpm => update_pnpm(ctx),
        PackageId::Rust => update_rust(ctx),
        PackageId::Go => install_go(ctx),
        PackageId::Terraform => update_terraform(ctx),
        PackageId::Kubectl => install_kubectl(ctx),
        PackageId::Kubectx => update_kubectx(ctx),
        PackageId::K9s => update_k9s(ctx),
        PackageId::Git => update_git(ctx),
        PackageId::Uv => update_uv(ctx),
        PackageId::Tmux => update_tmux(ctx),
        PackageId::Vim => update_vim(ctx),
        PackageId::Ffmpeg => update_ffmpeg(ctx),
    }
}

fn remove_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => remove_nvm(ctx),
        PackageId::Pnpm => remove_pnpm(ctx),
        PackageId::Rust => remove_rust(ctx),
        PackageId::Go => remove_go(ctx),
        PackageId::Terraform => remove_terraform(ctx),
        PackageId::Kubectl => remove_binary(ctx, "kubectl"),
        PackageId::Kubectx => remove_kubectx(ctx),
        PackageId::K9s => remove_k9s(ctx),
        PackageId::Git => remove_git(ctx),
        PackageId::Uv => remove_uv(ctx),
        PackageId::Tmux => remove_tmux(ctx),
        PackageId::Vim => remove_vim(ctx),
        PackageId::Ffmpeg => remove_ffmpeg(ctx),
    }
}

fn install_nvm(ctx: &mut ActionContext) -> Result<()> {
    run_shell(ctx, &format!("curl -o- {NVM_INSTALL_SCRIPT} | bash"), false)?;
    let nvm_dir = nvm_dir(ctx);
    let command = format!(
        "export NVM_DIR=\"{dir}\"; [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"; nvm install node; nvm alias default node",
        dir = nvm_dir.display()
    );
    run_shell(ctx, &command, false)?;
    Ok(())
}

fn update_nvm(ctx: &mut ActionContext) -> Result<()> {
    install_nvm(ctx)
}

fn remove_nvm(ctx: &mut ActionContext) -> Result<()> {
    let dir = nvm_dir(ctx);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|err| OperationError::Io {
            path: dir.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

fn install_pnpm(ctx: &mut ActionContext) -> Result<()> {
    run_shell(
        ctx,
        &format!("curl -fsSL {PNPM_INSTALL_SCRIPT} | sh -"),
        false,
    )?;
    Ok(())
}

fn update_pnpm(ctx: &mut ActionContext) -> Result<()> {
    install_pnpm(ctx)
}

fn remove_pnpm(ctx: &mut ActionContext) -> Result<()> {
    let pnpm_home = ctx.home_dir.join(".local/share/pnpm");
    let pnpm_global = ctx.home_dir.join(".local/share/pnpm-global");
    if pnpm_home.exists() {
        fs::remove_dir_all(&pnpm_home).map_err(|err| OperationError::Io {
            path: pnpm_home.display().to_string(),
            source: err,
        })?;
    }
    if pnpm_global.exists() {
        fs::remove_dir_all(&pnpm_global).map_err(|err| OperationError::Io {
            path: pnpm_global.display().to_string(),
            source: err,
        })?;
    }
    remove_home_binary(ctx, "pnpm")?;
    remove_home_binary(ctx, "pnpx")?;
    Ok(())
}

fn install_rust(ctx: &mut ActionContext) -> Result<()> {
    run_shell(
        ctx,
        &format!("curl --proto '=https' --tlsv1.2 -sSf {RUSTUP_INSTALL_SCRIPT} | sh -s -- -y"),
        false,
    )?;
    Ok(())
}

fn update_rust(ctx: &mut ActionContext) -> Result<()> {
    let rustup = rustup_path(ctx).ok_or_else(|| OperationError::Command {
        command: "rustup".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_RUSTUP_MISSING).to_string(),
    })?;
    run_command_path(ctx, &rustup, &["self", "update"], false)?;
    run_command_path(ctx, &rustup, &["update"], false)?;
    Ok(())
}

fn remove_rust(ctx: &mut ActionContext) -> Result<()> {
    if let Some(rustup) = rustup_path(ctx) {
        run_command_path(ctx, &rustup, &["self", "uninstall", "-y"], false)?;
    }
    let rustup_dir = ctx.home_dir.join(".rustup");
    let cargo_dir = ctx.home_dir.join(".cargo");
    if rustup_dir.exists() {
        let _ = fs::remove_dir_all(&rustup_dir);
    }
    if cargo_dir.exists() {
        let _ = fs::remove_dir_all(&cargo_dir);
    }
    Ok(())
}

fn install_go(ctx: &mut ActionContext) -> Result<()> {
    let download = latest_go_download(ctx)?;
    let temp_dir = create_temp_dir(ctx, "go-download")?;
    let archive_path = temp_dir.join(&download.filename);
    download_file(ctx, &download.url, &archive_path)?;

    match ctx.os {
        SupportedOs::Linux => {
            run_command(ctx, "rm", &["-rf", "/usr/local/go"], ctx.sudo_available)?;
            run_command(
                ctx,
                "tar",
                &[
                    "-C",
                    "/usr/local",
                    "-xzf",
                    archive_path.to_str().unwrap_or_default(),
                ],
                ctx.sudo_available,
            )?;
            ensure_profile_line(ctx, "export PATH=$PATH:/usr/local/go/bin")?;
        }
        SupportedOs::Macos => {
            run_command(
                ctx,
                "installer",
                &[
                    "-pkg",
                    archive_path.to_str().unwrap_or_default(),
                    "-target",
                    "/",
                ],
                ctx.sudo_available,
            )?;
        }
    }
    Ok(())
}

fn remove_go(ctx: &mut ActionContext) -> Result<()> {
    run_command(ctx, "rm", &["-rf", "/usr/local/go"], ctx.sudo_available)?;
    Ok(())
}

fn install_terraform(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "terraform"),
        SupportedOs::Linux => install_terraform_linux(ctx),
    }
}

fn update_terraform(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "terraform"),
        SupportedOs::Linux => update_terraform_linux(ctx),
    }
}

fn remove_terraform(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "terraform")
}

fn install_kubectl(ctx: &mut ActionContext) -> Result<()> {
    let version = fetch_text(
        ctx,
        "https://dl.k8s.io/release/stable.txt",
        &["-H", "User-Agent: ops-tools"],
    )?;
    let version = version.trim();
    let arch = go_arch()?;
    let os = ctx.os.kubectl_os();
    let url = format!(
        "https://dl.k8s.io/release/{}/bin/{}/{}/kubectl",
        version, os, arch
    );
    let checksum_url = format!("{}.sha256", url);

    let temp_dir = create_temp_dir(ctx, "kubectl")?;
    let bin_path = temp_dir.join("kubectl");
    download_file(ctx, &url, &bin_path)?;

    let checksum = fetch_text(ctx, &checksum_url, &["-H", "User-Agent: ops-tools"])?;
    verify_checksum(ctx, &bin_path, checksum.trim())?;

    install_binary(ctx, &bin_path, "kubectl")?;
    Ok(())
}

fn install_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => install_kubectx_linux(ctx),
    }
}

fn update_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => update_kubectx_linux(ctx),
    }
}

fn remove_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => remove_kubectx_linux(ctx),
    }
}

fn install_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "k9s"),
        SupportedOs::Linux => install_k9s_linux(ctx),
    }
}

fn update_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "k9s"),
        SupportedOs::Linux => install_k9s_linux(ctx),
    }
}

fn remove_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "k9s"),
        SupportedOs::Linux => remove_binary(ctx, "k9s"),
    }
}

fn install_git(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "git"),
        SupportedOs::Linux => install_with_manager(ctx, "git"),
    }
}

fn update_git(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "git")
}

fn remove_git(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "git")
}

fn install_uv(ctx: &mut ActionContext) -> Result<()> {
    run_shell(ctx, &format!("curl -LsSf {UV_INSTALL_SCRIPT} | sh"), false)?;
    install_uv_python(ctx)?;
    Ok(())
}

fn update_uv(ctx: &mut ActionContext) -> Result<()> {
    install_uv(ctx)
}

fn remove_uv(ctx: &mut ActionContext) -> Result<()> {
    if let Some(path) = uv_path(ctx) {
        remove_file(ctx, &path)?;
    }
    let uv_dir = ctx.home_dir.join(".local/share/uv");
    if uv_dir.exists() {
        let _ = fs::remove_dir_all(&uv_dir);
    }
    Ok(())
}

fn install_tmux(ctx: &mut ActionContext) -> Result<()> {
    install_with_manager(ctx, "tmux")?;
    setup_tmux_config(ctx)?;
    Ok(())
}

fn update_tmux(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "tmux")?;
    setup_tmux_config(ctx)?;
    Ok(())
}

fn remove_tmux(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "tmux")
}

fn install_vim(ctx: &mut ActionContext) -> Result<()> {
    install_with_manager(ctx, "vim")?;
    setup_vim_config(ctx)?;
    Ok(())
}

fn update_vim(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "vim")?;
    setup_vim_config(ctx)?;
    Ok(())
}

fn remove_vim(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "vim")
}

fn install_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "ffmpeg"),
        SupportedOs::Linux => run_ffmpeg_build(ctx),
    }
}

fn update_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    install_ffmpeg(ctx)
}

fn remove_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "ffmpeg"),
        SupportedOs::Linux => {
            let prefix = ctx.home_dir.join(".ffbuild");
            if prefix.exists() {
                let _ = fs::remove_dir_all(&prefix);
            }
            remove_home_binary(ctx, "ffmpeg")?;
            remove_home_binary(ctx, "ffprobe")?;
            Ok(())
        }
    }
}

fn install_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = ctx.require_package_manager()?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["install", package], false)?;
        }
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(ctx, "apt-get", &["install", "-y", package], true)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["install", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["install", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            ensure_pacman_sync(ctx)?;
            run_command(ctx, "pacman", &["-S", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["install", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["add", package], true)?;
        }
    }
    Ok(())
}

fn update_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = ctx.require_package_manager()?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["upgrade", package], false)?;
        }
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(
                ctx,
                "apt-get",
                &["install", "-y", "--only-upgrade", package],
                true,
            )?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["upgrade", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["update", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            ensure_pacman_sync(ctx)?;
            run_command(ctx, "pacman", &["-S", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["update", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["upgrade", package], true)?;
        }
    }
    Ok(())
}

fn remove_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = ctx.require_package_manager()?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["uninstall", package], false)?;
        }
        PackageManager::Apt => {
            run_command(ctx, "apt-get", &["remove", "-y", package], true)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["remove", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["remove", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            run_command(ctx, "pacman", &["-R", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["remove", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["del", package], true)?;
        }
    }
    Ok(())
}

fn install_terraform_linux(ctx: &mut ActionContext) -> Result<()> {
    ensure_hashicorp_repo(ctx)?;
    install_with_manager(ctx, "terraform")
}

fn update_terraform_linux(ctx: &mut ActionContext) -> Result<()> {
    ensure_hashicorp_repo(ctx)?;
    update_with_manager(ctx, "terraform")
}

fn install_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("git").is_none() {
        return Err(OperationError::Command {
            command: "git".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GIT_REQUIRED).to_string(),
        });
    }

    let repo_dir = ctx.home_dir.join(".kubectx");
    if repo_dir.exists() {
        run_command(
            ctx,
            "git",
            &[
                "-C",
                repo_dir.to_str().unwrap_or_default(),
                "pull",
                "--ff-only",
            ],
            false,
        )?;
    } else {
        run_command(
            ctx,
            "git",
            &[
                "clone",
                "https://github.com/ahmetb/kubectx",
                repo_dir.to_str().unwrap_or_default(),
            ],
            false,
        )?;
    }

    let bin_dir = ctx.home_dir.join(".local/bin");
    fs::create_dir_all(&bin_dir).map_err(|err| OperationError::Io {
        path: bin_dir.display().to_string(),
        source: err,
    })?;
    let link_path = bin_dir.join("kubectx");
    let target = repo_dir.join("kubectx");
    create_symlink(&target, &link_path)?;
    Ok(())
}

fn update_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    install_kubectx_linux(ctx)
}

fn remove_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    let repo_dir = ctx.home_dir.join(".kubectx");
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }
    remove_home_binary(ctx, "kubectx")?;
    Ok(())
}

fn install_k9s_linux(ctx: &mut ActionContext) -> Result<()> {
    let asset = latest_github_asset("derailed/k9s", ctx, "k9s_", ".tar.gz")?;
    let temp_dir = create_temp_dir(ctx, "k9s")?;
    let archive = temp_dir.join(&asset.name);
    download_file(ctx, &asset.url, &archive)?;
    extract_tar(ctx, &archive, &temp_dir)?;
    let binary = find_binary(&temp_dir, "k9s").ok_or_else(|| OperationError::Command {
        command: "k9s".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_BINARY_NOT_FOUND).to_string(),
    })?;
    install_binary(ctx, &binary, "k9s")?;
    Ok(())
}

fn install_uv_python(ctx: &mut ActionContext) -> Result<()> {
    let uv = uv_path(ctx).ok_or_else(|| OperationError::Command {
        command: "uv".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_UV_MISSING).to_string(),
    })?;
    run_command_path(ctx, &uv, &["python", "install"], false)?;
    Ok(())
}

fn setup_tmux_config(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("git").is_none() {
        return Err(OperationError::Command {
            command: "git".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GIT_REQUIRED).to_string(),
        });
    }

    let plugins_dir = ctx.home_dir.join(".tmux/plugins");
    let tpm_dir = plugins_dir.join("tpm");
    fs::create_dir_all(&plugins_dir).map_err(|err| OperationError::Io {
        path: plugins_dir.display().to_string(),
        source: err,
    })?;

    if tpm_dir.exists() {
        run_command(
            ctx,
            "git",
            &[
                "-C",
                tpm_dir.to_str().unwrap_or_default(),
                "pull",
                "--ff-only",
            ],
            false,
        )?;
    } else {
        run_command(
            ctx,
            "git",
            &[
                "clone",
                "https://github.com/tmux-plugins/tpm",
                tpm_dir.to_str().unwrap_or_default(),
            ],
            false,
        )?;
    }

    let vim_plug = ctx.home_dir.join(".vim/autoload/plug.vim");
    download_file(
        ctx,
        "https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim",
        &vim_plug,
    )?;

    write_config_with_backup(&ctx.home_dir.join(".tmux.conf"), TMUX_CONF_CONTENT)?;
    Ok(())
}

fn setup_vim_config(ctx: &mut ActionContext) -> Result<()> {
    let vim_plug = ctx.home_dir.join(".vim/autoload/plug.vim");
    download_file(
        ctx,
        "https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim",
        &vim_plug,
    )?;

    let colors_dir = ctx.home_dir.join(".vim/colors");
    fs::create_dir_all(&colors_dir).map_err(|err| OperationError::Io {
        path: colors_dir.display().to_string(),
        source: err,
    })?;
    download_file(
        ctx,
        "https://raw.githubusercontent.com/tomasr/molokai/master/colors/molokai.vim",
        &colors_dir.join("molokai.vim"),
    )?;

    write_config_with_backup(&ctx.home_dir.join(".vimrc"), VIMRC_CONTENT)?;
    Ok(())
}

fn run_ffmpeg_build(ctx: &mut ActionContext) -> Result<()> {
    let temp_dir = create_temp_dir(ctx, "ffmpeg-build")?;
    let script_path = temp_dir.join("build_ffmpeg.sh");
    fs::write(&script_path, FFMPEG_BUILD_SCRIPT).map_err(|err| OperationError::Io {
        path: script_path.display().to_string(),
        source: err,
    })?;
    run_command(
        ctx,
        "bash",
        &[script_path.to_str().unwrap_or_default()],
        false,
    )?;
    Ok(())
}

fn run_command(
    ctx: &ActionContext,
    program: &str,
    args: &[&str],
    use_sudo: bool,
) -> Result<String> {
    let mut args_vec: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let mut program = program.to_string();

    if use_sudo && ctx.sudo_available {
        args_vec.insert(0, program.clone());
        program = "sudo".to_string();
    }

    let output = Command::new(&program)
        .args(&args_vec)
        .output()
        .map_err(|err| OperationError::Command {
            command: program.clone(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(OperationError::Command {
            command: format!("{} {}", program, args_vec.join(" ")),
            message: stderr
                .lines()
                .next()
                .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                .to_string(),
        })
    }
}

fn run_command_path(
    ctx: &ActionContext,
    program: &Path,
    args: &[&str],
    use_sudo: bool,
) -> Result<String> {
    run_command(ctx, program.to_str().unwrap_or_default(), args, use_sudo)
}

fn run_shell(ctx: &ActionContext, command: &str, use_sudo: bool) -> Result<String> {
    if use_sudo && !ctx.sudo_available {
        return Err(OperationError::Command {
            command: "sudo".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_SUDO_REQUIRED).to_string(),
        });
    }

    if use_sudo {
        run_command(ctx, "sudo", &["bash", "-c", command], false)
    } else {
        run_command(ctx, "bash", &["-c", command], false)
    }
}

fn download_file(ctx: &ActionContext, url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|err| OperationError::Io {
            path: parent.display().to_string(),
            source: err,
        })?;
    }

    run_command(
        ctx,
        "curl",
        &["-fL", "-o", dest.to_str().unwrap_or_default(), url],
        false,
    )?;
    Ok(())
}

fn fetch_text(ctx: &ActionContext, url: &str, extra_args: &[&str]) -> Result<String> {
    let mut args = vec!["-sSfL"];
    args.extend_from_slice(extra_args);
    args.push(url);
    run_command(ctx, "curl", &args, false)
}

fn create_temp_dir(ctx: &ActionContext, prefix: &str) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| OperationError::Command {
            command: "time".to_string(),
            message: err.to_string(),
        })?
        .as_millis();
    let dir = ctx
        .temp_dir
        .join(format!("ops-tools-{}-{}", prefix, timestamp));
    fs::create_dir_all(&dir).map_err(|err| OperationError::Io {
        path: dir.display().to_string(),
        source: err,
    })?;
    Ok(dir)
}

fn ensure_apt_updated(ctx: &mut ActionContext) -> Result<()> {
    if ctx.apt_updated {
        return Ok(());
    }
    run_command(ctx, "apt-get", &["update"], true)?;
    ctx.apt_updated = true;
    Ok(())
}

fn ensure_pacman_sync(ctx: &mut ActionContext) -> Result<()> {
    if ctx.pacman_synced {
        return Ok(());
    }
    run_command(ctx, "pacman", &["-Sy", "--noconfirm"], true)?;
    ctx.pacman_synced = true;
    Ok(())
}

fn ensure_hashicorp_repo(ctx: &mut ActionContext) -> Result<()> {
    if ctx.hashicorp_repo_ready {
        return Ok(());
    }

    let manager = ctx.require_package_manager()?;
    match manager {
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(
                ctx,
                "apt-get",
                &["install", "-y", "gnupg", "software-properties-common"],
                true,
            )?;
            let codename = detect_apt_codename(ctx)?;
            let gpg_cmd = "curl -fsSL https://apt.releases.hashicorp.com/gpg | gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg";
            run_shell(ctx, gpg_cmd, true)?;
            let repo_line = format!(
                "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com {codename} main"
            );
            let repo_cmd =
                format!("echo \"{repo_line}\" | tee /etc/apt/sources.list.d/hashicorp.list");
            run_shell(ctx, &repo_cmd, true)?;
            ensure_apt_updated(ctx)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["install", "-y", "dnf-plugins-core"], true)?;
            run_command(
                ctx,
                "dnf",
                &[
                    "config-manager",
                    "--add-repo",
                    "https://rpm.releases.hashicorp.com/fedora/hashicorp.repo",
                ],
                true,
            )?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["install", "-y", "yum-utils"], true)?;
            run_command(
                ctx,
                "yum-config-manager",
                &[
                    "--add-repo",
                    "https://rpm.releases.hashicorp.com/RHEL/hashicorp.repo",
                ],
                true,
            )?;
        }
        _ => {}
    }

    ctx.hashicorp_repo_ready = true;
    Ok(())
}

fn detect_apt_codename(ctx: &ActionContext) -> Result<String> {
    if let Some(value) = read_os_release_value("VERSION_CODENAME")
        .or_else(|| read_os_release_value("UBUNTU_CODENAME"))
    {
        return Ok(value);
    }

    if is_command_available("lsb_release").is_some() {
        let output = run_command(ctx, "lsb_release", &["-cs"], false)?;
        let code = output.trim();
        if !code.is_empty() {
            return Ok(code.to_string());
        }
    }

    Err(OperationError::Command {
        command: "lsb_release".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_CODENAME_MISSING).to_string(),
    })
}

fn read_os_release_value(key: &str) -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        let mut parts = line.splitn(2, '=');
        let k = parts.next()?.trim();
        let v = parts.next()?.trim().trim_matches('"');
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

fn install_binary(ctx: &ActionContext, source: &Path, name: &str) -> Result<PathBuf> {
    let system_dir = Path::new("/usr/local/bin");
    if ctx.sudo_available {
        run_command(
            ctx,
            "install",
            &[
                "-m",
                "0755",
                source.to_str().unwrap_or_default(),
                system_dir.join(name).to_str().unwrap_or_default(),
            ],
            true,
        )?;
        return Ok(system_dir.join(name));
    }

    let local_dir = ctx.home_dir.join(".local/bin");
    fs::create_dir_all(&local_dir).map_err(|err| OperationError::Io {
        path: local_dir.display().to_string(),
        source: err,
    })?;
    let target = local_dir.join(name);
    fs::copy(source, &target).map_err(|err| OperationError::Io {
        path: target.display().to_string(),
        source: err,
    })?;
    set_executable(&target)?;
    Ok(target)
}

fn remove_binary(ctx: &ActionContext, name: &str) -> Result<()> {
    if let Some(path) = is_command_available(name) {
        remove_file(ctx, &path)?;
    }
    Ok(())
}

fn remove_home_binary(ctx: &ActionContext, name: &str) -> Result<()> {
    let local_bin = ctx.home_dir.join(".local/bin").join(name);
    if local_bin.exists() {
        fs::remove_file(&local_bin).map_err(|err| OperationError::Io {
            path: local_bin.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

fn remove_file(ctx: &ActionContext, path: &Path) -> Result<()> {
    if path.exists() {
        if path.starts_with("/usr/local") && ctx.sudo_available {
            run_command(ctx, "rm", &["-f", path.to_str().unwrap_or_default()], true)?;
        } else {
            fs::remove_file(path).map_err(|err| OperationError::Io {
                path: path.display().to_string(),
                source: err,
            })?;
        }
    }
    Ok(())
}

fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|err| OperationError::Io {
                path: path.display().to_string(),
                source: err,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

fn ensure_profile_line(ctx: &ActionContext, line: &str) -> Result<()> {
    let profile = ctx.home_dir.join(".profile");
    let mut needs_write = true;
    if let Ok(existing) = fs::read_to_string(&profile) {
        if existing.contains(line) {
            needs_write = false;
        }
    }

    if needs_write {
        let mut content = fs::read_to_string(&profile).unwrap_or_default();
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(line);
        content.push('\n');
        fs::write(&profile, content).map_err(|err| OperationError::Io {
            path: profile.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

fn write_config_with_backup(path: &Path, content: &str) -> Result<()> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(());
        }
        let backup = backup_path(path);
        fs::copy(path, &backup).map_err(|err| OperationError::Io {
            path: backup.display().to_string(),
            source: err,
        })?;
    }

    fs::write(path, content).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "config".to_string());
    path.with_file_name(format!("{}.bak", name))
}

fn nvm_dir(ctx: &ActionContext) -> PathBuf {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("nvm")
    } else {
        ctx.home_dir.join(".nvm")
    }
}

fn rustup_path(ctx: &ActionContext) -> Option<PathBuf> {
    if let Some(path) = is_command_available("rustup") {
        return Some(path);
    }
    let fallback = ctx.home_dir.join(".cargo/bin/rustup");
    if fallback.is_file() {
        Some(fallback)
    } else {
        None
    }
}

fn uv_path(ctx: &ActionContext) -> Option<PathBuf> {
    if let Some(path) = is_command_available("uv") {
        return Some(path);
    }
    let candidates = [
        ctx.home_dir.join(".local/bin/uv"),
        ctx.home_dir.join(".cargo/bin/uv"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn go_arch() -> Result<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        _ => Err(OperationError::Command {
            command: "arch".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_ARCH_UNSUPPORTED).to_string(),
        }),
    }
}

#[derive(Deserialize)]
struct GoRelease {
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize)]
struct GoFile {
    filename: String,
    os: String,
    arch: String,
    kind: String,
}

struct GoDownload {
    filename: String,
    url: String,
}

fn latest_go_download(ctx: &ActionContext) -> Result<GoDownload> {
    let json = fetch_text(ctx, "https://go.dev/dl/?mode=json", &[])?;
    let releases: Vec<GoRelease> =
        serde_json::from_str(&json).map_err(|err| OperationError::Command {
            command: "go release".to_string(),
            message: err.to_string(),
        })?;
    let release =
        releases
            .into_iter()
            .find(|rel| rel.stable)
            .ok_or_else(|| OperationError::Command {
                command: "go release".to_string(),
                message: i18n::t(keys::PACKAGE_MANAGER_GO_VERSION_MISSING).to_string(),
            })?;

    let arch = go_arch()?;
    let desired_kind = match ctx.os {
        SupportedOs::Linux => "archive",
        SupportedOs::Macos => "installer",
    };
    let file = release
        .files
        .into_iter()
        .find(|file| file.os == ctx.os.go_os() && file.arch == arch && file.kind == desired_kind)
        .ok_or_else(|| OperationError::Command {
            command: "go download".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GO_FILE_MISSING).to_string(),
        })?;

    Ok(GoDownload {
        filename: file.filename.clone(),
        url: format!("https://go.dev/dl/{}", file.filename),
    })
}

fn verify_checksum(ctx: &ActionContext, path: &Path, checksum: &str) -> Result<()> {
    if is_command_available("sha256sum").is_some() {
        let command = format!(
            "echo \"{}  {}\" | sha256sum --check",
            checksum,
            path.to_str().unwrap_or_default()
        );
        run_shell(ctx, &command, false)?;
        return Ok(());
    }

    if is_command_available("shasum").is_some() {
        let command = format!(
            "echo \"{}  {}\" | shasum -a 256 -c -",
            checksum,
            path.to_str().unwrap_or_default()
        );
        run_shell(ctx, &command, false)?;
        return Ok(());
    }

    Ok(())
}

fn extract_tar(ctx: &ActionContext, archive: &Path, target: &Path) -> Result<()> {
    run_command(
        ctx,
        "tar",
        &[
            "-xzf",
            archive.to_str().unwrap_or_default(),
            "-C",
            target.to_str().unwrap_or_default(),
        ],
        false,
    )?;
    Ok(())
}

fn find_binary(dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_binary(&path, name) {
                return Some(found);
            }
        } else if path
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| f == name)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

struct GithubAsset {
    name: String,
    url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

fn latest_github_asset(
    repo: &str,
    ctx: &ActionContext,
    prefix: &str,
    suffix: &str,
) -> Result<GithubAsset> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let json = fetch_text(ctx, &url, &["-H", "User-Agent: ops-tools"])?;
    let release: GithubRelease =
        serde_json::from_str(&json).map_err(|err| OperationError::Command {
            command: "github release".to_string(),
            message: err.to_string(),
        })?;

    let os_token = match ctx.os {
        SupportedOs::Linux => "Linux",
        SupportedOs::Macos => "Darwin",
    };
    let arch_token = match go_arch()? {
        "amd64" => "amd64",
        "arm64" => "arm64",
        other => other,
    };

    let asset = release
        .assets
        .into_iter()
        .find(|asset| {
            asset.name.contains(prefix)
                && asset.name.contains(os_token)
                && asset.name.contains(arch_token)
                && asset.name.ends_with(suffix)
        })
        .ok_or_else(|| OperationError::Command {
            command: "github release".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_RELEASE_ASSET_MISSING).to_string(),
        })?;

    Ok(GithubAsset {
        name: asset.name,
        url: asset.browser_download_url,
    })
}

fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if link.exists() {
        let _ = fs::remove_file(link);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|err| OperationError::Io {
            path: link.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

fn is_command_available(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.is_absolute() || command.contains(std::path::MAIN_SEPARATOR) {
        if path.is_file() {
            return Some(path.to_path_buf());
        }
        return None;
    }

    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            let extensions = ["exe", "cmd", "bat"];
            for ext in extensions {
                let candidate = dir.join(format!("{}.{}", command, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}
