mod config;
mod engines;
mod scanner;
mod types;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use config::{load_builder_config, save_builder_config, BuilderConfig};
use engines::{BuildEngine, BuildahEngine, DockerEngine};
use scanner::scan_dockerfiles;
use std::path::PathBuf;
use types::{Architecture, BuildContext, EngineType};

/// Execute Container Builder
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::CONTAINER_BUILDER_HEADER));

    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            console.error(&crate::tr!(
                keys::CONTAINER_BUILDER_CURRENT_DIR_FAILED,
                error = err
            ));
            return;
        }
    };

    // Load saved config
    let mut builder_config = load_builder_config().unwrap_or_default();

    // Step 1: Select build engine
    let engine_type = match select_engine(&prompts, &console) {
        Some(engine) => engine,
        None => {
            console.warning(i18n::t(keys::CONTAINER_BUILDER_CANCELLED));
            return;
        }
    };

    let engine: Box<dyn BuildEngine> = match engine_type {
        EngineType::Docker => Box::new(DockerEngine),
        EngineType::Buildah => Box::new(BuildahEngine),
    };

    // Verify engine is available
    if !engine.is_available() {
        console.error(&crate::tr!(
            keys::CONTAINER_BUILDER_ENGINE_NOT_FOUND,
            engine = engine.name()
        ));
        return;
    }

    console.success(&crate::tr!(
        keys::CONTAINER_BUILDER_USING_ENGINE,
        engine = engine.name()
    ));

    // Step 2: Select Dockerfile
    console.info(i18n::t(keys::CONTAINER_BUILDER_SCANNING_DOCKERFILES));
    let dockerfiles = scan_dockerfiles(&current_dir);

    if dockerfiles.is_empty() {
        console.error(i18n::t(keys::CONTAINER_BUILDER_NO_DOCKERFILE));
        return;
    }

    let dockerfile = match select_dockerfile(&prompts, &console, &dockerfiles) {
        Some(path) => path,
        None => {
            console.warning(i18n::t(keys::CONTAINER_BUILDER_CANCELLED));
            return;
        }
    };

    console.info(&crate::tr!(
        keys::CONTAINER_BUILDER_SELECTED_DOCKERFILE,
        path = dockerfile.display()
    ));

    // Step 3: Select architecture
    let architecture = match select_architecture(&prompts, &console) {
        Some(arch) => arch,
        None => {
            console.warning(i18n::t(keys::CONTAINER_BUILDER_CANCELLED));
            return;
        }
    };

    console.info(&crate::tr!(
        keys::CONTAINER_BUILDER_SELECTED_ARCH,
        arch = architecture.display_name()
    ));

    // Step 4: Input image name/tag
    let (image_name, tag) = match input_image_info(&prompts, &console, &mut builder_config) {
        Some((name, tag)) => (name, tag),
        None => {
            console.warning(i18n::t(keys::CONTAINER_BUILDER_CANCELLED));
            return;
        }
    };

    // Step 5: Ask about push
    let push_config = ask_push_config(&prompts, &console, &mut builder_config);

    // Save config for future use
    if let Err(err) = save_builder_config(&builder_config) {
        console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err));
    }

    // Build context
    let context_dir = dockerfile.parent().unwrap_or(&current_dir).to_path_buf();
    let build_context = BuildContext {
        dockerfile: dockerfile.clone(),
        context_dir,
        image_name: image_name.clone(),
        tag: tag.clone(),
        architecture: architecture.clone(),
        push: push_config.is_some(),
        registry: push_config.clone(),
    };

    // Confirm build
    console.blank_line();
    console.info(i18n::t(keys::CONTAINER_BUILDER_BUILD_SUMMARY));
    console.list_item("Engine:", engine.name());
    console.list_item("Dockerfile:", &dockerfile.display().to_string());
    console.list_item("Architecture:", architecture.display_name());
    console.list_item("Image:", &format!("{}:{}", image_name, tag));
    if let Some(ref registry) = push_config {
        console.list_item("Push to:", registry);
    }
    console.blank_line();

    if !prompts.confirm_with_options(i18n::t(keys::CONTAINER_BUILDER_CONFIRM_BUILD), true) {
        console.warning(i18n::t(keys::CONTAINER_BUILDER_CANCELLED));
        return;
    }

    // Execute build
    console.blank_line();
    console.info(i18n::t(keys::CONTAINER_BUILDER_BUILDING));

    match engine.build(&build_context) {
        Ok(result) => {
            if result.success {
                console.success(i18n::t(keys::CONTAINER_BUILDER_BUILD_SUCCESS));

                // Push if requested
                if build_context.push {
                    console.info(i18n::t(keys::CONTAINER_BUILDER_PUSHING));
                    match engine.push(&build_context) {
                        Ok(push_result) => {
                            if push_result.success {
                                console.success(i18n::t(keys::CONTAINER_BUILDER_PUSH_SUCCESS));
                            } else {
                                console.error(i18n::t(keys::CONTAINER_BUILDER_PUSH_FAILED));
                            }
                        }
                        Err(err) => {
                            console.error(&crate::tr!(
                                keys::CONTAINER_BUILDER_PUSH_ERROR,
                                error = err
                            ));
                        }
                    }
                }
            } else {
                console.error(i18n::t(keys::CONTAINER_BUILDER_BUILD_FAILED));
            }
        }
        Err(err) => {
            console.error(&crate::tr!(
                keys::CONTAINER_BUILDER_BUILD_ERROR,
                error = err
            ));
        }
    }
}

fn select_engine(prompts: &Prompts, _console: &Console) -> Option<EngineType> {
    let options = [
        format!(
            "Docker — {}",
            i18n::t(keys::CONTAINER_BUILDER_ENGINE_DOCKER_DESC)
        ),
        format!(
            "Buildah — {}",
            i18n::t(keys::CONTAINER_BUILDER_ENGINE_BUILDAH_DESC)
        ),
    ];
    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    prompts
        .select(i18n::t(keys::CONTAINER_BUILDER_SELECT_ENGINE), &option_refs)
        .map(|idx| match idx {
            0 => EngineType::Docker,
            _ => EngineType::Buildah,
        })
}

fn select_dockerfile(
    prompts: &Prompts,
    console: &Console,
    dockerfiles: &[PathBuf],
) -> Option<PathBuf> {
    console.info(&crate::tr!(
        keys::CONTAINER_BUILDER_FOUND_DOCKERFILES,
        count = dockerfiles.len()
    ));

    let options: Vec<String> = dockerfiles
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    prompts
        .select(
            i18n::t(keys::CONTAINER_BUILDER_SELECT_DOCKERFILE),
            &option_refs,
        )
        .map(|idx| dockerfiles[idx].clone())
}

fn select_architecture(prompts: &Prompts, _console: &Console) -> Option<Architecture> {
    let architectures = Architecture::all();
    let options: Vec<String> = architectures
        .iter()
        .map(|arch| format!("{} — {}", arch.display_name(), arch.description()))
        .collect();
    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    prompts
        .select(i18n::t(keys::CONTAINER_BUILDER_SELECT_ARCH), &option_refs)
        .map(|idx| architectures[idx].clone())
}

fn input_image_info(
    prompts: &Prompts,
    _console: &Console,
    config: &mut BuilderConfig,
) -> Option<(String, String)> {
    use dialoguer::{theme::ColorfulTheme, Input};

    // Image name
    let image_name: String = if config.recent_images.is_empty() {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_IMAGE_NAME))
            .interact_text()
            .ok()?
    } else {
        // Offer recent images or new input
        let mut options: Vec<String> = config.recent_images.clone();
        options.push(i18n::t(keys::CONTAINER_BUILDER_NEW_IMAGE).to_string());
        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let idx = prompts.select(
            i18n::t(keys::CONTAINER_BUILDER_SELECT_IMAGE_NAME),
            &option_refs,
        )?;

        if idx == options.len() - 1 {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_IMAGE_NAME))
                .interact_text()
                .ok()?
        } else {
            options[idx].clone()
        }
    };

    // Remember image name
    if !config.recent_images.contains(&image_name) {
        config.recent_images.insert(0, image_name.clone());
        if config.recent_images.len() > 10 {
            config.recent_images.truncate(10);
        }
    }

    // Tag
    let tag: String = if config.recent_tags.is_empty() {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_TAG))
            .default("latest".to_string())
            .interact_text()
            .ok()?
    } else {
        let mut options: Vec<String> = config.recent_tags.clone();
        options.push(i18n::t(keys::CONTAINER_BUILDER_NEW_TAG).to_string());
        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let idx = prompts.select(i18n::t(keys::CONTAINER_BUILDER_SELECT_TAG), &option_refs)?;

        if idx == options.len() - 1 {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_TAG))
                .default("latest".to_string())
                .interact_text()
                .ok()?
        } else {
            options[idx].clone()
        }
    };

    // Remember tag
    if !config.recent_tags.contains(&tag) {
        config.recent_tags.insert(0, tag.clone());
        if config.recent_tags.len() > 10 {
            config.recent_tags.truncate(10);
        }
    }

    Some((image_name, tag))
}

fn ask_push_config(
    prompts: &Prompts,
    _console: &Console,
    config: &mut BuilderConfig,
) -> Option<String> {
    use dialoguer::{theme::ColorfulTheme, Input};

    if !prompts.confirm(i18n::t(keys::CONTAINER_BUILDER_ASK_PUSH)) {
        return None;
    }

    let registry: String = if config.recent_registries.is_empty() {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_REGISTRY))
            .interact_text()
            .ok()?
    } else {
        let mut options: Vec<String> = config.recent_registries.clone();
        options.push(i18n::t(keys::CONTAINER_BUILDER_NEW_REGISTRY).to_string());
        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let idx = prompts.select(
            i18n::t(keys::CONTAINER_BUILDER_SELECT_REGISTRY),
            &option_refs,
        )?;

        if idx == options.len() - 1 {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(i18n::t(keys::CONTAINER_BUILDER_INPUT_REGISTRY))
                .interact_text()
                .ok()?
        } else {
            options[idx].clone()
        }
    };

    // Remember registry
    if !config.recent_registries.contains(&registry) {
        config.recent_registries.insert(0, registry.clone());
        if config.recent_registries.len() > 10 {
            config.recent_registries.truncate(10);
        }
    }

    Some(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_all() {
        let archs = Architecture::all();
        assert!(archs.len() >= 4);
    }
}
