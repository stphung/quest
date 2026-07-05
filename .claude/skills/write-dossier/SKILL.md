---
name: write-dossier
description: Write a new docs/dossiers/*.md design dossier or refresh an existing one — a living, cross-system, player-eye synthesis of an Act or major system, grounded in source citations and simulator evidence. Use when asked to create a dossier, refresh a dossier, or check whether a dossier reflects the current design.
---

<!-- meta-audit scope-universe: docs/dossiers/*.md -->

# Write or Refresh a Design Dossier

A dossier answers one question a capability spec can't: *what is this Act
or system, how does it interrelate with everything else, is it balanced,
and is it fun* — written player-eye first, every mechanical claim cited to
source, every balance claim backed by a real run, not a guess.

`docs/dossiers/README.md` states the contract dossiers exist under:
specs (`openspec/specs/`) are the living source of truth for what a system
*does*; archived changes (`openspec/changes/archive/`) are point-in-time
design intent per shipped change; `docs/decisions.md` is the resolved-
decision log. A dossier is none of those — it's the synthesis that would
otherwise have no home, refreshed manually as the system evolves.

## When to Use

- Asked to write a dossier for an Act, system, or feature that doesn't have one
- Asked to "refresh" an existing dossier
- Asked whether a dossier "reflects the current design" or is stale
- A `/goal` or reviewer flags a specific concept missing from a dossier
- Before a release, as part of the `audit` skill's docs sweep (`doc-audit`
  flags dossiers whose `Last refreshed` sha is far behind HEAD, but does
  not itself deep-refresh them — that's this skill's job)

Two existing dossiers are the reference implementation for structure and
tone: [`act1-ascent.md`](../../../docs/dossiers/act1-ascent.md) (single-pass,
freshly written) and [`act2-pilgrimage.md`](../../../docs/dossiers/act2-pilgrimage.md)
(living, refreshed across many sessions — read its `Refresh History` section
to see how a dossier accumulates history without cluttering its current-state
sections). When in doubt about format, match those two, not this skill's prose.

## The Dossier Shape

Every dossier uses the same section skeleton, in this order. Keep section
*names* identical across dossiers (not just similar) so they read as one
comparable set, not N one-off documents.

1. **Title** — `# <Act/System Name> — Design Dossier` (or `# Dossier: <Name>` for a cross-cutting one like `world-and-narrative.md`).
2. **Header line** — `> Last refreshed: <date> @ <short-sha> | Sources: <list of src/ paths, spec paths, archived-change paths, test/simulator commands>`.
3. **Status blockquote** — one paragraph: what this dossier covers, which specs/archives/decisions it complements (not replaces), links to sibling dossiers (an Act dossier links the other Act's dossier and the cross-act dossier), and — for living dossiers only — a pointer to the bottom `Refresh History` section so readers know the sections above are current-state-only.
4. `## The Player's Experience` — narrative prose. Walk a fresh player through the system/Act as they'd actually encounter it, in order. This is the only section allowed to read like fiction-adjacent prose rather than a reference; still name every mechanic precisely enough that a developer recognizes it.
5. `## Design Intent` — bullet list reconstructing *why*, not *what*. Pull from `docs/decisions.md` entries, archived `changes/archive/<name>/design.md` or `proposal.md` docs, and module `CLAUDE.md` design-rationale asides. Cite the specific decision-log heading or archived doc section for each bullet, not just "the design doc."
6. `## Mechanics & Constants` — the dense reference core. One `###` subsection per subsystem (not one wall of prose — see Anti-Patterns). Every number, formula, and behavior claim carries a `file.rs:line` citation. This is the section `doc-audit`/`meta-audit` hold to a factual standard, so it must stay accurate under future refreshes.
7. `## Interrelations` — an ASCII diagram plus bullets: what flows *in* from other Acts/systems, what's mechanically isolated ("during"), what flows *out* (the hooks a future Act/system keys off), and any internal closed loops worth naming. Call out every named hook/flag explicitly (e.g. two different Act-3-hook flags, not just "the Act 3 hook").
8. `## Balance Evidence` — dated, sourced numbers from an actual run you performed this session (simulator, targeted test, `--check-progression`), not a re-statement of a design doc's aspirational numbers. Tables over prose. Note when thresholds carry intentional headroom.
9. `## Fun Assessment` — score against the same seven heuristics every dossier uses (below). Dated. Cite concrete evidence per row, not vibes.
10. `## Open Questions & Decision History` — numbered list. Resolved items get `~~struck-through~~` text + **Resolved**: rationale; genuinely open items stay unstruck and get a one-line reason they're not urgent (or are). Don't silently drop a resolved item on refresh — the history is part of the value.
11. `## Refresh History` (living dossiers only — omit entirely on a dossier's first version) — session-by-session log, most recent first, of what changed at each refresh and why. This is what lets sections 4-10 stay current-state-only instead of accumulating "as of the previous session..." caveats inline.
12. `## Sources` — closing bullet list of every spec, archived change, CLAUDE.md, decisions.md, and test/simulator command cited above.

### The seven Fun Assessment heuristics (shared across all dossiers)

1. Visible next goal
2. Wall → reset → power
3. Discovery cadence
4. Cross-system braiding
5. Decision density
6. Anticipation instruments
7. Stakes and texture

These originated in `docs/dossiers/world-and-narrative.md`'s "Design
guardrails" section (Act 1's own benchmarks) and are reused verbatim so
dossiers can be read side by side as a deliberate comparison — e.g.
`act1-ascent.md` and `act2-pilgrimage.md` each explicitly note where the
two Acts are *scored differently on purpose* (Act 1 optimizes braiding and
discovery; Act 2 optimizes anticipation and stakes) rather than treating a
low score as an unfixed gap.

## Writing a New Dossier

### Phase 1 — Scope the unit

Identify which `openspec/specs/<capability>/spec.md` capabilities the
dossier covers (check `openspec/README.md`'s capability index), and which
`docs/decisions.md` entries and `openspec/changes/archive/` directories are
relevant. A dossier can cover one Act (many capabilities) or one system —
match the granularity of what's actually shipped as one coherent player
experience, not an arbitrary file boundary.

### Phase 2 — Parallel grounded research

Spawn several `Explore` or `general-purpose` agents in parallel (background
is fine — you don't need results until you write), each scoped to a cluster
of subsystems (e.g. for a 50-zone Act: core arc / endgame layered systems /
side systems / meta systems, as four separate agents). Each agent's brief
must demand:

- Exact `file:line` citations for every fact, read from source directly
- Cross-checking `openspec/README.md`'s "Known code-vs-docs discrepancies"
  list for the area, and flagging any *new* discrepancy found beyond it
- A short "most interesting/surprising design fact" per subsystem, to seed
  Player's Experience and Design Intent prose later

Do not let agents write final dossier prose — they return structured fact
sheets; you synthesize into one consistent voice. Mixing agent voices into
the final doc is the most common way a dossier reads like a committee wrote it.

### Phase 3 — Balance evidence, run yourself

Build the release simulator (`cargo build --release --bin simulator`) and
run `--check-progression` plus a few targeted strategy/seed sweeps for the
scenarios the dossier's Player's Experience section describes. For a
non-Act-1 system with its own simulator/test harness (Deep, Loom, Voyage),
use that instead. A dossier's Balance Evidence section must report numbers
*you* produced this session, not numbers copied from a design doc's intent.

### Phase 4 — Write

Follow the Dossier Shape section above exactly. Write Mechanics & Constants
last (it's the most mechanical to assemble from the research fact sheets)
and Player's Experience first in your own drafting order if that helps, but
the *sections* still appear in the fixed order in the file.

### Phase 5 — Completeness audit (do this even on a "finished" first draft)

Before considering a new dossier done, run one dedicated pass hunting
specifically for **named design concepts that are real, shipped, and
load-bearing, but not reflected under their name** — this is a distinct
failure mode from a wrong number or a missing mechanic description, and
plain fact-gathering agents routinely miss it because they describe
mechanisms without noticing the mechanism has a name the design treats as
its identity. Concretely, hunt in:

- `docs/decisions.md`, read in full (not grepped) — named philosophies and
  formulas ("the golden ratio", "Frontier Backoff") often sit in prose,
  not in a bullet a keyword search would catch
- `openspec/changes/archive/<name>/design.md` and `proposal.md` for the
  relevant systems — section headers and bolded terms are usually the
  design's own vocabulary for a pillar, not incidental phrasing
- Module `CLAUDE.md` files and the module's own doc comments (`//!` headers
  in `.rs` files) for a self-declared identity — e.g. a module doc comment
  that files itself under "sub-project N, The <Name>" is naming something
  the dossier should name too, not just mechanically describe

This is not a one-time step reserved for the first draft — re-run it on
every refresh, since a session that adds a new named system (a new refit
mechanic, a new named hazard, a new title for a loop) is exactly the kind
of change a "did the numbers change" refresh check will miss.

### Phase 6 — Verify every finding against source yourself

Treat an audit/research agent's report as a lead, not a fact. Before adding
anything to the dossier, `grep`/`Read` the cited file:line yourself and
confirm the constant, function, or comment actually says what's claimed.
This project's dossiers are held to a "cited to source" standard by
`doc-audit`/`meta-audit` — an unverified agent claim that turns out wrong
undermines that standard immediately.

### Phase 7 — Cross-link

Update the dossier's own Status blockquote to link sibling dossiers, and
check whether `world-and-narrative.md` (or the equivalent cross-cutting
index for non-Act dossiers) needs a pointer added. Don't edit
`docs/dossiers/README.md` unless the dossier introduces a genuinely new
*kind* of dossier the README's table doesn't describe — it's deliberately
generic and doesn't enumerate files by name.

## Refreshing an Existing Dossier

1. Diff the dossier's `Last refreshed @ sha` against current HEAD:
   `git log --oneline <sha>..HEAD -- <the paths its Sources line lists>`.
   Nothing landed → the dossier is already current; say so rather than
   editing for the sake of editing.
2. If only doc/path structure moved (a migration, a rename) with no
   mechanics change, that's a **housekeeping pass**: fix references, add a
   short `Refresh History` entry explaining what moved and why, leave
   Mechanics & Constants alone. (Precedent: the OpenSpec-migration fix to
   `act2-pilgrimage.md`'s citations.)
3. If real mechanics changed, re-run Phases 2-6 above scoped to just the
   changed area, update the affected sections in place (Mechanics &
   Constants, Interrelations, Balance Evidence, Fun Assessment as needed),
   and add a `Refresh History` entry summarizing the delta — don't leave
   the change-log narrative sitting inline in the sections above it.
4. If a specific concept was flagged missing (by a reviewer or a `/goal`),
   don't only fix that one thing — it's a signal a full Phase 5
   completeness audit is due, since if one named concept slipped through,
   others likely did too. Both real refreshes in this skill's own history
   found multiple gaps once triggered by a single reported one.

## Anti-Patterns

- **One dense prose block for Mechanics & Constants.** Split by subsystem
  with `###` headers — a reader should be able to jump to "Ascension" or
  "The Deep" without reading everything before it.
- **Citing a design doc's intent as if it shipped.** Always state what's
  *live in source today*; when a design doc's numbers were superseded,
  say so and cite the follow-up that superseded them (both dossiers do
  this for retired mechanics — follow the same pattern).
- **Silently fixing a discrepancy that spans normative docs.** If a gap
  reaches into `openspec/specs/*/spec.md` or a module `CLAUDE.md` (not just
  the dossier itself), log it as an Open Question rather than editing those
  files yourself — that's a separate decision outside a dossier refresh's
  scope.
- **Re-scoring Fun Assessment without new evidence.** Only move a score
  when new Balance Evidence or a played session actually supports the
  change; note deltas explicitly ("4/5, was 3/5 — evidence: ...") rather
  than silently bumping numbers.
- **Skipping your own simulator/test run.** A design doc's aspirational
  numbers are not Balance Evidence. If you can't run the relevant harness,
  say so explicitly rather than presenting old numbers as current.
- **No silent caps.** If you only spot-checked N of M archived changes, or
  scoped the completeness audit to specific modules, say so in the dossier
  or in your summary to the user — don't imply full coverage you didn't do.
