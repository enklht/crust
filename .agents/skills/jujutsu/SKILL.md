---
name: jujutsu
description: Repository-specific Jujutsu workflows for change descriptions, complex revsets, conflicts, bookmarks, history rewriting or recovery, remote synchronization, and PR creation.
---

# Jujutsu operations

The always-applicable policy lives in [AGENTS.md](../../../AGENTS.md).
This skill covers repository-specific decisions and procedures beyond
standard Jujutsu behavior.

## When to use this skill

- Writing change descriptions
- Writing or evaluating complex revsets
- Resolving conflicts
- Creating, moving, tracking, untracking, or deleting bookmarks
- Rebasing, squashing, restoring, abandoning, undoing, or restoring operations
- Splitting changes without a diff editor
- Handling stale working copies or immutable revision errors
- Fetching, pushing, or creating PRs

For routine inspection and starting ordinary work, follow `AGENTS.md`
directly; this skill is not required.

## Repository constraints

- Use `jj` commands; direct `git` commands are forbidden. `jj git ...`
  and `gh` are allowed.
- Never run `jj resolve`, `jj diffedit`, or `jj arrange`; use the
  noninteractive procedures below. Run `jj split` only with an explicit
  fileset and `-m`, never without arguments or with `-i` or `--tool`.
  If a task cannot be completed safely without an interactive editor,
  ask the user to perform that part.
- Always pass `--git` to `jj diff`, `jj show`, and `jj log -p`.
- After history-changing operations such as `jj new`, `jj rebase`,
  `jj squash`, `jj restore`, or `jj abandon`, run `jj status` before
  continuing and resolve any conflicts.
- Preserve stacked changes when preparing a PR; never squash the stack.

## Change descriptions

Write English Conventional Commits descriptions using
`<type>: <emoji> <summary>`. Use the literal emoji shortcode paired
with the type below, for example:
`docs: :memo: document semantic change descriptions`.

| Type | Emoji shortcode | Meaning |
| --- | --- | --- |
| `feat` | `:sparkles:` | A new feature |
| `fix` | `:bug:` | A bug fix |
| `docs` | `:memo:` | Documentation only changes |
| `style` | `:lipstick:` | Changes that do not affect the meaning of the code |
| `refactor` | `:recycle:` | A code change that neither fixes a bug nor adds a feature |
| `perf` | `:zap:` | A code change that improves performance |
| `test` | `:white_check_mark:` | Adding missing tests or correcting existing tests |
| `build` | `:package:` | Changes that affect the build system or external dependencies |
| `ci` | `:ferris_wheel:` | Changes to CI configuration files and scripts |
| `chore` | `:hammer:` | Other changes that don't modify src or test files |
| `revert` | `:rewind:` | Reverts a previous commit |

## Snapshot discipline

Use `--ignore-working-copy` only for read-only inspection, such as
`jj log`, `jj diff`, `jj bookmark list`, `jj evolog`, or `jj op log`,
after confirming that the working copy has already been snapshotted.
This flag suppresses new snapshots and can return stale state.

Omit it at the start of work and immediately after creating, editing,
or deleting files, when the current filesystem state must be captured.

## Split a change by file

```bash
jj split -m "<description of the extracted change>" <path>...
jj status
```

The explicit fileset prevents the diff editor from opening; `-m`
prevents the text editor from opening. Leave hunk-level splitting,
which requires the diff editor, to the user.

## Resolve conflicts

Edit conflict markers directly into the intended final contents, then
run `jj status` to confirm that no conflicts remain. Saving the file
resolves the conflict. Never run `jj resolve`; no staging step is needed.

## Place a bookmark on the revision to publish

Leave bookmarks in place during ordinary implementation. Place a
bookmark only when the user requests a push or PR creation, including
a PR request that implicitly requires a push.

Do not assume `@` is the revision to publish: after `jj new`, it may be
empty. Inspect the latest ancestors of `@` that have both content and
a description before placing a bookmark. Snapshot the working copy
first, then inspect:

```bash
jj status
jj log --ignore-working-copy --no-graph \
  -r 'heads(::@ & ~empty() & ~description(exact:""))' \
  -T 'change_id.short() ++ " " ++ description.first_line() ++ "\n"'
```

Confirm that the result identifies the intended publication target. If
it returns multiple revisions, inspect them and select the intended
revision explicitly rather than guessing. For a single confirmed target,
use one of:

```bash
jj bookmark create <name> -r 'heads(::@ & ~empty() & ~description(exact:""))'
jj bookmark move <name> --to 'heads(::@ & ~empty() & ~description(exact:""))'
jj status
```

Use `create` for a new bookmark and `move` for an existing one. Never
use `jj bookmark advance`; specify the target revision at the call site.

## Remote and PR workflow

Unless instructed otherwise, use `main` as the PR base and preserve
each change in the stack. If a remote bookmark is untracked, start
tracking it with `jj bookmark track <name>@origin`. Skip pushing when
the local and remote bookmarks already match.

Pass an explicit Conventional Commits title and write the PR body in
English. For a multiline body, write it to a temporary file and use
`--body-file`.

```bash
jj git fetch
jj status
jj log --ignore-working-copy
jj bookmark list --all --ignore-working-copy
# Inspect the results, then place the bookmark as described above.
jj git push -b <name>
gh pr create --base main --head <name> --title "<type>: <summary>" --body-file <body-file>
```

## Recovery and troubleshooting

Before `jj undo`, `jj op restore`, `jj restore`, or `jj abandon`,
inspect the affected operation or revisions and confirm that they
exactly match the intended target. Never discard or rewrite the user's
work based on a guessed target. Run `jj status` after the operation.

**Stale working copy:** Run `jj workspace update-stale`, then `jj status`.

**Immutable revision:** Respect immutability. Inspect the target,
select the intended mutable change, and retry the operation there.
