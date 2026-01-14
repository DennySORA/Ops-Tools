//! YAML 生成 Prompt 模板
//!
//! 根據專案類型生成對應的 YAML spec prompt

#![allow(dead_code)]

use crate::features::prompt_gen::models::ProjectType;

/// 生成 YAML 的基礎 prompt 模板
pub const YAML_GEN_BASE: &str = r#"You are a senior software architect and QA / automation platform designer.

OBJECTIVE
Generate a single YAML document that defines a list of engineering features for implementation and verification.

CRITICAL OUTPUT CONTRACT (MUST FOLLOW)
1) Output MUST be valid YAML only.
   - Do NOT output Markdown.
   - Do NOT use code fences.
   - Do NOT add any prose before or after the YAML.
2) The YAML root key MUST be: features
3) features MUST be a YAML list of feature objects.
4) Every feature object MUST include the EXACT keys below (no extra keys, no missing keys):
{REQUIRED_FIELDS}

LANGUAGE REQUIREMENT (MANDATORY)
- All YAML content MUST be written in English:
  - All string values MUST be English.
  - All YAML comments (lines starting with #) MUST be English.

PROJECT TYPE: {PROJECT_TYPE}
{PROJECT_TYPE_DESCRIPTION}

{PROJECT_SPECIFIC_REQUIREMENTS}

STYLE / SPEC RIGOR REQUIREMENTS
- Use clear, testable language using must/shall/should with explicit constraints and thresholds.
- Avoid vague statements (e.g., "handle properly", "improve performance" without measurable criteria).
- Emphasize:
  - contracts / schemas
  - state machines and illegal transition guards
  - idempotency and deduplication
  - auditability
  - observability (logs/metrics/tracing)
{VERIFICATION_REQUIREMENTS}

FIELD SEMANTICS (MUST APPLY)
feature_key
- Uniquely identifies the feature in tracking systems.
- Must be unique across the entire YAML.
- Format: lower-kebab-case (alphanumeric, hyphens, underscores only).
- Recommended action prefixes: add-, enhance-, fix-, refactor-, healthcheck-, upgrade-.

feature_name
- Human-facing title.
- MUST start with one of the prefixes:
  "Add:", "Enhance:", "Fix:", "Refactor:", "Upgrade:", "Healthcheck:"
- Must be concise but specific and include key scope/constraints when relevant.

project_type
- Must be: {PROJECT_TYPE}

{OPTIONAL_FIELD_SEMANTICS}

context
- A list of English strings (or single string) describing why/what at a high level.
- Must include:
  - Goal statement and user impact
  - Why now / problem being solved
  - Scope boundaries and key constraints

requirements
- A detailed, testable specification as a list of strings (or single string).
- Must be structured using YAML comment section headings EXACTLY like:
  # --------
  # A. <title>
  # --------
- Each requirement MUST be an English string item.
{REQUIREMENTS_CONTENT}

acceptance_criteria
- A list of binary-verifiable outcomes in English (or single string).
- Must include:
{ACCEPTANCE_CONTENT}

QUALITY BAR SELF-CHECK (DO THIS BEFORE YOU OUTPUT)
{QUALITY_CHECKLIST}

NOW GENERATE THE YAML IN THE EXACT SHAPE OF THE TEMPLATE BELOW.
- Keep the same key order as the template.
- Replace placeholders with concrete, English content.
- Output YAML only.

{YAML_TEMPLATE}
"#;

/// 取得專案類型描述
pub fn get_project_type_description(project_type: ProjectType) -> &'static str {
    match project_type {
        ProjectType::Frontend => {
            "This is a FRONTEND project. Focus on UI/UX, browser compatibility, and visual consistency."
        }
        ProjectType::Backend => {
            "This is a BACKEND project. Focus on API design, data persistence, and service reliability."
        }
        ProjectType::Fullstack => {
            "This is a FULLSTACK project. Focus on end-to-end data flow, API contracts, and UI-backend integration."
        }
        ProjectType::Cli => {
            "This is a CLI TOOL project. Focus on argument parsing, exit codes, and shell integration."
        }
        ProjectType::Library => {
            "This is a LIBRARY/SDK project. Focus on API design, documentation, and semantic versioning."
        }
        ProjectType::SystemLevel => {
            "This is a SYSTEM-LEVEL project. Focus on memory safety, performance constraints, and hardware compatibility."
        }
        ProjectType::Algorithm => {
            "This is an ALGORITHM project. Focus on correctness, complexity analysis, and performance benchmarks."
        }
        ProjectType::Infra => {
            "This is an INFRASTRUCTURE project. Focus on idempotency, state management, and configuration validation."
        }
    }
}

/// 取得必填欄位列表
pub fn get_required_fields(
    project_type: ProjectType,
    has_verification_env: bool,
    needs_deployment: bool,
) -> String {
    let mut fields = vec![
        "   - feature_key",
        "   - feature_name",
        "   - project_type",
        "   - context",
        "   - requirements",
        "   - acceptance_criteria",
    ];

    if has_verification_env {
        fields.push("   - verification_url");
        fields.push("   - int_credentials (optional)");
    }

    if needs_deployment {
        fields.push("   - options.needs_deployment: true");
    }

    // 專案類型特定欄位
    match project_type {
        ProjectType::Frontend | ProjectType::Fullstack => {
            fields.push("   - options.has_verification_env: true (if applicable)");
        }
        ProjectType::Cli => {
            fields.push("   - options.test_command (optional)");
        }
        ProjectType::Library => {
            fields.push("   - options.build_command (optional)");
        }
        ProjectType::Algorithm => {
            fields.push("   - options.custom_validation (for benchmark methodology)");
        }
        _ => {}
    }

    fields.join("\n")
}

/// 取得專案類型特定需求
pub fn get_project_specific_requirements(project_type: ProjectType) -> &'static str {
    match project_type {
        ProjectType::Frontend => {
            r#"FRONTEND-SPECIFIC REQUIREMENTS
- UI MUST reflect backend/DB truth (no client-side inference as source of truth).
- Acceptance criteria MUST include Chrome DevTools: console 0 error AND network 0 failure.
- Visual direction (typography, color, hierarchy, spacing) must be defined.
- Interaction states (hover/focus/active) must be specified."#
        }
        ProjectType::Backend => {
            r#"BACKEND-SPECIFIC REQUIREMENTS
- API contracts must be documented (OpenAPI/Swagger recommended).
- Error responses must follow consistent format with error codes.
- Endpoints must be idempotent where applicable.
- Database migrations must be reversible.
- Rate limiting and authentication requirements must be specified."#
        }
        ProjectType::Fullstack => {
            r#"FULLSTACK-SPECIFIC REQUIREMENTS
- API contracts between frontend and backend must be documented.
- UI MUST reflect backend/DB truth (no client-side inference).
- End-to-end data flow must be validated.
- Chrome DevTools: console 0 error AND network 0 failure.
- Database migrations must be reversible."#
        }
        ProjectType::Cli => {
            r#"CLI-SPECIFIC REQUIREMENTS
- Argument parsing must handle all edge cases (missing args, invalid values).
- Help text (--help) must be comprehensive and accurate.
- Exit codes must follow conventions (0=success, non-zero=error).
- Stderr for errors, stdout for normal output.
- Support for stdin/stdout piping where applicable."#
        }
        ProjectType::Library => {
            r#"LIBRARY-SPECIFIC REQUIREMENTS
- Public API must be stable and well-documented.
- Breaking changes must follow semantic versioning.
- Documentation must include usage examples.
- Dependencies must be minimal and justified.
- Thread safety and error handling must be documented."#
        }
        ProjectType::SystemLevel => {
            r#"SYSTEM-LEVEL REQUIREMENTS
- Memory safety must be verified (no leaks, no undefined behavior).
- Resource cleanup must be guaranteed (RAII or equivalent).
- Performance constraints must be documented and tested.
- Hardware compatibility requirements must be specified.
- Error handling must not crash the system."#
        }
        ProjectType::Algorithm => {
            r#"ALGORITHM-SPECIFIC REQUIREMENTS
- Time complexity must be documented and verified.
- Space complexity must be documented and verified.
- Numerical stability must be considered (if applicable).
- Edge cases must be identified and tested (empty input, max size, etc.).
- Benchmark suite must cover representative workloads."#
        }
        ProjectType::Infra => {
            r#"INFRASTRUCTURE-SPECIFIC REQUIREMENTS
- Configuration must be idempotent (re-running produces same result).
- Rollback procedure must be documented and tested.
- State drift detection and remediation must be considered.
- Secrets must not be committed (use secret managers).
- Dry-run validation before actual apply."#
        }
    }
}

/// 取得驗證要求區塊
pub fn get_verification_requirements(
    has_verification_env: bool,
    project_type: ProjectType,
) -> String {
    let mut requirements = String::new();

    if has_verification_env {
        requirements.push_str(
            r#"  - INT E2E with cleared data + evidence capture and archiving:
    - features/<feature_key>/E2E_RUN_REPORT.md
    - features/<feature_key>/STATE.md
  - INT environment is the source of truth."#,
        );
    } else {
        requirements.push_str(
            r#"  - Local validation with evidence capture:
    - features/<feature_key>/TEST_REPORT.md
    - features/<feature_key>/STATE.md"#,
        );
    }

    // 專案類型特定驗證
    match project_type {
        ProjectType::Frontend | ProjectType::Fullstack => {
            if has_verification_env {
                requirements.push_str("\n  - For frontend features: Chrome DevTools console 0 error AND network 0 failure.");
            }
        }
        ProjectType::Algorithm => {
            requirements.push_str("\n  - Benchmark results must be captured in features/<feature_key>/BENCHMARK_REPORT.md");
        }
        ProjectType::SystemLevel => {
            requirements.push_str(
                "\n  - Memory/resource safety report in features/<feature_key>/SAFETY_REPORT.md",
            );
        }
        _ => {}
    }

    requirements
}

/// 取得 YAML 模板
pub fn get_yaml_template(
    project_type: ProjectType,
    has_verification_env: bool,
    needs_deployment: bool,
) -> String {
    let mut template = String::from("```\nfeatures:\n  - feature_key: <kebab-case-unique-key>\n    feature_name: \"<Prefix:> <Concise but specific title>\"\n    project_type: ");
    template.push_str(&project_type.to_string());
    template.push_str("\n    options:\n      needs_local_validation: true\n");

    if has_verification_env {
        template.push_str("      has_verification_env: true\n");
    }

    if needs_deployment {
        template.push_str("      needs_deployment: true\n");
    }

    // 專案類型特定選項
    match project_type {
        ProjectType::Cli => {
            template.push_str("      # test_command: \"cargo test\" # optional\n");
        }
        ProjectType::Library => {
            template.push_str("      # build_command: \"cargo build --release\" # optional\n");
        }
        ProjectType::Algorithm => {
            template.push_str(
                "      # custom_validation: \"Run benchmarks with criterion\" # optional\n",
            );
        }
        _ => {}
    }

    template.push_str("    context:\n      - \"<Goal statement>\"\n      - \"<Why now / what problem it solves>\"\n      - \"<Scope boundaries + constraints>\"\n");

    if has_verification_env {
        template.push_str("    verification_url: \"<Verification URL starting with http(s):// or empty string>\"\n");
        template
            .push_str("    # int_credentials: \"<Optional: credential mechanism description>\"\n");
    }

    // 需求區塊 - 根據專案類型調整
    template.push_str("    requirements:\n");
    template.push_str(&get_requirements_template(project_type));

    // 驗收條件 - 根據專案類型調整
    template.push_str("    acceptance_criteria:\n");
    template.push_str(&get_acceptance_template(project_type, has_verification_env));

    template.push_str("```");
    template
}

fn get_requirements_template(project_type: ProjectType) -> String {
    match project_type {
        ProjectType::Frontend | ProjectType::Fullstack => r#"      # --------
      # A. UI Components and Layout
      # --------
      - "<Define UI components and their states>"
      - "<Define responsive behavior and breakpoints>"
      # --------
      # B. User Interactions
      # --------
      - "<Define interaction states (hover/focus/active)>"
      - "<Define loading/success/error states>"
      # --------
      # C. Data Flow
      # --------
      - "<Define data sources and update triggers>"
      - "<Define validation rules>"
      # --------
      # D. Observability
      # --------
      - "<Structured logs fields>"
      - "<Error tracking>"
      # --------
      # E. Tests
      # --------
      - "<Unit tests list>"
      - "<E2E test steps>"
"#
        .to_string(),
        ProjectType::Backend => r#"      # --------
      # A. API Contracts
      # --------
      - "<Define endpoints and methods>"
      - "<Define request/response schemas>"
      # --------
      # B. Business Logic
      # --------
      - "<Define state machine if applicable>"
      - "<Define idempotency rules>"
      # --------
      # C. Data Layer
      # --------
      - "<Define DB schema or persistence>"
      - "<Define migrations and rollback>"
      # --------
      # D. Security
      # --------
      - "<AuthN/AuthZ requirements>"
      - "<Rate limiting>"
      # --------
      # E. Observability
      # --------
      - "<Structured logs fields>"
      - "<Metrics list>"
      # --------
      # F. Tests
      # --------
      - "<Unit tests list>"
      - "<Integration tests list>"
"#
        .to_string(),
        ProjectType::Cli => r#"      # --------
      # A. Command Interface
      # --------
      - "<Define commands and subcommands>"
      - "<Define arguments and flags>"
      # --------
      # B. Input/Output
      # --------
      - "<Define stdin/stdout/stderr behavior>"
      - "<Define exit codes>"
      # --------
      # C. Error Handling
      # --------
      - "<Define error messages and codes>"
      - "<Define help text format>"
      # --------
      # D. Tests
      # --------
      - "<Unit tests list>"
      - "<Integration tests (command execution)>"
"#
        .to_string(),
        ProjectType::Library => r#"      # --------
      # A. Public API
      # --------
      - "<Define public types and functions>"
      - "<Define error types>"
      # --------
      # B. Behavior
      # --------
      - "<Define thread safety guarantees>"
      - "<Define performance characteristics>"
      # --------
      # C. Documentation
      # --------
      - "<Define rustdoc/jsdoc requirements>"
      - "<Define usage examples>"
      # --------
      # D. Tests
      # --------
      - "<Unit tests list>"
      - "<Integration tests list>"
      - "<Doctest requirements>"
"#
        .to_string(),
        ProjectType::SystemLevel => r#"      # --------
      # A. Interface
      # --------
      - "<Define system interface (syscalls, IOCTLs, etc.)>"
      - "<Define data structures>"
      # --------
      # B. Safety
      # --------
      - "<Define memory safety guarantees>"
      - "<Define resource cleanup>"
      # --------
      # C. Performance
      # --------
      - "<Define latency/throughput constraints>"
      - "<Define resource limits>"
      # --------
      # D. Tests
      # --------
      - "<Unit tests list>"
      - "<System tests (VM/container/bare metal)>"
"#
        .to_string(),
        ProjectType::Algorithm => r#"      # --------
      # A. Algorithm Specification
      # --------
      - "<Define input/output format>"
      - "<Define invariants>"
      # --------
      # B. Complexity
      # --------
      - "<Define time complexity>"
      - "<Define space complexity>"
      # --------
      # C. Edge Cases
      # --------
      - "<Define edge cases to handle>"
      - "<Define numerical stability (if applicable)>"
      # --------
      # D. Benchmarks
      # --------
      - "<Define benchmark methodology>"
      - "<Define expected performance thresholds>"
      # --------
      # E. Tests
      # --------
      - "<Correctness tests>"
      - "<Performance tests>"
"#
        .to_string(),
        ProjectType::Infra => r#"      # --------
      # A. Resources
      # --------
      - "<Define resources to create/modify>"
      - "<Define dependencies between resources>"
      # --------
      # B. Configuration
      # --------
      - "<Define configuration variables>"
      - "<Define secrets handling>"
      # --------
      # C. Validation
      # --------
      - "<Define dry-run validation>"
      - "<Define post-apply validation>"
      # --------
      # D. Rollback
      # --------
      - "<Define rollback procedure>"
      - "<Define state backup>"
      # --------
      # E. Tests
      # --------
      - "<Validation tests>"
      - "<Drift detection tests>"
"#
        .to_string(),
    }
}

fn get_acceptance_template(project_type: ProjectType, has_verification_env: bool) -> String {
    let mut template = String::from(
        "      - \"<Binary verifiable outcome 1>\"\n      - \"<Binary verifiable outcome 2>\"\n",
    );

    if has_verification_env {
        template.push_str("      - \"INT clear-data E2E pass + evidence archived to features/<feature_key>/E2E_RUN_REPORT.md\"\n");
    } else {
        template.push_str("      - \"Local tests pass + evidence archived to features/<feature_key>/TEST_REPORT.md\"\n");
    }

    // 專案類型特定驗收條件
    match project_type {
        ProjectType::Frontend | ProjectType::Fullstack => {
            template
                .push_str("      - \"Chrome DevTools: console 0 error AND network 0 failure\"\n");
        }
        ProjectType::Cli => {
            template.push_str("      - \"All exit codes documented and tested\"\n");
            template.push_str("      - \"Help text is complete and accurate\"\n");
        }
        ProjectType::Library => {
            template.push_str("      - \"Public API is documented with examples\"\n");
            template.push_str("      - \"Semantic version is correctly set\"\n");
        }
        ProjectType::SystemLevel => {
            template.push_str("      - \"No memory leaks (verified by valgrind/ASAN)\"\n");
            template.push_str("      - \"Performance within specified constraints\"\n");
        }
        ProjectType::Algorithm => {
            template.push_str("      - \"Correctness verified on all edge cases\"\n");
            template.push_str("      - \"Benchmark results meet performance thresholds\"\n");
        }
        ProjectType::Infra => {
            template.push_str("      - \"Dry-run validation passes\"\n");
            template.push_str("      - \"Rollback procedure tested\"\n");
        }
        ProjectType::Backend => {
            template.push_str("      - \"All API endpoints return correct status codes\"\n");
        }
    }

    template
}

/// 生成完整的 YAML prompt
pub fn generate_yaml_prompt(
    project_type: ProjectType,
    has_verification_env: bool,
    needs_deployment: bool,
    custom_validation: Option<&str>,
) -> String {
    let required_fields = get_required_fields(project_type, has_verification_env, needs_deployment);
    let project_type_desc = get_project_type_description(project_type);
    let project_specific_reqs = get_project_specific_requirements(project_type);
    let verification_reqs = get_verification_requirements(has_verification_env, project_type);
    let yaml_template = get_yaml_template(project_type, has_verification_env, needs_deployment);

    let optional_fields = if has_verification_env {
        r#"verification_url (Mandatory fallback rule)
- verification_url MUST always be present and MUST be a quoted string.
- If a real, reachable verification URL is available, use it (must start with http:// or https://).
- If a real verification URL is NOT available, set exactly:
  - verification_url: ""
- Do NOT omit verification_url, do NOT use null.

int_credentials (Optional)
- Credentials or login method for the INT verification environment.
- If credentials are needed, provide them as a string or list of strings.
- If not needed or provided via environment variables/SSO/existing mechanisms, omit this field or set to empty string.
- NEVER include actual secrets - only describe the mechanism (e.g., "Use SSO login", "Set API_KEY env var")."#
    } else {
        "# No verification environment fields needed for this project type."
    };

    let requirements_content = r#"- Requirements MUST specify (as applicable):
  - contracts/schemas (fields + types + validation rules)
  - state machine, allowed transitions, illegal transition behavior
  - idempotency, dedupe rules, and replay safety
  - concurrency and locking rules
  - APIs: endpoints, auth, pagination/filtering, error model
  - data models: tables/indexes/retention, migrations + rollback
  - artifact storage: types, checksum, retention policy
  - observability: structured logs, metrics, tracing propagation
  - test plan: unit/integration + E2E steps
  - evidence capture requirements and file paths"#;

    let acceptance_content = if has_verification_env {
        r#"  - INT clear-data E2E pass conditions
  - Evidence archived to:
    - features/<feature_key>/E2E_RUN_REPORT.md
    - features/<feature_key>/STATE.md"#
    } else {
        r#"  - Local test pass conditions
  - Evidence archived to:
    - features/<feature_key>/TEST_REPORT.md
    - features/<feature_key>/STATE.md"#
    };

    let quality_checklist = r#"- Feature completeness:
  - context includes goal + constraints
  - requirements include deep operational detail with sections
  - acceptance_criteria are binary-verifiable
  - terminology is consistent: run, step, artifact, report, schedule
- Engineering rigor:
  - explicit contracts/schemas
  - state machine with illegal transition guard (if applicable)
  - idempotency and dedupe rules (if applicable)
  - observability + audit logs
  - security: authn/authz + RBAC (if applicable)
  - retention policy for artifacts/results (if applicable)
- Verification:
  - unit tests + integration tests specified
  - evidence capture requirements included"#;

    let mut prompt = YAML_GEN_BASE
        .replace("{REQUIRED_FIELDS}", &required_fields)
        .replace("{PROJECT_TYPE}", &project_type.to_string())
        .replace("{PROJECT_TYPE_DESCRIPTION}", project_type_desc)
        .replace("{PROJECT_SPECIFIC_REQUIREMENTS}", project_specific_reqs)
        .replace("{VERIFICATION_REQUIREMENTS}", &verification_reqs)
        .replace("{OPTIONAL_FIELD_SEMANTICS}", optional_fields)
        .replace("{REQUIREMENTS_CONTENT}", requirements_content)
        .replace("{ACCEPTANCE_CONTENT}", acceptance_content)
        .replace("{QUALITY_CHECKLIST}", quality_checklist)
        .replace("{YAML_TEMPLATE}", &yaml_template);

    // 添加自定義驗證說明（如果有的話）
    if let Some(custom) = custom_validation {
        prompt.push_str("\n\nCUSTOM VALIDATION REQUIREMENTS:\n");
        prompt.push_str(custom);
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_yaml_prompt_frontend() {
        let prompt = generate_yaml_prompt(ProjectType::Frontend, true, true, None);
        assert!(prompt.contains("FRONTEND"));
        assert!(prompt.contains("Chrome DevTools"));
        assert!(prompt.contains("verification_url"));
    }

    #[test]
    fn test_generate_yaml_prompt_cli_no_verification() {
        let prompt = generate_yaml_prompt(ProjectType::Cli, false, false, None);
        assert!(prompt.contains("CLI"));
        assert!(prompt.contains("exit codes"));
        assert!(!prompt.contains("INT environment"));
    }

    #[test]
    fn test_generate_yaml_prompt_with_custom_validation() {
        let prompt = generate_yaml_prompt(
            ProjectType::Algorithm,
            false,
            false,
            Some("Run custom benchmark with specific dataset"),
        );
        assert!(prompt.contains("ALGORITHM"));
        assert!(prompt.contains("Run custom benchmark with specific dataset"));
    }

    #[test]
    fn test_generate_yaml_prompt_backend() {
        let prompt = generate_yaml_prompt(ProjectType::Backend, true, true, None);
        assert!(prompt.contains("BACKEND"));
        assert!(prompt.contains("API contracts"));
        assert!(prompt.contains("verification_url"));
    }

    #[test]
    fn test_generate_yaml_prompt_fullstack() {
        let prompt = generate_yaml_prompt(ProjectType::Fullstack, true, true, None);
        assert!(prompt.contains("FULLSTACK"));
        assert!(prompt.contains("Chrome DevTools"));
        assert!(prompt.contains("API contracts"));
    }

    #[test]
    fn test_generate_yaml_prompt_library() {
        let prompt = generate_yaml_prompt(ProjectType::Library, false, false, None);
        assert!(prompt.contains("LIBRARY"));
        assert!(prompt.contains("semantic versioning"));
        assert!(prompt.contains("build_command"));
    }

    #[test]
    fn test_generate_yaml_prompt_system_level() {
        let prompt = generate_yaml_prompt(ProjectType::SystemLevel, false, false, None);
        assert!(prompt.contains("SYSTEM-LEVEL"));
        assert!(prompt.contains("Memory safety"));
        assert!(prompt.contains("SAFETY_REPORT"));
    }

    #[test]
    fn test_generate_yaml_prompt_infra() {
        let prompt = generate_yaml_prompt(ProjectType::Infra, true, true, None);
        assert!(prompt.contains("INFRASTRUCTURE"));
        assert!(prompt.contains("idempotent"));
        assert!(prompt.contains("Dry-run"));
    }

    #[test]
    fn test_get_project_type_description_all() {
        for project_type in ProjectType::ALL {
            let desc = get_project_type_description(project_type);
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_get_project_specific_requirements_all() {
        for project_type in ProjectType::ALL {
            let reqs = get_project_specific_requirements(project_type);
            assert!(!reqs.is_empty());
            assert!(reqs.contains("REQUIREMENTS"));
        }
    }

    #[test]
    fn test_get_required_fields_with_verification() {
        let fields = get_required_fields(ProjectType::Frontend, true, true);
        assert!(fields.contains("verification_url"));
        assert!(fields.contains("needs_deployment"));
    }

    #[test]
    fn test_get_required_fields_without_verification() {
        let fields = get_required_fields(ProjectType::Cli, false, false);
        assert!(!fields.contains("verification_url"));
        assert!(!fields.contains("needs_deployment: true"));
    }

    #[test]
    fn test_get_verification_requirements_with_env() {
        let reqs = get_verification_requirements(true, ProjectType::Frontend);
        assert!(reqs.contains("INT E2E"));
        assert!(reqs.contains("Chrome DevTools"));
    }

    #[test]
    fn test_get_verification_requirements_without_env() {
        let reqs = get_verification_requirements(false, ProjectType::Library);
        assert!(reqs.contains("Local validation"));
        assert!(!reqs.contains("INT E2E"));
    }

    #[test]
    fn test_get_yaml_template_contains_project_type() {
        for project_type in ProjectType::ALL {
            let template = get_yaml_template(project_type, false, false);
            assert!(template.contains(&project_type.to_string()));
        }
    }
}
