//! 嵌入式設定檔內容
//!
//! 包含 tmux、vim 等工具的預設設定

pub const NVM_INSTALL_SCRIPT: &str =
    "https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh";
pub const PNPM_INSTALL_SCRIPT: &str = "https://get.pnpm.io/install.sh";
pub const RUSTUP_INSTALL_SCRIPT: &str = "https://sh.rustup.rs";
pub const UV_INSTALL_SCRIPT: &str = "https://astral.sh/uv/install.sh";
pub const BUN_INSTALL_SCRIPT: &str = "https://bun.sh/install";

pub const TMUX_CONF_CONTENT: &str = r#"# prefix setting
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

pub const VIMRC_CONTENT: &str = r#"" =========================
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

pub const FFMPEG_BUILD_SCRIPT: &str = r#"#!/usr/bin/env bash
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
make -j"$(nproc)"
make install
cd ..

if [ ! -d x265_git ]; then
  git clone --depth 1 https://bitbucket.org/multicoreware/x265_git.git
else
  git -C x265_git pull --ff-only || true
fi
cd x265_git/build/linux
rm -rf CMakeFiles CMakeCache.txt
cmake -G "Unix Makefiles" -DCMAKE_INSTALL_PREFIX="$PREFIX" -DENABLE_SHARED=OFF ../../source
make -j"$(nproc)"
make install
cd "$PREFIX/src"

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
