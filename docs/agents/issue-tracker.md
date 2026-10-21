# Issue tracker: Local Markdown

Issues and specs live in `.scratch/`. Use local files for tracker
operations.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`.
- Spec: `.scratch/<feature-slug>/spec.md`.
- Tickets: `.scratch/<feature-slug>/issues/<NN>-<slug>.md`,
  numbered from `01`, one file per ticket.
- Record triage state as a `Status:` line near the top.
  Use the strings in `triage-labels.md`.
- Append discussion under `## Comments`.

When a skill says to publish to the issue tracker, create the
appropriate local file. When it says to fetch a ticket, read the
referenced file.

## Wayfinding operations

- Map: `.scratch/<effort>/map.md`, containing Notes,
  Decisions-so-far, and Fog.
- Child tickets: `.scratch/<effort>/issues/<NN>-<slug>.md`.
  Record `Type: research`, `prototype`, `grilling`, or `task`.
- Dependencies: `Blocked by: NN, NN` near the top.
  A ticket is unblocked when all listed tickets are resolved.
- Frontier: first ticket by number that is open, unblocked,
  and unclaimed.
- Claim: save `Status: claimed` before starting work.
- Resolve: append findings under `## Answer`, set
  `Status: resolved`, and add a summary and link to the map’s
  Decisions-so-far.
