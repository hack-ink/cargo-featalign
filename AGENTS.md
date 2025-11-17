# Agent Workflow

## Overview

This document defines the tooling rules, execution model, coding conventions, and hard restrictions that apply to all automated agents interacting with this Rust repository. Its purpose is to ensure reproducibility, consistent formatting, predictable behavior, and safe automation across the workspace.

Agents must strictly follow every rule in this document when generating, modifying, or verifying source code or configuration.

---

# **1. Execution Principles**

## **1.1 Guiding Principle**

Agents must always use the curated `cargo make` tasks defined in `Makefile.toml` for all build, lint, format, test, and auxiliary operations.

Agents must **never** run raw `cargo` commands or manually invoke toolchains.

This guarantees:

- Correct toolchain selection
- Consistent feature flags
- Stable formatting and linting
- No divergence from expected project behavior

---

## **1.2 Toolchain Notes**

- The repository pins the stable Rust toolchain via `rust-toolchain.toml`.
- Certain tasks (e.g., formatting, some lint steps) require nightly, and the corresponding `cargo make` tasks already inject the correct version.
- Agents must not alter toolchain definitions or configuration files.

---

## **1.3 Common Tasks**

- `cargo make clippy` — Lints the entire workspace under the expected feature matrix.
- `cargo make fmt` — Formats the repository using the pinned nightly toolchain.
- `cargo make nextest` — Runs the full test suite via nextest.

Agents must prefer these tasks over any raw `cargo` invocation.

---

## **1.4 Adding New Automation**

When introducing any new automation:

1. Define a dedicated task inside `Makefile.toml`.
2. Document that task in this file so that all agents can use it.
3. Always invoke the task via `cargo make <task>`.

---

# **2. Rust Coding Conventions**

These conventions apply to all crates and all directories (libraries, binaries, tests, examples, benches). They define the required structure and style of Rust source code.
**Items that appear here do not appear again in the “Never Do” list to avoid duplication.**

---

## **2.1 Module, Import, and Declaration Order**

Each file must follow the strict declaration order:

```
mod
use
macro_rules
type
const
trait
enum
struct
fn
```

Within each group:

- `pub` items must appear before non-`pub` items.

Import section headers (e.g., `// std`, `// crates.io`, `// self`) must be preserved exactly and must not be removed.

---

## **2.2 Structs and Impl Blocks**

A struct’s `impl` must appear **immediately** after the struct definition with **no blank line** between them.

---

## **2.3 Generics, Bounds, and UFCS**

### Bound Placement

- All trait bounds must appear in a `where` clause.
- Inline trait bounds after the generic list are not allowed.
- Bounds must be ordered as:
  **(1) lifetimes → (2) standard library traits → (3) project traits**

### Generics

Whenever explicit generic type specification is required, agents must use turbofish syntax.

If turbofish cannot be used (e.g., `Into::into`), the explicit constructor form must be used instead.

### UFCS

Prefer UFCS (type-qualified paths) when referencing inherent items or specific trait implementations.

---

## **2.4 Borrowing Rules**

Prefer using plain references such as `&value` rather than `.as_ref()`, `.as_str()`, or similar adapters, unless the adapter is strictly required.

---

## **2.5 Macro, Formatting, and Logging Rules**

- Tracing macros must always be invoked with fully qualified paths (`tracing::info!`, etc.).
- When using `format!`, tracing, or logging macros, directly reference existing variables in the format string.
- Never introduce temporary variables solely for use inside a formatting expression.

---

# **3. Language Requirements**

These rules apply to:

- Comments
- Documentation comments (`///` and `//!`)
- Log messages

All such text must:

- Use standard, grammatically correct English.
- Begin with capital letters and end with proper punctuation.
- Avoid slang, localisms, and ambiguous abbreviations.
- Avoid non-English languages completely in code-facing text.

---

# **4. Never Do**

The following actions are **strictly prohibited**. These rules ensure workspace integrity, prevent unpredictable behavior, and restrict agents to safe operational boundaries.

## **4.1 Toolchain and System Prohibitions**

### **(A) Never modify toolchain management files.**

Agents must not edit:

- `rust-toolchain.toml`
- `.cargo/config.toml`
- `rustfmt.toml`

### **(B) Never install, upgrade, or manually switch Rust toolchains.**

Prohibited commands include:

```
rustup update
rustup install ...
rustup override ...
```

### **(C) Never execute system-level installation commands.**

Prohibited:

```
apt-get install ...
brew install ...
pip install ...
```

---

## **4.2 File Boundary Restrictions**

### **(D) Never modify files outside the repository root.**

Agents must not touch:

- Parent directories
- Home directories
- Global system paths
- External submodule files

### **(E) Never modify generated files.**

Including but not limited to:

- `OUT_DIR` outputs
- `build.rs` generated files
- `target/` directory
- Vendored third-party code
- Automatically generated assets or schemas

---

## **4.3 Patch and Diff Restrictions**

### **(F) Never produce non-unified diffs.**

Patch output must use standard unified diff format (`diff --git ...`).

### **(G) Never modify unrelated code in the same patch.**

Patches must be minimal and scoped exclusively to the user’s explicit request.

---

## **4.4 Style Restrictions**

### **(H) Never import tracing macros.**

Example of prohibited form:

```
use tracing::{info, warn};
```

Agents must always use:

```
tracing::info!();
```

### **(I) Never use inline trait bounds.**

(Prohibited under coding conventions; enforced again here as a hard rule.)

### **(J) Never place blank lines between a struct and its impl.**

### **(K) Never use `.as_ref()`, `.as_str()`, or similar adapters when `&value` suffices.**

---

## **4.5 Runtime and Async Restrictions**

### **(L) Never use `unwrap()` or `expect()` in non-test code.**

Unless the failure case is provably impossible (e.g., parsing a literal).

### **(M) Never block inside async contexts.**

Prohibited operations include:

```
std::thread::sleep(...)
blocking synchronous I/O
```

---

## **4.6 Documentation and Language Restrictions**

### **(N) Never mix languages in comments, docs, or logs.**

### **(O) Never introduce ambiguous abbreviations or informal slang.**

Examples of prohibited shorthand:

- `w/`
- `u`
- `tho`
- nonstandard contractions

---

## **4.7 Behavioral Restrictions**

### **(P) Never infer missing requirements.**

Agents may not guess intent or expand tasks beyond what is explicitly stated.

### **(Q) Never apply optimizations or refactors unless explicitly asked.**

This includes:

- Code cleanup
- Performance changes
- Interface redesign
- Removal of unused code

---

# **5. Summary**

This document defines the complete execution workflow, toolchain usage rules, coding conventions, language standards, and prohibited actions for all automated agents operating within this Rust workspace. By adhering to these rules, agents ensure that generated code is consistent, predictable, maintainable, and safe to apply across the entire repository.
