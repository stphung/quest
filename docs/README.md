# docs/

**What the game *does* is specified in [`openspec/specs/`](../openspec/README.md)**
— a reverse-engineered, code-grounded baseline of 20 capabilities and the single
source of truth. Start non-trivial work from a spec via `/opsx:propose`.

`docs/` holds the **design process** that OpenSpec's behavioral specs don't
capture: exploration, evolving system/world design, and resolved decisions —
plus assets.

## Where does this doc go? (the boundary rule)

The whole system hinges on one question — *is this design owned by a single
shippable change?*

| Kind of thing | Home | Lifecycle |
|---|---|---|
| What the game **does** now | `openspec/specs/` | Living source of truth |
| Design + plan **for one change** (approach, alternatives, tone/UX) | that change's `design.md` / `tasks.md` | Preserved in `openspec/changes/archive/` on ship |
| **Pre-commitment** exploration ("what should Act 3 be?") | [`explorations/`](explorations/) | Disposable; graduates to a change or dossier |
| **Evolving** per-system / world design | [`dossiers/`](dossiers/) | Living, refreshed manually as systems change |
| A **resolved decision** + rationale | [`decisions.md`](decisions.md) | Append-only log |
| Editorial / QA reviews | [`reviews/`](reviews/) | As-written |
| Storyboards, screenshots | `storyboards/`, `screenshots/` | Assets |
| Player-facing prose (letters, scene text) | in code/data | Content, not docs |

If it belongs to one shippable change → it lives in that change (and is archived
with it). If it precedes or outlives any single change → it lives here.

## The design-rationale trail lives in OpenSpec

Historical per-feature design and plans were **backported into
`openspec/changes/archive/`** — each shipped feature is an archived change with
its `proposal.md` / `design.md` / `tasks.md`, mapped to the capability it
touched. That is now the durable record of *how and why* things were built.
**Never delete `openspec/changes/archive/`.** (The former top-level system docs
and the `docs/design/` + `docs/plans/` + `docs/superpowers/` trees were removed
in favor of that archive and `openspec/specs/`; full history remains in git.)

## What's here now

| Path | What it is |
|------|-----------|
| `decisions.md` | Design decisions log with rationale |
| `dossiers/` | Living per-system dossiers, incl. the cross-act `world-and-narrative.md` bible |
| `explorations/` | Pre-proposal scratch notes |
| `reviews/` | Editorial and testing-strategy reviews |
| `storyboards/`, `screenshots/` | Visual assets |

When a design note and the code disagree, trust the code and reconcile the
relevant `openspec/specs/` requirement.
