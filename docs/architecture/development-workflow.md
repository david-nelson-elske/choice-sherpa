# Development Workflow: Linting, Formatting, and CI

**Type:** Development Standards
**Priority:** P0 (Required before implementation)
**Last Updated:** 2026-01-08

> Complete specification for code quality tooling, formatting standards, and continuous integration pipeline.

---

## Overview

This document defines the mandatory development workflow for Choice Sherpa:

| Aspect | Backend (Rust) | Frontend (SvelteKit/TS) |
|--------|----------------|-------------------------|
| **Linter** | Clippy | ESLint |
| **Formatter** | rustfmt | Prettier |
| **Type Checker** | rustc (built-in) | TypeScript |
| **Test Runner** | cargo test | Vitest |
| **CI Platform** | GitHub Actions | GitHub Actions |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Developer Workflow                                  │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      Local Development                               │   │
│   │                                                                      │   │
│   │   1. Write Code                                                      │   │
│   │         │                                                            │   │
│   │         ▼                                                            │   │
│   │   2. Pre-commit Hooks (automatic)                                    │   │
│   │      ├── Format check (rustfmt, prettier)                           │   │
│   │      ├── Lint check (clippy, eslint)                                │   │
│   │      └── Type check (cargo check, tsc)                              │   │
│   │         │                                                            │   │
│   │         ▼                                                            │   │
│   │   3. git commit (blocked if checks fail)                            │   │
│   │         │                                                            │   │
│   │         ▼                                                            │   │
│   │   4. git push                                                        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
└────────────────────────────────────┼─────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           GitHub Actions CI                                   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      Pull Request Checks                             │   │
│   │                                                                      │   │
│   │   Parallel Jobs:                                                     │   │
│   │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │   │
│   │   │   Backend   │  │  Frontend   │  │  Security   │                │   │
│   │   │   Pipeline  │  │  Pipeline   │  │    Scan     │                │   │
│   │   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │   │
│   │          │                │                │                        │   │
│   │          ▼                ▼                ▼                        │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │                    All Checks Pass?                          │   │   │
│   │   │                           │                                  │   │   │
│   │   │         ┌─────────────────┴─────────────────┐               │   │   │
│   │   │         │                                   │               │   │   │
│   │   │         ▼                                   ▼               │   │   │
│   │   │   ✅ Merge Allowed              ❌ Merge Blocked            │   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Backend Linting (Rust/Clippy)

### Clippy Configuration

```toml
# backend/.clippy.toml

# Maximum cognitive complexity before warning
cognitive-complexity-threshold = 25

# Maximum number of lines in a function
too-many-lines-threshold = 100

# Maximum number of arguments in a function
too-many-arguments-threshold = 7

# Disallowed methods (use alternatives)
disallowed-methods = [
    { path = "std::env::var", reason = "Use config crate instead" },
    { path = "println", reason = "Use tracing macros for logging" },
    { path = "eprintln", reason = "Use tracing macros for logging" },
]

# Disallowed types
disallowed-types = [
    { path = "std::collections::HashMap", reason = "Use hashbrown::HashMap for consistency" },
]
```

### Cargo.toml Lint Configuration

```toml
# backend/Cargo.toml

[lints.rust]
# Deny unsafe code by default
unsafe_code = "deny"

# Warn on missing docs for public items
missing_docs = "warn"

# Deny unreachable patterns
unreachable_patterns = "deny"

[lints.clippy]
# == Error-level (must fix) ==
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
unreachable = "deny"

# == Warning-level (should fix) ==
# Correctness
clone_on_ref_ptr = "warn"
float_cmp = "warn"
lossy_float_literal = "warn"

# Performance
inefficient_to_string = "warn"
large_enum_variant = "warn"
large_stack_arrays = "warn"
needless_collect = "warn"
unnecessary_to_owned = "warn"

# Style
bool_to_int_with_if = "warn"
cloned_instead_of_copied = "warn"
default_trait_access = "warn"
explicit_iter_loop = "warn"
flat_map_option = "warn"
from_iter_instead_of_collect = "warn"
if_not_else = "warn"
implicit_clone = "warn"
inconsistent_struct_constructor = "warn"
index_refutable_slice = "warn"
macro_use_imports = "warn"
manual_let_else = "warn"
manual_ok_or = "warn"
manual_string_new = "warn"
match_bool = "warn"
match_same_arms = "warn"
missing_const_for_fn = "warn"
needless_for_each = "warn"
needless_pass_by_value = "warn"
option_option = "warn"
range_plus_one = "warn"
redundant_closure_for_method_calls = "warn"
redundant_else = "warn"
ref_binding_to_reference = "warn"
semicolon_if_nothing_returned = "warn"
single_match_else = "warn"
string_add_assign = "warn"
trait_duplication_in_bounds = "warn"
trivially_copy_pass_by_ref = "warn"
uninlined_format_args = "warn"
unnested_or_patterns = "warn"
unused_async = "warn"
unused_self = "warn"
used_underscore_binding = "warn"

# Complexity
too_many_lines = "warn"
cognitive_complexity = "warn"
too_many_arguments = "warn"

# Pedantic (optional but recommended)
doc_markdown = "warn"
manual_assert = "warn"
similar_names = "warn"

# == Allow (explicitly permitted) ==
module_inception = "allow"
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

### Running Clippy

```bash
# Standard check (CI enforced)
cargo clippy --all-targets --all-features -- -D warnings

# With fix suggestions
cargo clippy --fix --allow-dirty --allow-staged

# Check specific package
cargo clippy -p choice-sherpa-domain -- -D warnings
```

---

## 2. Backend Formatting (rustfmt)

### rustfmt Configuration

```toml
# backend/rustfmt.toml

# Edition
edition = "2021"

# Line width
max_width = 100

# Tabs vs spaces
hard_tabs = false
tab_spaces = 4

# Imports
imports_granularity = "Module"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true

# Comments
comment_width = 100
wrap_comments = true
normalize_comments = true
normalize_doc_attributes = true

# Functions
fn_params_layout = "Tall"
fn_single_line = false

# Match arms
match_arm_blocks = true
match_arm_leading_pipes = "Never"
match_block_trailing_comma = true

# Structs
struct_lit_single_line = true
struct_field_align_threshold = 0

# Control flow
control_brace_style = "AlwaysSameLine"
force_explicit_abi = true

# Misc
use_field_init_shorthand = true
use_try_shorthand = true
format_code_in_doc_comments = true
format_macro_matchers = true
format_macro_bodies = true
hex_literal_case = "Lower"

# Strings
format_strings = false

# Chains
chain_width = 60
```

### Running rustfmt

```bash
# Check formatting (CI enforced)
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all

# Check specific file
rustfmt --check src/main.rs
```

---

## 3. Frontend Linting (ESLint)

### ESLint Configuration

```javascript
// frontend/eslint.config.js

import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

/** @type {import('eslint').Linter.Config[]} */
export default [
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  prettier,
  ...svelte.configs['flat/prettier'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },
  {
    rules: {
      // == Error-level (must fix) ==
      'no-console': ['error', { allow: ['warn', 'error'] }],
      'no-debugger': 'error',
      'no-alert': 'error',
      'no-var': 'error',
      'prefer-const': 'error',
      'no-unused-vars': 'off', // Use TypeScript version
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/explicit-function-return-type': [
        'error',
        { allowExpressions: true, allowTypedFunctionExpressions: true },
      ],
      '@typescript-eslint/no-non-null-assertion': 'error',

      // == Warning-level (should fix) ==
      '@typescript-eslint/prefer-nullish-coalescing': 'warn',
      '@typescript-eslint/prefer-optional-chain': 'warn',
      '@typescript-eslint/no-floating-promises': 'warn',
      'no-nested-ternary': 'warn',
      complexity: ['warn', { max: 15 }],
      'max-depth': ['warn', { max: 4 }],
      'max-lines-per-function': ['warn', { max: 100, skipBlankLines: true, skipComments: true }],

      // == Svelte-specific ==
      'svelte/no-at-html-tags': 'error',
      'svelte/require-stores-init': 'error',
      'svelte/valid-compile': 'error',
      'svelte/no-unused-svelte-ignore': 'warn',
    },
  },
  {
    ignores: [
      '.svelte-kit/**',
      'build/**',
      'node_modules/**',
      '*.config.js',
      '*.config.ts',
    ],
  },
];
```

### Running ESLint

```bash
# Standard check (CI enforced)
npm run lint

# With auto-fix
npm run lint -- --fix

# Check specific file
npx eslint src/routes/+page.svelte
```

### Package.json Scripts

```json
{
  "scripts": {
    "lint": "eslint . --ext .js,.ts,.svelte",
    "lint:fix": "eslint . --ext .js,.ts,.svelte --fix"
  }
}
```

---

## 4. Frontend Formatting (Prettier)

### Prettier Configuration

```json
// frontend/.prettierrc
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "es5",
  "printWidth": 100,
  "tabWidth": 2,
  "useTabs": false,
  "bracketSpacing": true,
  "bracketSameLine": false,
  "arrowParens": "always",
  "endOfLine": "lf",
  "quoteProps": "as-needed",
  "jsxSingleQuote": false,
  "proseWrap": "preserve",
  "htmlWhitespaceSensitivity": "css",
  "embeddedLanguageFormatting": "auto",
  "singleAttributePerLine": false,
  "plugins": ["prettier-plugin-svelte", "prettier-plugin-tailwindcss"],
  "overrides": [
    {
      "files": "*.svelte",
      "options": {
        "parser": "svelte"
      }
    }
  ]
}
```

### Prettier Ignore

```gitignore
# frontend/.prettierignore

# Build outputs
.svelte-kit/
build/
dist/

# Dependencies
node_modules/

# Generated
*.generated.ts
src/lib/api/client.ts

# Config
*.config.js
*.config.ts
svelte.config.js
vite.config.ts
```

### Running Prettier

```bash
# Check formatting (CI enforced)
npm run format:check

# Apply formatting
npm run format

# Check specific file
npx prettier --check src/routes/+page.svelte
```

### Package.json Scripts

```json
{
  "scripts": {
    "format": "prettier --write .",
    "format:check": "prettier --check ."
  }
}
```

---

## 5. TypeScript Configuration

### tsconfig.json

```json
// frontend/tsconfig.json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitThis": true,
    "useUnknownInCatchVariables": true,
    "alwaysStrict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "noPropertyAccessFromIndexSignature": true,
    "allowUnusedLabels": false,
    "allowUnreachableCode": false,
    "forceConsistentCasingInFileNames": true,
    "moduleResolution": "bundler",
    "target": "ES2022",
    "module": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "paths": {
      "$lib": ["./src/lib"],
      "$lib/*": ["./src/lib/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"],
  "exclude": ["node_modules"]
}
```

### Running Type Check

```bash
# Full type check (CI enforced)
npm run check

# Watch mode
npm run check -- --watch
```

### Package.json Scripts

```json
{
  "scripts": {
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch"
  }
}
```

---

## 6. Pre-commit Hooks

### Husky Setup

```bash
# Install husky
npm install -D husky
npx husky init
```

### Pre-commit Hook

```bash
#!/bin/sh
# .husky/pre-commit

# Exit on error
set -e

echo "🔍 Running pre-commit checks..."

# === Backend Checks ===
if git diff --cached --name-only | grep -q "^backend/"; then
    echo "📦 Backend changes detected"

    # Format check
    echo "  → Checking Rust formatting..."
    (cd backend && cargo fmt --all -- --check)

    # Lint check
    echo "  → Running Clippy..."
    (cd backend && cargo clippy --all-targets --all-features -- -D warnings)

    # Type check (fast compile check)
    echo "  → Type checking..."
    (cd backend && cargo check --all-targets)
fi

# === Frontend Checks ===
if git diff --cached --name-only | grep -q "^frontend/"; then
    echo "🎨 Frontend changes detected"

    # Format check
    echo "  → Checking Prettier formatting..."
    (cd frontend && npm run format:check)

    # Lint check
    echo "  → Running ESLint..."
    (cd frontend && npm run lint)

    # Type check
    echo "  → Type checking..."
    (cd frontend && npm run check)
fi

echo "✅ All pre-commit checks passed!"
```

### Pre-push Hook (Optional - runs tests)

```bash
#!/bin/sh
# .husky/pre-push

set -e

echo "🧪 Running pre-push checks..."

# === Backend Tests ===
if git diff origin/main --name-only | grep -q "^backend/"; then
    echo "📦 Running backend tests..."
    (cd backend && cargo test --all-features)
fi

# === Frontend Tests ===
if git diff origin/main --name-only | grep -q "^frontend/"; then
    echo "🎨 Running frontend tests..."
    (cd frontend && npm run test:run)
fi

echo "✅ All pre-push checks passed!"
```

### lint-staged Configuration

```json
// package.json (root)
{
  "lint-staged": {
    "backend/**/*.rs": [
      "cd backend && cargo fmt -- --check",
      "cd backend && cargo clippy -- -D warnings"
    ],
    "frontend/**/*.{js,ts,svelte}": [
      "prettier --check",
      "eslint --max-warnings=0"
    ],
    "frontend/**/*.{css,scss}": [
      "prettier --check"
    ],
    "**/*.md": [
      "prettier --check"
    ]
  }
}
```

---

## 7. GitHub Actions CI

### Main CI Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ============================================
  # Backend Pipeline
  # ============================================
  backend-format:
    name: Backend Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        working-directory: backend
        run: cargo fmt --all -- --check

  backend-lint:
    name: Backend Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend

      - name: Run Clippy
        working-directory: backend
        run: cargo clippy --all-targets --all-features -- -D warnings

  backend-test:
    name: Backend Test
    runs-on: ubuntu-latest
    needs: [backend-format, backend-lint]
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: choice_sherpa_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend

      - name: Run tests
        working-directory: backend
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/choice_sherpa_test
          REDIS_URL: redis://localhost:6379
        run: cargo test --all-features

  backend-coverage:
    name: Backend Coverage
    runs-on: ubuntu-latest
    needs: backend-test
    if: github.event_name == 'pull_request'
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: choice_sherpa_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Run coverage
        working-directory: backend
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/choice_sherpa_test
        run: cargo tarpaulin --out xml --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: backend/coverage/cobertura.xml
          flags: backend
          fail_ci_if_error: false

  # ============================================
  # Frontend Pipeline
  # ============================================
  frontend-format:
    name: Frontend Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Check formatting
        working-directory: frontend
        run: npm run format:check

  frontend-lint:
    name: Frontend Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Run ESLint
        working-directory: frontend
        run: npm run lint

  frontend-typecheck:
    name: Frontend Type Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Type check
        working-directory: frontend
        run: npm run check

  frontend-test:
    name: Frontend Test
    runs-on: ubuntu-latest
    needs: [frontend-format, frontend-lint, frontend-typecheck]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Run tests
        working-directory: frontend
        run: npm run test:run -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: frontend/coverage/coverage-final.json
          flags: frontend
          fail_ci_if_error: false

  frontend-build:
    name: Frontend Build
    runs-on: ubuntu-latest
    needs: frontend-test
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Build
        working-directory: frontend
        run: npm run build

  # ============================================
  # Security Scanning
  # ============================================
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Audit backend dependencies
        working-directory: backend
        run: cargo audit

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Audit frontend dependencies
        working-directory: frontend
        run: npm audit --audit-level=high

  # ============================================
  # Final Check Gate
  # ============================================
  ci-success:
    name: CI Success
    runs-on: ubuntu-latest
    needs:
      - backend-format
      - backend-lint
      - backend-test
      - frontend-format
      - frontend-lint
      - frontend-typecheck
      - frontend-test
      - frontend-build
      - security-audit
    if: always()
    steps:
      - name: Check all jobs
        run: |
          if [[ "${{ needs.backend-format.result }}" != "success" ]] ||
             [[ "${{ needs.backend-lint.result }}" != "success" ]] ||
             [[ "${{ needs.backend-test.result }}" != "success" ]] ||
             [[ "${{ needs.frontend-format.result }}" != "success" ]] ||
             [[ "${{ needs.frontend-lint.result }}" != "success" ]] ||
             [[ "${{ needs.frontend-typecheck.result }}" != "success" ]] ||
             [[ "${{ needs.frontend-test.result }}" != "success" ]] ||
             [[ "${{ needs.frontend-build.result }}" != "success" ]] ||
             [[ "${{ needs.security-audit.result }}" != "success" ]]; then
            echo "❌ CI failed"
            exit 1
          fi
          echo "✅ All CI checks passed"
```

### Branch Protection Rules

Configure in GitHub repository settings:

```yaml
# Recommended branch protection for `main`:
Branch protection rules:
  main:
    required_status_checks:
      strict: true
      contexts:
        - "CI Success"
    required_pull_request_reviews:
      dismiss_stale_reviews: true
      require_code_owner_reviews: false
      required_approving_review_count: 1
    enforce_admins: false
    restrictions: null
    allow_force_pushes: false
    allow_deletions: false
    require_conversation_resolution: true
```

---

## 8. Editor Configuration

### EditorConfig

```ini
# .editorconfig

root = true

[*]
charset = utf-8
end_of_line = lf
indent_style = space
insert_final_newline = true
trim_trailing_whitespace = true

[*.rs]
indent_size = 4

[*.{js,ts,svelte,json,yaml,yml,md}]
indent_size = 2

[*.md]
trim_trailing_whitespace = false

[Makefile]
indent_style = tab
```

### VS Code Settings

```json
// .vscode/settings.json
{
  // Rust
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-features"],
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },

  // TypeScript/JavaScript
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },

  // Svelte
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode",
    "editor.formatOnSave": true
  },
  "svelte.enable-ts-plugin": true,

  // JSON
  "[json]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },

  // Markdown
  "[markdown]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },

  // General
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": "explicit",
    "source.organizeImports": "explicit"
  },
  "files.eol": "\n",
  "files.insertFinalNewline": true,
  "files.trimTrailingWhitespace": true
}
```

### VS Code Extensions

```json
// .vscode/extensions.json
{
  "recommendations": [
    // Rust
    "rust-lang.rust-analyzer",

    // Frontend
    "svelte.svelte-vscode",
    "esbenp.prettier-vscode",
    "dbaeumer.vscode-eslint",

    // General
    "editorconfig.editorconfig",
    "streetsidesoftware.code-spell-checker",
    "eamodio.gitlens",

    // Testing
    "vitest.explorer",

    // Docker
    "ms-azuretools.vscode-docker"
  ]
}
```

---

## 9. CI Pipeline Visualization

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              Pull Request CI Pipeline                                 │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────┐   │
│   │                          Parallel Execution                                  │   │
│   │                                                                              │   │
│   │   Backend Track              Frontend Track            Security Track        │   │
│   │   ─────────────              ──────────────            ──────────────        │   │
│   │                                                                              │   │
│   │   ┌──────────┐              ┌──────────┐              ┌──────────┐          │   │
│   │   │  Format  │              │  Format  │              │  Audit   │          │   │
│   │   │ (rustfmt)│              │(prettier)│              │(cargo +  │          │   │
│   │   └────┬─────┘              └────┬─────┘              │npm audit)│          │   │
│   │        │                         │                    └──────────┘          │   │
│   │   ┌────▼─────┐              ┌────▼─────┐                                    │   │
│   │   │   Lint   │              │   Lint   │                                    │   │
│   │   │ (clippy) │              │ (eslint) │                                    │   │
│   │   └────┬─────┘              └────┬─────┘                                    │   │
│   │        │                         │                                          │   │
│   │        │                    ┌────▼─────┐                                    │   │
│   │        │                    │TypeCheck │                                    │   │
│   │        │                    │  (tsc)   │                                    │   │
│   │        │                    └────┬─────┘                                    │   │
│   │        │                         │                                          │   │
│   │   ┌────▼─────┐              ┌────▼─────┐                                    │   │
│   │   │   Test   │              │   Test   │                                    │   │
│   │   │ (cargo)  │              │ (vitest) │                                    │   │
│   │   └────┬─────┘              └────┬─────┘                                    │   │
│   │        │                         │                                          │   │
│   │   ┌────▼─────┐              ┌────▼─────┐                                    │   │
│   │   │ Coverage │              │   Build  │                                    │   │
│   │   │(tarpaulin│              │ (vite)   │                                    │   │
│   │   └────┬─────┘              └────┬─────┘                                    │   │
│   │        │                         │                                          │   │
│   └────────┼─────────────────────────┼──────────────────────────────────────────┘   │
│            │                         │                                              │
│            └────────────┬────────────┘                                              │
│                         │                                                           │
│                    ┌────▼─────┐                                                     │
│                    │CI Success│                                                     │
│                    │  Gate    │                                                     │
│                    └────┬─────┘                                                     │
│                         │                                                           │
│              ┌──────────┴──────────┐                                               │
│              │                     │                                               │
│              ▼                     ▼                                               │
│        ✅ Pass: Merge        ❌ Fail: Block                                        │
│           Allowed              Merge                                               │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘

Approximate Timings:
├── Format checks: ~30s each
├── Lint checks: ~1-2 min each
├── Type check: ~30s
├── Backend tests: ~3-5 min
├── Frontend tests: ~1-2 min
├── Build: ~1-2 min
├── Security audit: ~1 min
└── Total (parallel): ~5-7 min
```

---

## 10. Command Reference

### Quick Reference

```bash
# === Backend Commands ===
cd backend

# Format
cargo fmt --all                    # Apply formatting
cargo fmt --all -- --check         # Check only (CI)

# Lint
cargo clippy                       # Basic lint
cargo clippy -- -D warnings        # Strict (CI)
cargo clippy --fix                 # Auto-fix

# Test
cargo test                         # Run all tests
cargo test --all-features          # With all features
cargo tarpaulin --out Html         # Coverage report

# Build
cargo build                        # Debug build
cargo build --release              # Release build
cargo check                        # Type check only (fast)


# === Frontend Commands ===
cd frontend

# Format
npm run format                     # Apply formatting
npm run format:check               # Check only (CI)

# Lint
npm run lint                       # Run ESLint
npm run lint -- --fix              # Auto-fix

# Type Check
npm run check                      # Full type check
npm run check -- --watch           # Watch mode

# Test
npm run test                       # Interactive mode
npm run test:run                   # Single run (CI)
npm run test:run -- --coverage     # With coverage

# Build
npm run dev                        # Dev server
npm run build                      # Production build
npm run preview                    # Preview production build


# === Root Level ===
# Pre-commit hooks (installed automatically)
npm run prepare                    # Setup husky

# Run all checks
make check                         # If using Makefile
```

### Makefile (Optional)

```makefile
# Makefile (root)

.PHONY: all check format lint test build clean

all: check

# Run all checks
check: format-check lint test

# Format
format:
	cd backend && cargo fmt --all
	cd frontend && npm run format

format-check:
	cd backend && cargo fmt --all -- --check
	cd frontend && npm run format:check

# Lint
lint:
	cd backend && cargo clippy --all-targets --all-features -- -D warnings
	cd frontend && npm run lint
	cd frontend && npm run check

# Test
test:
	cd backend && cargo test --all-features
	cd frontend && npm run test:run

# Build
build:
	cd backend && cargo build --release
	cd frontend && npm run build

# Clean
clean:
	cd backend && cargo clean
	cd frontend && rm -rf .svelte-kit build node_modules

# Development
dev-backend:
	cd backend && cargo watch -x run

dev-frontend:
	cd frontend && npm run dev

# Setup
setup:
	cd frontend && npm ci
	cd frontend && npm run prepare
	cd backend && cargo build
```

---

## Summary

| Check | Local | CI | Blocking |
|-------|-------|-----|----------|
| Backend format (rustfmt) | Pre-commit | ✅ | Yes |
| Backend lint (clippy) | Pre-commit | ✅ | Yes |
| Backend tests | Pre-push | ✅ | Yes |
| Backend coverage | - | ✅ | No (reporting only) |
| Frontend format (prettier) | Pre-commit | ✅ | Yes |
| Frontend lint (eslint) | Pre-commit | ✅ | Yes |
| Frontend type check (tsc) | Pre-commit | ✅ | Yes |
| Frontend tests (vitest) | Pre-push | ✅ | Yes |
| Frontend build | - | ✅ | Yes |
| Security audit | - | ✅ | Yes |

---

## Related Documents

- **Consistency Patterns**: `docs/architecture/consistency-patterns.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`
- **Contributing Guide**: `CONTRIBUTING.md` (to be created)

---

*Version: 1.0.0*
*Created: 2026-01-08*
