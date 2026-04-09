# Contributing to Scrutt

Thanks for taking the time to contribute to **Scrutt** — a local supply-chain firewall for the Node.js ecosystem written in Rust.

This project is still early, so the workflow is intentionally strict. The idea is simple: keep `main` stable, keep changes reviewable, and make the history tell a real development story.

---

## Table of Contents

- [Issue-First Workflow](#issue-first-workflow)
- [Label System](#label-system)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Commit Convention](#commit-convention)
- [Branch Naming](#branch-naming)
- [Pull Request Rules](#pull-request-rules)
- [Code of Conduct](#code-of-conduct)

---

## Issue-First Workflow

**No PR without an issue.**

The only exception is the initial bootstrap PR that establishes the repository foundation. That first PR may ship without a linked issue because it exists to create the project baseline itself.

After the bootstrap PR is merged, the issue-first workflow is mandatory for every feature, fix, refactor, docs change, and project phase.

Before writing code:

1. Open an issue describing the bug, feature, refactor, or docs change.
2. Wait for a maintainer to approve it with the `status:approved` label.
3. Comment on the issue if you're going to work on it.
4. Create a branch from `main`.
5. Open a PR that references the approved issue.

If a PR is not linked to an approved issue, it should not be merged.

This sounds strict because it is strict. Early projects get messy FAST if every idea goes straight into code.

---

## Label System

### Type Labels (applied to PRs)

| Label | Description |
|-------|-------------|
| `type:bug` | Bug fix |
| `type:feature` | New feature or enhancement |
| `type:refactor` | Refactor with no intended behavior change |
| `type:docs` | Documentation only |
| `type:test` | Test coverage additions or test-only changes |
| `type:chore` | Tooling, setup, repository maintenance |
| `type:breaking` | Breaking change |

### Status Labels (applied to issues)

| Label | Description |
|-------|-------------|
| `status:needs-review` | Newly opened and waiting for maintainer review |
| `status:approved` | Approved for implementation |
| `status:in-progress` | Someone is actively working on it |
| `status:blocked` | Blocked by another task or dependency |
| `status:wont-fix` | Not planned |

### Priority Labels

| Label | Description |
|-------|-------------|
| `priority:critical` | Security issue or blocking problem |
| `priority:high` | Important work with broad impact |
| `priority:medium` | Normal priority |
| `priority:low` | Nice to have |

---

## Development Setup

### Prerequisites

- Rust stable
- Cargo
- Git

### Clone and Test

```bash
git clone https://github.com/Jelsin29/scrutt.git
cd scrutt
cargo test
```

### Run Locally

```bash
cargo run -- shield tests/fixtures/valid
```

Right now the project is small on purpose. If you're expecting the full product vision already, not yet.

---

## Testing

Run the full test suite:

```bash
cargo test
```

Run integration tests only:

```bash
cargo test --test shield_command
```

Run a specific test:

```bash
cargo test loads_dependency_counts_from_fixture
```

If your change adds behavior, it should add or update tests. If it fixes a bug, add a regression test.

---

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

Commit messages must match this pattern:

```text
^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9\._-]+\))?!?: .+
```

### Format

```text
<type>(<optional-scope>)!: <description>
```

### Examples

```text
feat(cli): add shield command
fix(pkg-json): handle missing package.json
test(cli): cover invalid manifest fixture
docs: add contributing guide
chore(repo): add branch protection docs
```

### Breaking Changes

Use `!` and explain the break clearly:

```text
feat(cli)!: rename shield output format

BREAKING CHANGE: the shield output format changed and may break scripts that parse it.
```

---

## Branch Naming

Branch names must match this pattern:

```text
^(feat|fix|chore|docs|style|refactor|perf|test|build|ci|revert)\/[a-z0-9._-]+$
```

### Rules

- all lowercase
- keep it short and descriptive
- use hyphens, dots, or underscores as separators
- branch from `main`
- merge back through a PR — never push directly to `main`

### Examples

- `feat/shield-summary`
- `fix/missing-package-json`
- `docs/contributing-guide`
- `ci/add-pr-checks`

---

## Pull Request Rules

### Before Opening a PR

- [ ] There is a linked approved issue (`Closes #<N>`, `Fixes #<N>`, or `Resolves #<N>`)
- [ ] Tests pass (`cargo test`)
- [ ] Commits follow Conventional Commits
- [ ] The branch name follows the repository pattern
- [ ] The code was self-reviewed before requesting review

### PR Title

Use the same Conventional Commits format as commit messages:

```text
feat(cli): add shield summary output
fix(error): improve invalid manifest message
docs: add contribution rules
```

### Expected PR Checks

PRs should eventually be blocked on these checks:

| Check | What it verifies |
|-------|------------------|
| Issue reference | PR body contains `Closes/Fixes/Resolves #N` |
| Approved issue | The linked issue has `status:approved` |
| Type label | Exactly one `type:*` label is applied |
| Tests | `cargo test` passes |

Not all automation is in place yet, but contributors should follow these rules now, not later.

---

## Code of Conduct

Be respectful and direct.

- Critique code, not people
- Explain the tradeoff, not just the preference
- Help newcomers understand the reasoning
- Keep security discussions serious and evidence-based

---

## Questions?

If you want to discuss an idea before turning it into code, open an issue first. That's the cleanest place to start while the project is still taking shape.
