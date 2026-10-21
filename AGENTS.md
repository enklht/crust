# Project principles

Crust is a C compiler written in Rust, following Nora Sandler’s
Writing a C Compiler.

Simplicity, beauty, and faithfulness to the book guide implementation
decisions. Prefer clear, direct code and the book’s terminology,
compiler stages, and intermediate representations.

## Agent skills

### Issue tracker

Track issues and specs locally in `.scratch/`.
See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five default triage statuses in local issue files.
See `docs/agents/triage-labels.md`.

### Domain docs

Use a single-context layout: root `GLOSSARY.md` and `docs/adr/`.
See `docs/agents/domain.md`.

## Version control — required workflow

This project uses Jujutsu (`jj`) for version control. Use `jj` terminology
and commands. Never run `git` directly; `jj git ...` and `gh` are allowed.

Before editing, run `jj status` to snapshot the working copy and inspect
the working-copy commit (`@`). Reuse `@` only when it has neither a
description nor a diff; otherwise, create a new change. Write change
descriptions in English using `<type>: <emoji> <summary>` with the
types and literal emoji shortcodes in the
[`jujutsu` skill](.agents/skills/jujutsu/SKILL.md#change-descriptions).

Always pass `--git` to `jj diff`, `jj show`, and `jj log -p`. Use
`--ignore-working-copy` only for read-only inspection after confirming
that the working copy has already been snapshotted.

Never run `jj resolve`, `jj diffedit`, or `jj arrange`. Run `jj split`
only with an explicit fileset and `-m`; invocation without arguments,
`-i`, and `--tool` are forbidden. These constraints prevent editors from
opening. After any history-changing operation, run `jj status` and
resolve any conflicts before continuing.

Move bookmarks only when publishing changes. Unless instructed otherwise,
target PRs at `main`. Never squash stacked changes when preparing a PR.

For change descriptions, complex revsets, conflicts, bookmark operations,
history rewriting or recovery, fetch, push, and PR creation, read the
[`jujutsu` skill](.agents/skills/jujutsu/SKILL.md).
