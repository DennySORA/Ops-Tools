//! 交互式介面模組
//!
//! 提供用戶友好的交互式執行介面
//!
//! 支援快速命令：
//! - `g <name/number>` - 執行指定功能
//! - `s <name/number>` - 查看功能詳情
//! - `r` - 重新載入
//! - `q` - 離開
//! - `a` - 執行所有功能

use anyhow::{bail, Context, Result};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, MultiSelect, Select};
use std::path::{Path, PathBuf};

use super::executor::{Executor, ExecutorConfig};
use super::progress::{read_state_status, FeatureInfo, FeatureStatus, Step};

// ============================================================================
// 執行模式
// ============================================================================

/// 執行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RunMode {
    /// 自動模式：依序執行所有功能
    Auto,
    /// 交互模式：讓用戶選擇
    Interactive,
    /// 單一功能模式
    SingleFeature,
}

// ============================================================================
// 快速命令
// ============================================================================

/// 快速命令
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickCommand {
    /// 執行功能 (g <name/number>)
    Go(FeatureSelector),
    /// 查看功能詳情 (s <name/number>)
    Show(FeatureSelector),
    /// 執行所有功能 (a)
    All,
    /// 重新載入 (r)
    Reload,
    /// 離開 (q)
    Quit,
    /// 顯示幫助 (h 或 ?)
    Help,
    /// 使用選單模式
    Menu,
}

/// 功能選擇器
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureSelector {
    /// 按編號選擇 (1-based)
    Index(usize),
    /// 按名稱選擇（支援部分匹配）
    Name(String),
}

impl QuickCommand {
    /// 解析命令字串
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // 空輸入 -> 使用選單
        if input.is_empty() {
            return Some(QuickCommand::Menu);
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim());

        match cmd.as_str() {
            // 執行功能
            "g" | "go" | "run" | "r" if arg.is_some() => {
                Some(QuickCommand::Go(FeatureSelector::parse(arg.unwrap())))
            }
            // 查看詳情
            "s" | "show" | "v" | "view" => {
                arg.map(|a| QuickCommand::Show(FeatureSelector::parse(a)))
            }
            // 執行所有
            "a" | "all" => Some(QuickCommand::All),
            // 重新載入
            "r" | "reload" if arg.is_none() => Some(QuickCommand::Reload),
            // 離開
            "q" | "quit" | "exit" => Some(QuickCommand::Quit),
            // 幫助
            "h" | "help" | "?" => Some(QuickCommand::Help),
            // 嘗試作為功能名稱解析
            _ => {
                // 如果是數字，當作執行功能
                if let Ok(n) = input.parse::<usize>() {
                    Some(QuickCommand::Go(FeatureSelector::Index(n)))
                } else {
                    // 當作功能名稱
                    Some(QuickCommand::Go(FeatureSelector::Name(input.to_string())))
                }
            }
        }
    }
}

impl FeatureSelector {
    /// 解析選擇器
    pub fn parse(input: &str) -> Self {
        let input = input.trim();
        if let Ok(n) = input.parse::<usize>() {
            FeatureSelector::Index(n)
        } else {
            FeatureSelector::Name(input.to_string())
        }
    }
}

// ============================================================================
// 交互式執行器
// ============================================================================

/// 交互式執行器
pub struct InteractiveRunner {
    /// 功能目錄
    features_dir: PathBuf,
    /// 功能列表
    features: Vec<FeatureInfo>,
    /// Claude 執行器
    executor: Executor,
    /// 終端機
    term: Term,
    /// 是否自動繼續執行下一步
    auto_continue: bool,
}

impl InteractiveRunner {
    /// 從功能目錄建立
    pub fn new<P: AsRef<Path>>(features_dir: P, config: ExecutorConfig) -> Result<Self> {
        let features_dir = features_dir.as_ref().to_path_buf();

        if !features_dir.exists() {
            bail!("功能目錄不存在：{}", features_dir.display());
        }

        let auto_continue = config.auto_continue;
        let executor = Executor::new(config);

        Ok(Self {
            features_dir,
            features: Vec::new(),
            executor,
            term: Term::stdout(),
            auto_continue,
        })
    }

    /// 從順序檔案載入功能列表
    pub fn load_features_from_order_file<P: AsRef<Path>>(&mut self, order_file: P) -> Result<()> {
        let order_file = order_file.as_ref();

        if !order_file.exists() {
            bail!("順序檔案不存在：{}", order_file.display());
        }

        let content = std::fs::read_to_string(order_file)
            .with_context(|| format!("無法讀取順序檔案：{}", order_file.display()))?;

        self.features.clear();

        for line in content.lines() {
            let feature_key = line.trim();
            if feature_key.is_empty() {
                continue;
            }

            let feature_dir = self.features_dir.join(feature_key);
            if !feature_dir.exists() {
                eprintln!(
                    "{} 功能目錄不存在，跳過：{}",
                    style("[警告]").yellow(),
                    feature_dir.display()
                );
                continue;
            }

            let info = FeatureInfo::load_from_dir(&feature_dir, feature_key)?;
            self.features.push(info);
        }

        Ok(())
    }

    /// 掃描功能目錄載入功能列表
    pub fn scan_features(&mut self) -> Result<()> {
        self.features.clear();

        for entry in std::fs::read_dir(&self.features_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let feature_key = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 檢查是否有提示檔案
            let prompt_file = path.join("01_requirements_and_delivery.md");
            if !prompt_file.exists() {
                continue;
            }

            let info = FeatureInfo::load_from_dir(&path, feature_key)?;
            self.features.push(info);
        }

        // 按名稱排序
        self.features
            .sort_by(|a, b| a.feature_key.cmp(&b.feature_key));

        Ok(())
    }

    /// 顯示歡迎訊息
    pub fn show_welcome(&self) {
        self.term.clear_screen().ok();

        println!();
        println!(
            "{}",
            style("╔════════════════════════════════════════════════════════════╗")
                .cyan()
                .bold()
        );
        println!(
            "{}",
            style("║       Claude Code Prompt Runner - 交互式執行器            ║")
                .cyan()
                .bold()
        );
        println!(
            "{}",
            style("╚════════════════════════════════════════════════════════════╝")
                .cyan()
                .bold()
        );
        println!();
    }

    /// 顯示功能狀態總覽
    pub fn show_status_overview(&self) {
        println!("{}", style("功能狀態總覽:").bold().underlined());
        println!();

        for (idx, feature) in self.features.iter().enumerate() {
            let status_icon = match feature.status {
                FeatureStatus::Ready => style("✓").green(),
                FeatureStatus::Unknown => style("?").dim(),
                _ => style("○").yellow(),
            };

            let progress_str = match feature.progress.last_done {
                Step::None => style("未開始").dim(),
                Step::P1 => style("P1").yellow(),
                Step::P2 => style("P2").yellow(),
                Step::P3 => style("P3").yellow(),
                Step::P4 => style("P4").green(),
            };

            println!(
                "  {} {} {} [{}] {}",
                style(format!("{:2}.", idx + 1)).dim(),
                status_icon,
                progress_str,
                style(&feature.status.to_string()).cyan(),
                feature.feature_key
            );
        }

        println!();
    }

    /// 顯示快速命令幫助
    pub fn show_quick_help(&self) {
        println!();
        println!("{}", style("快速命令:").bold().underlined());
        println!();
        println!(
            "  {}  {} - 執行指定功能（編號或名稱）",
            style("g <n>").green().bold(),
            style("go").dim()
        );
        println!(
            "  {}  {} - 查看功能詳情",
            style("s <n>").cyan().bold(),
            style("show").dim()
        );
        println!(
            "  {}      {} - 執行所有功能",
            style("a").yellow().bold(),
            style("all").dim()
        );
        println!(
            "  {}      {} - 重新載入功能列表",
            style("r").blue().bold(),
            style("reload").dim()
        );
        println!(
            "  {}      {} - 離開程式",
            style("q").red().bold(),
            style("quit").dim()
        );
        println!(
            "  {}      {} - 顯示此幫助",
            style("h").magenta().bold(),
            style("help").dim()
        );
        println!();
        println!("{}", style("範例:").dim());
        println!("  {} - 執行第 1 個功能", style("g 1").green());
        println!("  {} - 執行名稱包含 'auth' 的功能", style("g auth").green());
        println!("  {} - 直接輸入編號也可執行", style("1").green());
        println!();
        println!("{}", style("直接按 Enter 進入選單模式").dim());
        println!();
    }

    /// 根據選擇器找到功能索引
    pub fn find_feature_by_selector(&self, selector: &FeatureSelector) -> Option<usize> {
        match selector {
            FeatureSelector::Index(n) => {
                // 1-based index
                if *n >= 1 && *n <= self.features.len() {
                    Some(*n - 1)
                } else {
                    None
                }
            }
            FeatureSelector::Name(name) => {
                let name_lower = name.to_lowercase();

                // 精確匹配
                if let Some(idx) = self
                    .features
                    .iter()
                    .position(|f| f.feature_key.to_lowercase() == name_lower)
                {
                    return Some(idx);
                }

                // 部分匹配（從開頭）
                if let Some(idx) = self
                    .features
                    .iter()
                    .position(|f| f.feature_key.to_lowercase().starts_with(&name_lower))
                {
                    return Some(idx);
                }

                // 包含匹配
                self.features
                    .iter()
                    .position(|f| f.feature_key.to_lowercase().contains(&name_lower))
            }
        }
    }

    /// 提示輸入命令
    pub fn prompt_command(&self) -> Result<Option<QuickCommand>> {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("命令 (h=幫助, Enter=選單)")
            .allow_empty(true)
            .interact_text()?;

        Ok(QuickCommand::parse(&input))
    }

    /// 主選單
    pub fn main_menu(&self) -> Result<MainMenuChoice> {
        let choices = vec![
            "🚀 執行所有功能（自動模式）",
            "📋 選擇功能執行",
            "🔍 查看功能詳情",
            "🔄 重新載入功能列表",
            "❌ 離開",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇操作")
            .items(&choices)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => MainMenuChoice::RunAll,
            1 => MainMenuChoice::SelectFeatures,
            2 => MainMenuChoice::ViewDetails,
            3 => MainMenuChoice::Reload,
            4 => MainMenuChoice::Exit,
            _ => MainMenuChoice::Exit,
        })
    }

    /// 選擇功能
    pub fn select_features(&self) -> Result<Vec<usize>> {
        let items: Vec<String> = self
            .features
            .iter()
            .map(|f| {
                let status = match f.progress.last_done {
                    Step::None => "🔵",
                    Step::P4 => "✅",
                    _ => "🟡",
                };
                format!("{} {} ({})", status, f.feature_key, f.progress.last_done)
            })
            .collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("選擇要執行的功能（空白鍵選擇，Enter 確認）")
            .items(&items)
            .interact()?;

        Ok(selections)
    }

    /// 選擇單一功能
    pub fn select_single_feature(&self) -> Result<Option<usize>> {
        let items: Vec<String> = self
            .features
            .iter()
            .map(|f| {
                format!(
                    "{} | {} | {}",
                    f.feature_key, f.progress.last_done, f.status
                )
            })
            .collect();

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("搜尋並選擇功能")
            .items(&items)
            .interact_opt()?;

        Ok(selection)
    }

    /// 選擇起始步驟
    pub fn select_start_step(&self, feature: &FeatureInfo) -> Result<Step> {
        let next_step = feature.progress.next_step();

        // 如果已經完成所有步驟
        if next_step.is_none() {
            let restart = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("此功能已完成所有步驟。是否要重新執行？")
                .default(false)
                .interact()?;

            if restart {
                return self.select_specific_step();
            } else {
                bail!("用戶取消");
            }
        }

        // 顯示當前進度
        println!();
        println!(
            "{} 當前進度: {}",
            style("[資訊]").blue(),
            feature.progress.last_done
        );

        if let Some(next) = next_step {
            println!(
                "{} 下一步驟: {} - {}",
                style("[資訊]").blue(),
                next,
                next.description()
            );
        }

        let choices = vec![
            format!(
                "從下一步驟繼續 ({})",
                next_step.map(|s| s.as_str()).unwrap_or("完成")
            ),
            "選擇特定步驟開始".to_string(),
            "取消".to_string(),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("選擇執行方式")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok(next_step.unwrap_or(Step::P1)),
            1 => self.select_specific_step(),
            _ => bail!("用戶取消"),
        }
    }

    /// 選擇特定步驟
    pub fn select_specific_step(&self) -> Result<Step> {
        let items: Vec<String> = Step::ALL
            .iter()
            .map(|s| format!("{} - {}", s, s.description()))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("選擇要執行的步驟")
            .items(&items)
            .default(0)
            .interact()?;

        Ok(Step::ALL[selection])
    }

    /// 顯示功能詳情
    pub fn show_feature_details(&self, idx: usize) -> Result<()> {
        let feature = &self.features[idx];

        println!();
        println!("{}", style("═".repeat(60)).cyan());
        println!(
            "{} {}",
            style("功能:").bold(),
            style(&feature.feature_key).green().bold()
        );
        println!("{}", style("═".repeat(60)).cyan());
        println!();

        println!(
            "  {} {}",
            style("目錄:").dim(),
            feature.feature_dir.display()
        );
        println!("  {} {}", style("進度:").dim(), feature.progress.last_done);
        println!("  {} {}", style("狀態:").dim(), feature.status);

        if let Some(ref sid) = feature.progress.session_id {
            println!("  {} {}", style("Session:").dim(), sid);
        }

        println!();
        println!("{}", style("可用提示檔案:").underlined());

        for step in Step::ALL {
            if let Some(prompt_file) = feature.prompt_file(step) {
                let exists = prompt_file.exists();
                let icon = if exists { "✓" } else { "✗" };
                let icon_style = if exists {
                    style(icon).green()
                } else {
                    style(icon).red()
                };

                println!(
                    "  {} {} - {}",
                    icon_style,
                    step,
                    prompt_file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
            }
        }

        println!();
        Ok(())
    }

    /// 執行單一功能
    pub fn run_feature(&mut self, idx: usize, start_step: Option<Step>) -> Result<()> {
        let feature = &mut self.features[idx];

        let start = start_step.unwrap_or_else(|| feature.progress.next_step().unwrap_or(Step::P1));

        println!();
        println!(
            "{} 開始執行功能: {}",
            style("[開始]").green().bold(),
            style(&feature.feature_key).cyan().bold()
        );

        // 取得要執行的步驟列表
        let steps_to_run: Vec<Step> = Step::ALL.iter().copied().filter(|s| *s >= start).collect();

        for step in steps_to_run {
            // 檢查提示檔案是否存在
            let prompt_file = match feature.prompt_file(step) {
                Some(f) if f.exists() => f,
                _ => {
                    println!(
                        "{} 步驟 {} 的提示檔案不存在，跳過",
                        style("[跳過]").yellow(),
                        step
                    );
                    continue;
                }
            };

            // 判斷是否需要 resume
            let resume_session = if step.needs_resume() {
                feature.progress.session_id.as_deref()
            } else {
                None
            };

            // 如果需要 resume 但沒有 session_id
            if step.needs_resume() && resume_session.is_none() {
                bail!(
                    "步驟 {} 需要 session_id，但找不到。請從 P1 開始執行。",
                    step
                );
            }

            // 執行步驟
            let result = self.executor.run_step(
                &feature.feature_key,
                step,
                &prompt_file,
                &feature.logs_dir(),
                resume_session,
            )?;

            // 更新進度
            if result.success {
                feature.progress.mark_done(step, result.session_id);
                feature.progress.save_to_file(feature.progress_file())?;

                // 更新狀態
                feature.status = read_state_status(feature.state_file())?;

                println!(
                    "{} 進度已保存: {}",
                    style("[保存]").blue(),
                    feature.progress.last_done
                );
            } else {
                bail!("步驟 {} 執行失敗", step);
            }

            // 詢問是否繼續下一步（除非設定為自動繼續）
            if step != Step::P4 && !self.auto_continue {
                let continue_next = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("是否繼續執行下一步驟？")
                    .default(true)
                    .interact()?;

                if !continue_next {
                    println!("{} 用戶中斷執行", style("[中斷]").yellow());
                    break;
                }
            }
        }

        // 檢查最終狀態
        if feature.status.is_ready() {
            println!();
            println!(
                "{} 功能 {} 已完成！狀態: READY",
                style("[完成]").green().bold(),
                style(&feature.feature_key).cyan()
            );
        } else {
            println!();
            println!(
                "{} 功能 {} 尚未就緒。當前狀態: {}",
                style("[注意]").yellow().bold(),
                style(&feature.feature_key).cyan(),
                feature.status
            );
        }

        Ok(())
    }

    /// 執行多個功能
    pub fn run_features(&mut self, indices: &[usize]) -> Result<()> {
        let total = indices.len();

        for (i, &idx) in indices.iter().enumerate() {
            println!();
            println!(
                "{} 執行進度: {}/{}",
                style("[進度]").blue().bold(),
                i + 1,
                total
            );

            // 重新載入功能資訊
            let feature_key = self.features[idx].feature_key.clone();
            let feature_dir = self.features[idx].feature_dir.clone();
            self.features[idx] = FeatureInfo::load_from_dir(&feature_dir, &feature_key)?;

            // 檢查是否已完成
            if self.features[idx].status.is_ready() {
                println!(
                    "{} 功能 {} 已就緒，跳過",
                    style("[跳過]").dim(),
                    style(&feature_key).cyan()
                );
                continue;
            }

            self.run_feature(idx, None)?;
        }

        println!();
        println!("{} 所有選定功能執行完成", style("[完成]").green().bold());

        Ok(())
    }

    /// 執行所有功能
    pub fn run_all(&mut self) -> Result<()> {
        let indices: Vec<usize> = (0..self.features.len()).collect();
        self.run_features(&indices)
    }

    /// 主執行迴圈
    pub fn run_interactive(&mut self) -> Result<()> {
        self.show_welcome();

        // 檢查 Claude CLI
        if let Err(e) = self.executor.check_availability() {
            println!("{} Claude CLI 不可用: {}", style("[錯誤]").red().bold(), e);
            return Ok(());
        }

        println!("{} Claude CLI 已就緒", style("[確認]").green());

        // 顯示快速命令提示
        println!();
        println!(
            "{} 輸入 {} 查看快速命令",
            style("[提示]").blue(),
            style("h").magenta().bold()
        );

        loop {
            println!();
            self.show_status_overview();

            // 先嘗試快速命令輸入
            let cmd = self.prompt_command()?;

            match cmd {
                Some(QuickCommand::Go(selector)) => {
                    if let Some(idx) = self.find_feature_by_selector(&selector) {
                        let feature_key = self.features[idx].feature_key.clone();
                        println!(
                            "{} 執行功能: {}",
                            style("[選擇]").green(),
                            style(&feature_key).cyan().bold()
                        );
                        if let Err(e) = self.run_feature(idx, None) {
                            println!("{} {}", style("[錯誤]").red().bold(), e);
                        }
                    } else {
                        println!("{} 找不到符合的功能: {:?}", style("[錯誤]").red(), selector);
                    }
                }
                Some(QuickCommand::Show(selector)) => {
                    if let Some(idx) = self.find_feature_by_selector(&selector) {
                        self.show_feature_details(idx)?;

                        let run_it = Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt("是否執行此功能？")
                            .default(false)
                            .interact()?;

                        if run_it {
                            let step = self.select_start_step(&self.features[idx])?;
                            if let Err(e) = self.run_feature(idx, Some(step)) {
                                println!("{} {}", style("[錯誤]").red().bold(), e);
                            }
                        }
                    } else {
                        println!("{} 找不到符合的功能: {:?}", style("[錯誤]").red(), selector);
                    }
                }
                Some(QuickCommand::All) => {
                    if let Err(e) = self.run_all() {
                        println!("{} {}", style("[錯誤]").red().bold(), e);
                    }
                }
                Some(QuickCommand::Reload) => {
                    let order_file = self.features_dir.join("FEATURE_ORDER.txt");
                    if order_file.exists() {
                        self.load_features_from_order_file(&order_file)?;
                    } else {
                        self.scan_features()?;
                    }
                    println!(
                        "{} 已載入 {} 個功能",
                        style("[重新載入]").green(),
                        self.features.len()
                    );
                }
                Some(QuickCommand::Quit) => {
                    println!("{} 再見！", style("[離開]").cyan());
                    break;
                }
                Some(QuickCommand::Help) => {
                    self.show_quick_help();
                }
                Some(QuickCommand::Menu) | None => {
                    // 使用傳統選單模式
                    match self.main_menu()? {
                        MainMenuChoice::RunAll => {
                            if let Err(e) = self.run_all() {
                                println!("{} {}", style("[錯誤]").red().bold(), e);
                            }
                        }
                        MainMenuChoice::SelectFeatures => {
                            let selections = self.select_features()?;
                            if !selections.is_empty() {
                                if let Err(e) = self.run_features(&selections) {
                                    println!("{} {}", style("[錯誤]").red().bold(), e);
                                }
                            }
                        }
                        MainMenuChoice::ViewDetails => {
                            if let Some(idx) = self.select_single_feature()? {
                                self.show_feature_details(idx)?;

                                let run_it = Confirm::with_theme(&ColorfulTheme::default())
                                    .with_prompt("是否執行此功能？")
                                    .default(false)
                                    .interact()?;

                                if run_it {
                                    let step = self.select_start_step(&self.features[idx])?;
                                    if let Err(e) = self.run_feature(idx, Some(step)) {
                                        println!("{} {}", style("[錯誤]").red().bold(), e);
                                    }
                                }
                            }
                        }
                        MainMenuChoice::Reload => {
                            let order_file = self.features_dir.join("FEATURE_ORDER.txt");
                            if order_file.exists() {
                                self.load_features_from_order_file(&order_file)?;
                            } else {
                                self.scan_features()?;
                            }
                            println!(
                                "{} 已載入 {} 個功能",
                                style("[重新載入]").green(),
                                self.features.len()
                            );
                        }
                        MainMenuChoice::Exit => {
                            println!("{} 再見！", style("[離開]").cyan());
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 主選單選項
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuChoice {
    RunAll,
    SelectFeatures,
    ViewDetails,
    Reload,
    Exit,
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_mode() {
        assert_ne!(RunMode::Auto, RunMode::Interactive);
    }
}
