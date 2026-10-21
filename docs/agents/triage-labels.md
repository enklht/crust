# Triage statuses

Store these canonical role strings in each issue’s `Status:` line.

| Role / status | Meaning |
| --- | --- |
| needs-triage | Maintainer needs to evaluate the issue |
| needs-info | Waiting for more information |
| ready-for-agent | Fully specified and ready for an agent |
| ready-for-human | Requires human implementation |
| wontfix | Will not be actioned |

When a skill requests a triage label, update the local Status line.
Wayfinding also uses claimed and resolved as lifecycle statuses.
