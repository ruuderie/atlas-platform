# Private docs (local only)

**Everything under `docs/private/` is intentionally not published to GitHub.**

Use this tree for:

- Future / unreleased product specs (`famtasm/`, `claim-swift/`, …)
- UI specs (`*/ui-spec/`)
- Backlog explorations, GTM, market reports, strategy
- Stitch dumps and product research notes

## Rule

| Path | Visibility |
|------|------------|
| `docs/private/**` | Private — gitignored |
| `docs/private/README.md` | Public — this file only |
| `docs/folio/`, `docs/architecture/`, `docs/contracts/`, runbooks | Public platform docs |

Agents and humans: put new product or research writing **here**, not under `docs/` root.

If you need a private git remote for backup/sync, point it at this tree (or a sibling `atlas-specs` repo) — do not force-add these paths to `origin`.
