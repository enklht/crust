# Project principles

Crust is a C compiler written in Rust, following Nora Sandler’s
Writing a C Compiler.

Simplicity, beauty, and faithfulness to the book guide implementation
decisions. Prefer clear, direct code and the book’s terminology,
compiler stages, and intermediate representations.

## Coding style

Prefer code whose names, types, and structure explain the algorithm.
Use comments for rationale, constraints, and subtle invariants that the
code cannot express. When code needs an explanation of its structure,
first improve its naming, decomposition, or control flow.

- Make compiler stages and their order visible in orchestration code.
- Name meaningful pieces before composing them, as with parser rules.
- Let modules, functions, and data types follow the language's structure
  and the book's terminology.
- Keep each rule close to its implementation. Accept repetition when
  it preserves that correspondence. A long match can be appropriate
  when it expresses one coherent case analysis.
- Introduce small helpers that express domain concepts and improve
  readability. Avoid generalization solely to remove repeated lines.
- Use concise names when the surrounding context makes them clear,
  such as `Exp`, `Decl`, `typ`, `src`, and `dst`.
- Choose Rust constructs to suit the task: combinators for grammar,
  iterators for transformations, and loops and explicit mutation for
  sequential work.

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

## Version control

This project uses Git. Inspect `git status` and `git diff` before editing,
and preserve unrelated working changes. Stage files explicitly to keep
commits focused.

Unless instructed otherwise, target PRs at `main`.

### Commit messages

Write commit messages in English using `<type>: <emoji> <summary>`.
Use the literal emoji shortcode paired with the type below, for example:
`docs: :memo: document commit message conventions`.

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
