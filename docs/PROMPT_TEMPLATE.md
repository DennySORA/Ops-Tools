# Feature Specification Prompt

To generate the YAML specification file required for the LLM Prompt Generator, use the following prompt. This ensures the output adheres to the strict schema required by the tool.

```text
You are a senior software architect and QA / automation platform designer.

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
   - feature_key
   - feature_name
   - is_frontend
   - verification_url
   - context
   - requirements
   - acceptance_criteria

LANGUAGE REQUIREMENT (MANDATORY)
- All YAML content MUST be written in English:
  - All string values MUST be English.
  - All YAML comments (lines starting with #) MUST be English.

STYLE / SPEC RIGOR REQUIREMENTS
- Use clear, testable language using must/shall/should with explicit constraints and thresholds.
- Avoid vague statements (e.g., “handle properly”, “improve performance” without measurable criteria).
- Emphasize:
  - contracts / schemas
  - state machines and illegal transition guards
  - idempotency and deduplication
  - auditability
  - observability (logs/metrics/tracing)
  - E2E with cleared data + evidence capture and archiving:
    - features/<feature_key>/E2E_RUN_REPORT.md
    - features/<feature_key>/STATE.md
- Verification environment is the source of truth.
- For frontend features:
  - UI MUST reflect backend/DB truth (no client-side inference as source of truth).
  - Acceptance criteria MUST include Chrome DevTools: console 0 error AND network 0 failure.

FIELD SEMANTICS (MUST APPLY)
feature_key
- Uniquely identifies the feature in tracking systems.
- Must be unique across the entire YAML.
- Format: lower-kebab-case.
- Recommended action prefixes: add-, enhance-, fix-, refactor-, healthcheck-, upgrade-.

feature_name
- Human-facing title.
- MUST start with one of the prefixes:
  "Add:", "Enhance:", "Fix:", "Refactor:", "Upgrade:", "Healthcheck:"
- Must be concise but specific and include key scope/constraints when relevant.

is_frontend
- true if any UI deliverable exists, otherwise false.

verification_url (Mandatory fallback rule)
- verification_url MUST always be present and MUST be a quoted string.
- If a real, reachable verification URL is available, use it.
- If a real verification URL is NOT available, set exactly:
  - verification_url: "No verification_url"
- Do NOT omit verification_url, do NOT use null, and do NOT use an empty string.

context
- A list of English strings describing why/what at a high level.
- Must include:
  - Goal statement and user impact
  - Why now / problem being solved
  - Scope boundaries and key constraints (Verification truth, clear-data E2E, non-goals if needed)

requirements
- A detailed, testable specification as a list of strings.
- Must be structured using YAML comment section headings EXACTLY like:
  # --------
  # A. <title>
  # --------
- Each requirement MUST be an English string item.
- Requirements MUST specify (as applicable):
  - contracts/schemas (fields + types + validation rules)
  - state machine, allowed transitions, illegal transition behavior
  - idempotency, dedupe rules, and replay safety
  - concurrency and locking rules
  - APIs: endpoints, auth, pagination/filtering, error model
  - data models: tables/indexes/retention, migrations + rollback
  - artifact storage: types, checksum, retention policy
  - observability: structured logs, metrics, tracing propagation
  - test plan: unit/integration + E2E steps starting with clearing data
  - evidence capture requirements and file paths

acceptance_criteria
- A list of binary-verifiable outcomes in English.
- Must include:
  - Clear-data E2E pass conditions in verification environment
  - Evidence archived to:
    - features/<feature_key>/E2E_RUN_REPORT.md
    - features/<feature_key>/STATE.md
  - For frontend features: Chrome DevTools console 0 error AND network 0 failure

QUALITY BAR SELF-CHECK (DO THIS BEFORE YOU OUTPUT)
- Feature completeness:
  - context includes goal + constraints
  - requirements include deep operational detail with sections
  - acceptance_criteria are binary-verifiable and mention clear-data E2E + evidence paths
  - terminology is consistent: run, step, artifact, report, schedule
- Engineering rigor:
  - explicit contracts/schemas
  - state machine with illegal transition guard
  - idempotency and dedupe rules
  - observability + audit logs
  - security: authn/authz + RBAC
  - retention policy for artifacts/results
- Verification:
  - unit tests + integration tests + E2E with clear-data prerequisite in verification environment
  - evidence capture requirements included

NOW GENERATE THE YAML IN THE EXACT SHAPE OF THE TEMPLATE BELOW.
- Keep the same key order as the template.
- Replace placeholders with concrete, English content.
- Output YAML only.

features:
  - feature_key: <kebab-case-unique-key>
    feature_name: "<Prefix:> <Concise but specific title>"
    is_frontend: <true|false>
    verification_url: "<Verification URL or 'No verification_url'>"
    context:
      - "<Goal statement>"
      - "<Why now / what problem it solves>"
      - "<Scope boundaries + constraints (Verification truth, clear data E2E, etc.)>"
    requirements:
      # --------
      # A. Scope and contracts
      # --------
      - "<Define primary entities/contracts and required fields/types>"
      - "<Define input/output schemas; strict validation rules>"
      # --------
      # B. Execution semantics
      # --------
      - "<Define state machine; allowed transitions; illegal transition handling>"
      - "<Define idempotency and dedupe rules>"
      - "<Define concurrency/locking rules>"
      # --------
      # C. APIs and security
      # --------
      - "<List endpoints and required query filters/pagination>"
      - "<AuthN/AuthZ requirements; RBAC matrix>"
      - "<Audit log fields and required coverage>"
      # --------
      # D. Storage, artifacts, retention
      # --------
      - "<DB schema or persistence requirements; migrations and rollback>"
      - "<Artifact types and storage; checksum; retention policy>"
      # --------
      # E. Observability
      # --------
      - "<Structured logs fields>"
      - "<Metrics list>"
      - "<Tracing propagation rules>"
      # --------
      # F. Tests and E2E (clear data)
      # --------
      - "<Unit tests list>"
      - "<Integration tests list>"
      - "<E2E steps starting with clearing data (DB/queue/cache/object storage as applicable) in verification environment>"
      - "<Evidence capture requirements (features/<feature_key>/E2E_RUN_REPORT.md and features/<feature_key>/STATE.md)>"
    acceptance_criteria:
      - "<Binary verifiable outcome 1>"
      - "<Binary verifiable outcome 2>"
      - "<Clear-data E2E pass conditions in verification environment + evidence archived to features/<feature_key>/E2E_RUN_REPORT.md and features/<feature_key>/STATE.md>"
      - "<(Frontend only) Chrome DevTools: console 0 error AND network 0 failure>"
```