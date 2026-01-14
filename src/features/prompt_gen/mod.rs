//! Prompt Generator - LLM 4 步驟提示生成與執行
//!
//! 提供功能：
//! - 從 YAML/JSON 規格生成 4 步驟提示檔案
//! - 交互式或自動執行提示
//! - 查看功能執行狀態

mod executor;
mod interactive;
mod loader;
mod models;
mod progress;
mod renderer;
mod templates;
mod writer;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::PathBuf;

use executor::{CliType, ExecutorConfig};
use interactive::InteractiveRunner;
use loader::SpecLoader;
use models::ProjectType;
use progress::{FeatureInfo, Step};
use renderer::render_all;
use templates::yaml_gen_prompt::generate_yaml_prompt;
use writer::PromptWriter;

/// 執行 Prompt Generator 功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::PROMPT_GEN_HEADER));

    loop {
        let options = vec![
            i18n::t(keys::PROMPT_GEN_ACTION_GENERATE),
            i18n::t(keys::PROMPT_GEN_ACTION_RUN),
            i18n::t(keys::PROMPT_GEN_ACTION_STATUS),
            i18n::t(keys::PROMPT_GEN_ACTION_VALIDATE),
            i18n::t(keys::PROMPT_GEN_ACTION_YAML_PROMPT),
            i18n::t(keys::MENU_EXIT),
        ];

        let selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::PROMPT_GEN_SELECT_ACTION))
            .items(&options)
            .default(0)
            .interact_opt()
        {
            Ok(Some(sel)) => sel,
            Ok(None) | Err(_) => {
                console.warning(i18n::t(keys::PROMPT_GEN_CANCELLED));
                return;
            }
        };

        match selection {
            0 => {
                if let Err(e) = cmd_generate(&console, &prompts) {
                    console.error(&format!("{}", e));
                }
            }
            1 => {
                if let Err(e) = cmd_run(&console) {
                    console.error(&format!("{}", e));
                }
            }
            2 => {
                if let Err(e) = cmd_status(&console, &prompts) {
                    console.error(&format!("{}", e));
                }
            }
            3 => {
                if let Err(e) = cmd_validate(&console) {
                    console.error(&format!("{}", e));
                }
            }
            4 => {
                if let Err(e) = cmd_yaml_prompt(&console, &prompts) {
                    console.error(&format!("{}", e));
                }
            }
            5 => {
                return;
            }
            _ => unreachable!(),
        }

        println!();
    }
}

/// 生成命令
fn cmd_generate(console: &Console, prompts: &Prompts) -> Result<()> {
    // 取得規格檔案路徑
    let spec_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_SPEC_FILE))
        .default("features.yaml".to_string())
        .interact_text()
        .context("Failed to read input")?;

    let spec_path = PathBuf::from(&spec_file);
    if !spec_path.exists() {
        console.error(&crate::tr!(
            keys::PROMPT_GEN_FILE_NOT_FOUND,
            path = spec_file
        ));
        return Ok(());
    }

    // 取得輸出目錄
    let out_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_OUTPUT_DIR))
        .default("features".to_string())
        .interact_text()
        .context("Failed to read input")?;

    // 確認是否覆蓋
    let overwrite =
        prompts.confirm_with_options(i18n::t(keys::PROMPT_GEN_CONFIRM_OVERWRITE), false);

    console.info(i18n::t(keys::PROMPT_GEN_GENERATING));

    // 載入規格
    let spec = SpecLoader::load_from_path(&spec_path)
        .with_context(|| format!("Failed to load spec file: {}", spec_path.display()))?;

    console.info(&crate::tr!(
        keys::PROMPT_GEN_LOADED_FEATURES,
        count = spec.features.len()
    ));

    // 渲染所有提示
    let all_prompts = render_all(&spec.features);

    // 建立輸出目錄
    let out_base = if PathBuf::from(&out_dir).is_absolute() {
        PathBuf::from(&out_dir)
    } else {
        std::env::current_dir()?.join(&out_dir)
    };

    std::fs::create_dir_all(&out_base)
        .with_context(|| format!("Failed to create output directory: {}", out_base.display()))?;

    // 寫入檔案
    let writer = PromptWriter::new(out_base.clone(), overwrite);

    for feature_prompts in &all_prompts {
        writer.write_feature_prompts(feature_prompts)?;
        console.success_item(&crate::tr!(
            keys::PROMPT_GEN_FEATURE_GENERATED,
            key = feature_prompts.feature_key.as_str()
        ));
    }

    // 生成順序檔案
    let order_content = spec
        .features
        .iter()
        .map(|f| f.feature_key.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    let order_path = out_base.join("FEATURE_ORDER.txt");
    std::fs::write(&order_path, &order_content)
        .with_context(|| format!("Failed to write order file: {}", order_path.display()))?;

    console.success(&crate::tr!(
        keys::PROMPT_GEN_GENERATED,
        count = spec.features.len(),
        path = out_base.display()
    ));

    Ok(())
}

/// 執行命令
fn cmd_run(console: &Console) -> Result<()> {
    // 選擇 CLI 類型
    let cli_options: Vec<String> = CliType::ALL
        .iter()
        .map(|c| c.display_name().to_string())
        .collect();

    let cli_selection = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_SELECT_CLI))
        .items(&cli_options)
        .default(0)
        .interact_opt()
    {
        Ok(Some(sel)) => sel,
        Ok(None) | Err(_) => {
            console.warning(i18n::t(keys::PROMPT_GEN_CANCELLED));
            return Ok(());
        }
    };

    let cli_type = CliType::from_index(cli_selection).unwrap_or_default();
    console.info(&crate::tr!(
        keys::PROMPT_GEN_USING_CLI,
        cli = cli_type.display_name()
    ));

    // 取得功能目錄
    let features_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_FEATURES_DIR))
        .default("features".to_string())
        .interact_text()
        .context("Failed to read input")?;

    let features_path = if PathBuf::from(&features_dir).is_absolute() {
        PathBuf::from(&features_dir)
    } else {
        std::env::current_dir()?.join(&features_dir)
    };

    if !features_path.exists() {
        console.error(&crate::tr!(
            keys::PROMPT_GEN_DIR_NOT_FOUND,
            path = features_dir
        ));
        return Ok(());
    }

    console.info(i18n::t(keys::PROMPT_GEN_RUNNING));

    // 建立執行器配置
    let config = ExecutorConfig {
        cli_type,
        skip_permissions: true,
        output_format: executor::OutputFormat::StreamJson,
        auto_continue: true,
    };

    // 建立交互式執行器
    let mut runner = InteractiveRunner::new(&features_path, config)?;

    // 載入功能列表
    let order_file = features_path.join("FEATURE_ORDER.txt");
    if order_file.exists() {
        runner.load_features_from_order_file(&order_file)?;
    } else {
        runner.scan_features()?;
    }

    // 執行交互式模式
    runner.run_interactive()?;

    Ok(())
}

/// 狀態命令
fn cmd_status(console: &Console, _prompts: &Prompts) -> Result<()> {
    // 取得功能目錄
    let features_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_FEATURES_DIR))
        .default("features".to_string())
        .interact_text()
        .context("Failed to read input")?;

    let features_path = if PathBuf::from(&features_dir).is_absolute() {
        PathBuf::from(&features_dir)
    } else {
        std::env::current_dir()?.join(&features_dir)
    };

    if !features_path.exists() {
        console.error(&crate::tr!(
            keys::PROMPT_GEN_DIR_NOT_FOUND,
            path = features_dir
        ));
        return Ok(());
    }

    // 載入功能列表
    let order_file = features_path.join("FEATURE_ORDER.txt");
    let mut features = Vec::new();

    if order_file.exists() {
        let content = std::fs::read_to_string(&order_file)?;
        for line in content.lines() {
            let feature_key = line.trim();
            if feature_key.is_empty() {
                continue;
            }
            let feature_dir = features_path.join(feature_key);
            if feature_dir.exists() {
                if let Ok(info) = FeatureInfo::load_from_dir(&feature_dir, feature_key) {
                    features.push(info);
                }
            }
        }
    } else {
        // 掃描目錄
        for entry in std::fs::read_dir(&features_path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let feature_key = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let prompt_file = path.join("01_requirements_and_delivery.md");
            if prompt_file.exists() {
                if let Ok(info) = FeatureInfo::load_from_dir(&path, feature_key) {
                    features.push(info);
                }
            }
        }
        features.sort_by(|a, b| a.feature_key.cmp(&b.feature_key));
    }

    // 顯示狀態
    println!();
    println!(
        "{}",
        style("╔════════════════════════════════════════════════════════════╗")
            .cyan()
            .bold()
    );
    println!(
        "{}",
        style("║                    功能狀態總覽                            ║")
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

    let mut completed = 0;
    let mut in_progress = 0;
    let mut not_started = 0;

    for (idx, feature) in features.iter().enumerate() {
        let status_icon = if feature.status.is_ready() {
            completed += 1;
            style("✓").green()
        } else if feature.progress.last_done == Step::None {
            not_started += 1;
            style("○").dim()
        } else {
            in_progress += 1;
            style("◐").yellow()
        };

        let progress_str = format!("{:>4}", feature.progress.last_done.as_str());
        let progress_styled = match feature.progress.last_done {
            Step::None => style(progress_str).dim(),
            Step::P4 => style(progress_str).green(),
            _ => style(progress_str).yellow(),
        };

        println!(
            "  {:2}. {} {} [{:>25}] {}",
            idx + 1,
            status_icon,
            progress_styled,
            style(feature.status.to_string()).cyan(),
            feature.feature_key
        );
    }

    println!();
    println!("{}", style("─".repeat(60)).dim());
    println!(
        "  {}: {} | {} {} | {} {} | {} {}",
        i18n::t(keys::PROMPT_GEN_STATUS_TOTAL),
        features.len(),
        style(completed).green(),
        i18n::t(keys::PROMPT_GEN_STATUS_READY),
        style(in_progress).yellow(),
        i18n::t(keys::PROMPT_GEN_STATUS_IN_PROGRESS),
        style(not_started).dim(),
        i18n::t(keys::PROMPT_GEN_STATUS_NOT_STARTED)
    );
    println!();

    Ok(())
}

/// 驗證命令
fn cmd_validate(console: &Console) -> Result<()> {
    // 取得規格檔案路徑
    let spec_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_SPEC_FILE))
        .default("features.yaml".to_string())
        .interact_text()
        .context("Failed to read input")?;

    let spec_path = PathBuf::from(&spec_file);
    if !spec_path.exists() {
        console.error(&crate::tr!(
            keys::PROMPT_GEN_FILE_NOT_FOUND,
            path = spec_file
        ));
        return Ok(());
    }

    console.info(i18n::t(keys::PROMPT_GEN_VALIDATING));

    // 載入並驗證規格
    match SpecLoader::load_from_path(&spec_path) {
        Ok(spec) => {
            console.success(&crate::tr!(
                keys::PROMPT_GEN_VALIDATE_SUCCESS,
                count = spec.features.len()
            ));

            // 顯示功能清單
            for (idx, feature) in spec.features.iter().enumerate() {
                println!(
                    "  {:2}. {} — {}",
                    idx + 1,
                    feature.feature_key.as_str(),
                    feature.feature_name.as_str()
                );
            }
        }
        Err(e) => {
            console.error(&crate::tr!(keys::PROMPT_GEN_VALIDATE_FAILED, error = e));
        }
    }

    Ok(())
}

/// YAML Prompt 生成命令
fn cmd_yaml_prompt(console: &Console, prompts: &Prompts) -> Result<()> {
    // 選擇專案類型
    let project_type_options: Vec<String> = ProjectType::ALL
        .iter()
        .map(|pt| format!("{} - {}", pt, pt.role_description()))
        .collect();

    let project_type_selection = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_SELECT_PROJECT_TYPE))
        .items(&project_type_options)
        .default(0)
        .interact_opt()
    {
        Ok(Some(sel)) => sel,
        Ok(None) | Err(_) => {
            console.warning(i18n::t(keys::PROMPT_GEN_CANCELLED));
            return Ok(());
        }
    };

    let project_type = ProjectType::ALL[project_type_selection];

    // 是否有遠端驗證環境
    let has_verification_env = prompts.confirm_with_options(
        i18n::t(keys::PROMPT_GEN_HAS_VERIFICATION_ENV),
        project_type.typically_needs_verification_env(),
    );

    // 是否需要部署
    let needs_deployment = prompts.confirm_with_options(
        i18n::t(keys::PROMPT_GEN_NEEDS_DEPLOYMENT),
        project_type.typically_needs_deployment(),
    );

    // 自定義驗證方式
    let custom_validation: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_CUSTOM_VALIDATION))
        .allow_empty(true)
        .interact_text()
        .context("Failed to read input")?;

    let custom_validation_opt = if custom_validation.trim().is_empty() {
        None
    } else {
        Some(custom_validation.as_str())
    };

    // 生成 YAML prompt
    let prompt = generate_yaml_prompt(
        project_type,
        has_verification_env,
        needs_deployment,
        custom_validation_opt,
    );

    // 取得輸出檔案路徑
    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::PROMPT_GEN_INPUT_OUTPUT_FILE))
        .allow_empty(true)
        .interact_text()
        .context("Failed to read input")?;

    if output_file.trim().is_empty() {
        // 直接顯示
        println!();
        println!("{}", style("─".repeat(80)).dim());
        println!("{}", prompt);
        println!("{}", style("─".repeat(80)).dim());
        println!();
        console.success(i18n::t(keys::PROMPT_GEN_YAML_PROMPT_GENERATED));
    } else {
        // 寫入檔案
        let output_path = if PathBuf::from(&output_file).is_absolute() {
            PathBuf::from(&output_file)
        } else {
            std::env::current_dir()?.join(&output_file)
        };

        std::fs::write(&output_path, &prompt)
            .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

        console.success(i18n::t(keys::PROMPT_GEN_YAML_PROMPT_GENERATED));
        console.info(&format!("  {}", output_path.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // 確保模組可以編譯
    }
}
