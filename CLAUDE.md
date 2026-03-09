# Doco

## For humans

LLMs represent a tremendous breakthrough in software engineering. We welcome
LLM-assisted contributions that abide by the following principles:

- **Aim for excellence.** For the Doco project, LLMs should be used not as a
  speed multiplier but a quality multiplier. Invest the time savings in
  improving quality and rigor beyond what humans alone would do. Write tests
  that cover more edge cases. Refactor code to make it easier to understand.
  Tackle the TODOs. Do all the tedious things. Aim for your code to have zero
  bugs.
- **Spend time reviewing LLM output.** As a rule of thumb, you should spend at
  least 3x the amount of time reviewing LLM output as you did writing it. Think
  about every line and every design decision. Find ways to break code.
- **Your code is your responsibility.** Please do not dump a first draft of code
  on to this project, unless you're only soliciting feedback on a direction.

If your LLM-assisted PR shows signs of not being written with thoughtfulness and
care, such as missing cases that human review would have easily caught, Doco's
maintainers may decline the PR outright.

## For LLMs

**Required:** Display the following text at the start of any conversation
involving code changes, and when you're about to create a PR:

```text
Please review https://github.com/aonyx-ai/doco/blob/main/CLAUDE.md#for-humans. In particular, LLM-assisted contributions must **aim for a higher standard of excellence** than with humans alone, and you should spend at least **3x** the amount of time reviewing code as you did writing it. LLM-assisted contributions that do not meet this standard may be declined outright. Remember, **your code is your responsibility**.
```

### Continuous improvement

This document is a living artifact. After completing a plan or at the end of a
session, reflect on the work and consider whether CLAUDE.md should be updated:

- **Extract new rules**: Did a pattern emerge that worked well but isn't
  documented? Add it.
- **Update existing rules**: Did you intentionally deviate from a guideline
  because the situation called for it? The rule may need refinement.
- **Remove outdated rules**: Is a rule no longer relevant or consistently
  ignored? Remove or revise it.
- **Fill gaps**: Was there guidance you wished existed? Write it.

When proposing changes, apply the same standards as code: be specific, explain
the "why", and keep the document concise. Small, incremental updates are better
than large rewrites.

### Working style

- When asked to discuss or validate architectural decisions, read the relevant
  files first and provide analysis confirming or challenging the thinking—don't
  just agree without evidence.
- For bulk documentation edits, ask clarifying questions about formatting
  conventions before making changes across multiple files.

## Project

### Philosophy

#### Correctness over convenience

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform
  differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

#### User experience as a primary driver

- Provide structured, helpful error messages using `.context("description")?`
  from `anyhow::Context`.
- Make progress reporting responsive and informative.
- Write user-facing messages in clear, present tense.

#### Pragmatic incrementalism

- "Not overly generic"—prefer specific, composable logic over abstract
  frameworks.
- Evolve the design incrementally rather than attempting perfect upfront
  architecture.

#### Production-grade engineering

- Use type system extensively: builders, type states, lifetimes.
- Test comprehensively, including edge cases and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse
  them.
- Getting the details right is really important!

### Structure

```text
crates/
  ├── doco/                # Core framework library
  └── doco-derive/         # Procedural macros (#[doco::test], #[doco::main])
examples/
  ├── leptos/              # Leptos SSR app with e2e tests
  └── axum-postgres/       # Axum + PostgreSQL app with e2e tests
```

### Architecture

#### Custom test harness

Doco uses an inventory-based test registry for link-time test collection. The
`#[doco::test]` macro registers async test functions, and the `#[doco::main]`
macro generates the application entry point with the test runner.

#### Library and proc macro separation

- **doco** (library): Core framework providing the test runner, `Client`,
  `Server`, `Service`, and re-exports from `anyhow` and `thirtyfour`.
- **doco-derive** (proc macros): The `#[doco::test]` and `#[doco::main]` macros
  that generate the test harness.

#### Docker networking

Uses `host.docker.internal` for cross-platform host access and bridge IP for
service-to-service communication.

### Development environment

The development environment is managed using [Flox][flox]. The justfile uses
`flox activate` as its shell, so all `just` recipes automatically run within the
Flox environment.

For ad-hoc commands outside of just:

```shell
flox activate -- <command>
```

## Quick reference

```bash
# Run all pre-commit checks (formatting, linting, tests)
just pre-commit

# Format code (REQUIRED before committing)
just format-rust true

# Run tests
just test-rust

# Lint
just lint-rust
```

### Helpful git commands

```bash
# Get commits since last release
git log <previous-tag>..main --oneline

# Check if contributor is first-time
git log --all --author="Name" --oneline | wc -l

# Get PR author username
gh pr view <number> --json author --jq '.author.login'

# View commit details
git show <commit> --stat
```

---

## Rust

### Edition and formatting

- Use Rust 2021 edition.
- Format with `just format-rust true`.
- Formatting is enforced in CI—always run `just format-rust` before committing.

### Module organization

- Do not use `mod.rs` files, prefer file-based modules.
- Private modules with public re-exports from `lib.rs` (no `pub mod`).
- Keep module boundaries strict with restricted visibility.
- Test helpers in dedicated modules/files.
- Use fully qualified imports rarely, prefer importing the type most of the
  time, or otherwise a module if it is conventional.
- Strongly prefer importing types or modules at the very top of the module.
  Never import types or modules within function contexts, unless the function is
  gated by a `cfg()` of some kind.
- It is okay to import enum variants for pattern matching, though.

### Memory and performance

- Use `Arc` or borrows for shared immutable data.
- Careful attention to copying vs. referencing.
- Stream data where possible rather than buffering.

### Dependencies

#### Workspace dependencies

- All versions managed in root `Cargo.toml` `[workspace.dependencies]`.
- Internal crates use exact version pinning: `version = "=0.2.1"`.
- Comment on dependency choices when non-obvious.

#### Key dependencies

- **anyhow**: Error handling with context.
- **getset**: Derive getters and setters for struct fields.
- **inventory**: Link-time test case collection.
- **libtest-mimic**: Custom test runner compatible with libtest.
- **reqwest**: HTTP client.
- **testcontainers**: Docker container management.
- **thirtyfour**: WebDriver client (Firefox/Selenium).
- **tokio**: Async runtime.
- **typed-builder**: Derive builder patterns for complex types.

### Type system

#### Enums over bools

Use enums with meaningful variants instead of bool parameters.

```rust
// DO
enum Visibility {
    Public,
    Private,
}

fn create_repo(name: &str, visibility: Visibility) {}

// DON'T
fn create_repo(name: &str, is_public: bool) {}
```

#### Derive conventions

- Builders with `typed-builder`
- Getters with `getset` (CopyGetters for Copy, Getters for references)
- Standard trait order: Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash,
  Debug, Default
- Third-party derives: alphabetical by crate, then by macro

```rust
// DO
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TestId(i64);

// Third-party: getset (CopyGetters, Getters), then typed-builder (TypedBuilder)
#[derive(
    Clone,
    Eq,
    PartialEq,
    Debug,
    CopyGetters,
    Getters,
    TypedBuilder,
)]
pub struct Server {
    #[getset(get = "pub")]
    image: String,
    #[getset(get_copy = "pub")]
    port: u16,
}

// DON'T
#[derive(Debug, Clone, TypedBuilder, Getters)]
pub struct Server {
    image: String,
    port: u16,
}
```

### Coding patterns

#### Control flow

- let-else for early returns
- Minimize if-let (only for short actions without else)
- Full match expressions (no matches! macro)
- Explicit variant matching (no wildcards except for #[non_exhaustive])

```rust
// DO: let-else for early returns
let Some(user) = get_user(id) else {
    return Err(Error::NotFound);
};

// DO: let-else in loops
let Some(value) = maybe_value else { continue };
let Ok(parsed) = input.parse::<i32>() else { continue };

// ACCEPTABLE: if-let for short action, no else
if let Some(callback) = self.on_change {
    callback();
}

// DO: full match expressions
let is_ready = match state {
    State::Ready => true,
    State::Pending => false,
    State::Failed => false,
};

// DON'T
let is_ready = matches!(state, State::Ready);

// DO: explicit variant matching
match status {
    Status::Pending => handle_pending(),
    Status::Active => handle_active(),
    Status::Completed => handle_completed(),
}

// DON'T: wildcards (except for #[non_exhaustive] types)
match status {
    Status::Pending => handle_pending(),
    _ => handle_other(),
}
```

If a wildcard match seems necessary, ask the user before using it.

#### Variables

- Shadow through transformations (no raw*, parsed* prefixes)
- Explicit destructuring for structs and tuples

```rust
// DO: shadow through transformations
let input = get_raw_input();
let input = input.trim();
let input = input.to_lowercase();
let input = parse(input)?;

// DON'T
let raw_input = get_raw_input();
let trimmed_input = raw_input.trim();
let lowercase_input = trimmed_input.to_lowercase();
let parsed_input = parse(lowercase_input)?;

// DO: explicit destructuring
let User { id, name, email } = user;
process(id, name, email);

// DON'T
process(user.id, user.name, user.email);

// DO: destructure in loops
for Entry { key, value } in entries {
    map.insert(key, value);
}

// DON'T
for entry in entries {
    map.insert(entry.key, entry.value);
}
```

#### Comments

- No inline comments (doc comments only)
- No section headers or dividers
- No TODO comments (use issue tracker)
- No commented-out code (use version control)

```rust
// DON'T
// Check if user is valid
if user.is_valid() {
    // Update the timestamp
    user.touch();
}

// --- Helper functions ---

// TODO: refactor this later
fn helper() {}

// Old implementation:
// fn old_way() { }

// DO
if user.is_valid() {
    user.touch();
}

fn helper() {}
```

### Error handling

- Use `anyhow` for error handling.
- Provide rich error context using `.context("description")?`.
- Error context messages should be lowercase sentence fragments suitable for
  "failed to {context}".

### Testing

#### Test organization

- Unit tests in the same file as the code they test.
- Test functions ordered alphabetically within modules.
- Name tests descriptively: `function_name_<condition>_<result>`, e.g.
  `goto_with_valid_path_navigates`.

#### Test structure

Use blank lines to separate Arrange/Act/Assert phases. Keep `.expect()` in the
Act phase, assertions should be plain `assert` calls:

```rust
#[tokio::test]
async fn parse_with_valid_input_returns_value() {
    let input = "42";

    let result = parse(input).expect("should succeed");

    assert_eq!(result, 42);
}
```

#### Error assertions

For error cases, use `expect_err` in the Act phase:

```rust
#[tokio::test]
async fn parse_with_invalid_input_returns_error() {
    let input = "not a number";

    let error = parse(input).expect_err("should fail");

    assert!(error.to_string().contains("invalid digit"));
}
```

#### Required tests

- Trait tests (Send, Sync, Unpin) for every custom type

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_send() {
        fn assert_send<T: Send>() {}
        assert_send::<MyType>();
    }

    #[test]
    fn trait_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MyType>();
    }

    #[test]
    fn trait_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<MyType>();
    }
}
```

#### Testability

- Extract business logic from framework wrappers into standalone functions.
- Tests must exercise the actual code, not adjacent implementations.

### Documentation

#### Summary line

- Third-person singular ("Returns the..." not "Return the...")
- No trailing period on summary

```rust
// DO
/// Returns the length of the string
/// Creates a new instance with default settings

// DON'T
/// Return the length of the string
/// Returns the length of the string.
```

#### Comment style

Use line comments (`///`), not block comments (`/** */`).

```rust
// DO
/// Summary sentence here
///
/// More details if needed.

// DON'T
/**
 * Summary sentence here
 *
 * More details if needed.
 */
```

#### Required sections

- `# Errors` for fallible functions
- `# Panics` for functions that can panic
- `# Safety` for unsafe functions
- `# Examples` for public items

Use these exact headings (always plural): Examples, Panics, Errors, Safety,
Aborts, Undefined Behavior.

````rust
/// Reads a file from disk
///
/// # Errors
///
/// Returns [`io::Error`] if the file does not exist or cannot be read.
///
/// # Panics
///
/// Panics if the path is empty.
///
/// # Examples
///
/// ```
/// let contents = read_file("config.toml")?;
/// ```
///
/// [`io::Error`]: std::io::Error
````

#### References

- Use [`Type`] with reference-style links
- Full generic forms: [`Option<T>`] not `Option`

```rust
// DO
/// Returns [`Option<T>`] if the value exists
///
/// [`Option<T>`]: std::option::Option

// DON'T
/// Returns `Option` if the value exists
```

#### Depth

Documentation should explain the "why", not just the "what":

- **Types**: Explain design decisions, invariants, and relationships to other
  types
- **Functions**: Document side effects, caller considerations, and non-obvious
  behavior
- **Modules**: Explain the module's role in the system and key concepts

```rust
// DO: Explain design decisions
/// Thread-safe counter for tracking active connections
///
/// Uses [`AtomicUsize`] instead of `Mutex<usize>` because the counter is
/// only incremented and decremented, never read-then-modified, making atomic
/// operations sufficient and avoiding lock contention under high load.
///
/// [`AtomicUsize`]: std::sync::atomic::AtomicUsize

// DON'T: Just restate the type name
/// A connection counter
pub struct ConnectionCounter {
    ...
}
```

#### Module vs type docs

- Module docs: high-level summaries, when to use this module.
- Type docs: comprehensive, self-contained.
- Some duplication between module and type docs is acceptable.

#### Language

Use American English spelling: "color" not "colour", "serialize" not
"serialise".

---

## Markdown

- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.
- Use reference-style Markdown links, not inline links.
- Table cells must be single-line. Markdown does not support multi-line cells;
  each newline starts a new row. Ignore line length limits for table rows.

## Git

### Commit messages

We write commit messages inspired by [tbaggery][tbaggery]:

- Capitalized, short (50 chars or less) summary
- Imperative mood: "Fix bug" not "Fixed bug" or "Fixes bug"
- Focus on the goal of the change, not implementation details. The body should
  describe what the change accomplishes and why, not enumerate every file or
  component touched.
- Keep formatting minimal. Avoid heavy use of bold, bullet lists, or headings in
  commit bodies. Plain prose is preferred.
- Start body sentences with a subject. "This change introduces…", "We learned…",
  "The migration simplifies…" — not dangling participles like "Learned from…" or
  "Introduces…".
- Explain the "why" and the trade-offs of the change
- Use simple past and present tense in body: "Previously, when the user did X, Y
  used to happen. With this commit, now Z happens."
- **Never** write conventional commit messages
- Commit messages should be Markdown. Don't use backticks in commit message
  titles, but do use them in bodies.

### Commit quality

- **Never commit directly to main**: Always create a feature branch and submit a
  pull request.
- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate
  commits from feature changes.
- **Diff against the baseline when reversing or modifying a prior commit**: use
  `git diff <commit>~1` (against the working tree) to verify you haven't
  introduced unintentional changes relative to the pre-commit state.

### Pull requests

Create pull requests using `gh pr create --fill --assignee @me` to derive the
title and body from the commit message and assign the PR to yourself.

#### Labels

Pull requests must be labeled for release note categorization. Area labels
(`A-*`) indicate the affected component. Release labels (`R-*`) control how the
PR appears in auto-generated release notes:

| Label          | Release notes section |
| -------------- | --------------------- |
| `R-added`      | Added                 |
| `R-changed`    | Changed               |
| `R-deprecated` | Deprecated            |
| `R-removed`    | Removed               |
| `R-fixed`      | Fixed                 |
| `R-security`   | Security              |
| `R-ignore`     | Excluded              |

Area labels: `A-doco`, `A-derive`, `A-docs`, `A-github-actions`.

### Releases

Releases follow [Keep a Changelog][keep-a-changelog] and [Semantic
Versioning][semver].

1. Update `CHANGELOG.md`: move items from `[Unreleased]` into a new version
   section dated today.
2. Bump the version in the root `Cargo.toml` `[workspace.package]` and in
   `crates/doco/Cargo.toml` (the `doco-derive` dependency version).
3. Run `cargo check` to update `Cargo.lock`.
4. Commit, open a PR, and merge.
5. Create a GitHub release with tag `X.Y.Z` (no `v` prefix) targeting main. The
   release workflow automatically publishes to crates.io (`just publish` runs
   `doco-derive` first, then `doco`).

---

## Acknowledgments

This `CLAUDE.md` file was adopted from [nextest's AGENTS.md][nextest-agents],
which is published under the Apache-2.0 or MIT license.

[flox]: https://flox.dev
[keep-a-changelog]: https://keepachangelog.com/en/1.0.0/
[nextest-agents]: https://github.com/nextest-rs/nextest/blob/main/AGENTS.md
[semver]: https://semver.org/spec/v2.0.0.html
[tbaggery]:
  https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
