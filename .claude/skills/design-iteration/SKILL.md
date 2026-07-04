---
name: design-iteration
description: Design-iteration loop for evolving game systems with the human designer in the decision seat — dissects a system into a living dossier, gathers balance evidence from simulators and real play, audits fun against Act 1's benchmarks, then surfaces the genuine design decisions instead of making them. Use when asked "where are we with Act 2", "dissect the Deep", "design review of X", "is X fun", "walk me through the system", "what's changed since I last looked", or when iterating on any game system's design (not its implementation details).
---

# Design Iteration Loop

Quest's systems get built fast, and the designer (the user) cannot be inside
every diff. This skill keeps them in the decision seat anyway. It exists
because Act 1 turned out well *because* the designer continuously dissected
how things worked — this skill reproduces that dissection as a repeatable
loop, so Act 2 and beyond get the same treatment without the designer having
to personally excavate the code each time.

The loop produces three things:

1. A **living dossier** per system (`docs/dossiers/<system>.md`) — the durable,
   refreshable understanding of what the system is, how it interrelates, and
   how it's balanced.
2. A **state-of-the-system briefing** in chat — what changed since the
   designer last looked, what the evidence says, where the fun is and isn't.
3. A **decision brief** — the small number of genuine design questions,
   each with data and a recommendation, asked via AskUserQuestion and
   recorded in `docs/decisions.md`.

## The Contract: Who Decides What

This is the heart of the skill. Violating it defeats its purpose.

| Kind of question | Who decides | Your job |
|------------------|-------------|----------|
| **Design/taste**: system identity, what's fun, pacing *intent* ("should a crossing feel like a week or a day?"), what to cut, narrative tone | **The user.** Never decide silently, never bury it in an implementation PR | Frame the question with data, 2-4 options with tradeoffs, and a recommendation. Ask via AskUserQuestion |
| **Balance tuning within stated intent**: which constant moves, by how much, to hit a pacing target the user already set | You propose, user approves direction | Concrete old→new values with simulator evidence, not "make it faster" |
| **Implementation**: file structure, algorithms, test strategy | **You.** Don't ask | Just do it well |

The test for "is this a design decision?": would two good designers plausibly
choose differently, and would players feel the difference? If yes, it goes in
the decision brief. If only engineers would notice, decide it yourself.

Batch design questions — max 4 per round, most load-bearing first. A trickle
of one-off questions is worse than one well-framed brief.

## The Loop

Run whichever phases the request calls for. "Where are we with Act 2?" = phases
1-4. "Is the Deep fun?" = phases 2-3. A full iteration after landing changes =
all five. Phase 4 (decisions) only happens when there are genuine open
questions — don't manufacture them.

### Phase 0: Scope

Name the system(s) under iteration. Default to what's actively being built
(check recent `git log --oneline -20` and open branches). A "system" is dossier-
sized: the Voyage, the Colony ferry loop, the Deep, the Loom — not "combat"
and not "one constant".

### Phase 1: Dissect — build or refresh the dossier

Read the system's source, its `CLAUDE.md`, and its design docs in
`docs/plans/` (grep by system name — designs and plans are dated files there).
For a large system, fan out parallel Explore agents (one per subsystem/file
cluster) and synthesize; for a module-sized system, read directly.

Write `docs/dossiers/<system>.md` with these sections:

```markdown
# <System> — Design Dossier
> Last refreshed: <date> @ <short sha> | Sources: <module paths, key docs>

## The Player's Experience
The system as the player lives it, not as the code organizes it. Walk the
timeline: first contact → learning → the core loop at steady state → the
arc's end. Note the *cadence*: what happens every second / minute / session /
week. This section is prose, and it is the most important one.

## Design Intent
What the design docs say this system is FOR — its identity, its intended
pacing, the feeling it's meant to produce. Cite the plan docs. If intent was
never written down, say so; that's usually the first decision-brief question.

## Mechanics & Constants
The moving parts, with actual values cited as `file.rs:line`. Tables over
prose. Include the derived numbers players feel (e.g. "maiden voyage ≈ 2 real
weeks"), not just raw constants.

## Interrelations
What feeds this system, what it feeds, what gates it, what it gates —
currencies in/out, unlock edges, shared state. A small diagram (ASCII or
mermaid) plus one line per edge on WHY it exists. Flag dangling edges:
resources with no sink, gates nothing points at, systems that should talk
but don't.

## Balance Evidence
Latest simulator/play findings (see Phase 2), dated. Intent vs measured.

## Fun Assessment
Latest rubric scores (see Phase 3), dated, with the evidence one line each.

## Open Questions & Decision History
Open design questions not yet put to the designer; links to resolved ones
in docs/decisions.md.
```

**Refresh mode** (dossier already exists): `git log --oneline <last-refreshed-sha>..HEAD -- <system paths>`
and update only what changed. The chat briefing leads with the delta — "since
you last looked: X landed, Y's numbers moved, Z is newly undecided" — because
*continuously understanding a moving system* is the point. Never rewrite a
dossier from scratch when a refresh will do; the diff history of the dossier
is itself a record of the system's evolution.

### Phase 2: Evidence — balance and pacing

Claims about balance come from running things, not from reading constants.
Pick the instruments that match the system:

| System | Instruments |
|--------|-------------|
| Voyage / Colony (Act 2) | `cargo run --release --bin voyage_simulator -- --runs 5 --strategy all` (structural promises + crossing pacing); constants tables in `src/vessel/CLAUDE.md` |
| The Deep | `cargo run --bin deep_simulator -- --hours 24 --seed 1 --strategy <rush/farm/balanced/infrastructure>` across strategies |
| Whole-game progression | `cargo run --release --bin simulator -- --check-progression`; for real questions invoke the **balance-sim** skill (9-agent strategy×seed matrix) |
| Feel / moment-to-moment | **drive-game** skill against mkstate fixtures; for Act 2 set `QUEST_ACT2=1` (and `QUEST_VOYAGE_TIME_SCALE` to compress wall-clock) |

Then compare **measured vs intent** — intent from the Design Intent section,
measurement from the runs. Report both numbers side by side ("intent: ~19
crossings / ~3 months; measured at current constants: 26 / 4.2 months").
A gap is either a tuning task (intent stands, constants move — you propose
the change) or an intent question (the target itself is in doubt — decision
brief).

Simulated strategies are injected approximations — when a result looks wrong,
first check whether it's a simulator-strategy artifact before calling it a
balance problem (balance-sim's report format separates these; do the same).

### Phase 3: Fun audit

"Is the fun there?" is answerable with a rubric, not a vibe. These seven
heuristics are *why Act 1 works* — score the system 1-5 on each, with one
line of evidence per score, and name the Act 1 mechanism it's benchmarked
against:

1. **Visible next goal at every timescale.** Act 1 always shows something
   seconds away (kill), minutes away (boss at 10 kills), hours away (zone),
   days away (prestige, discovery). Is the system's "next thing" always on
   screen or one keypress away?
2. **Wall → reset → power cadence.** Act 1's core loop: hit a wall, reset
   with permanent gain, steamroll old walls. Does this system have its own
   version, and does the post-reset power feel *earned and felt*?
3. **Discovery cadence.** Act 1 drips new systems (challenges ~2h, Haven at
   P10+, Soulforge P15+, the Deep, the Loom). Does this system keep revealing
   new nouns at a sustainable rate, or does it front-load then flatline?
4. **Cross-system braiding.** Stormbreaker ties fishing + Haven + prestige
   into one quest. Does this system pull on other systems and get pulled on,
   or is it an island? (Check the dossier's Interrelations for dangling edges.)
5. **Decision density vs automation.** Idle-game rhythm: meaningful choices
   at comfortable intervals, automation between them. Too dense = a chore;
   too sparse = a screensaver. Where does this system sit at steady state,
   and does the density *change appropriately* over its arc (e.g. the maiden
   voyage is hands-on, ferry runs are hands-off — is that ramp deliberate)?
6. **Anticipation instruments.** Act 1 telegraphs what's coming (boss
   counters, discovery whispers, ticker). Does this system let the player
   *look forward* to things, or do they just happen?
7. **Stakes and texture.** Deaths, losses, weather, all-or-nothing burns —
   does anything here make the player *feel* something, and is permanent
   loss authored rather than random (the `mark_lost()` covenant)?

Two rules for honest scoring:
- **Play it before scoring it.** At minimum drive the real game to the
  system's screens (drive-game skill) and read what a player actually sees.
  A rubric filled from source code alone scores the design doc, not the game.
- **Low scores are findings, not failures.** A 2 on discovery cadence with
  evidence is exactly what the designer needs; don't grade-inflate.

Where the system deliberately breaks an Act 1 pattern (Act 2's wall-clock
crossing is nothing like the tick loop), say so — the rubric flags the
*departure* so the designer can confirm it's intentional, not to force
conformance.

### Phase 4: Decision brief

Collect the genuine design questions surfaced by phases 1-3. For each:

```markdown
### <Question, one line, ends with ?>
- **Why now**: what triggered this (gap found, fork in the road, evidence)
- **Data**: the 2-3 numbers/observations that matter
- **Options**: 2-4, each with the tradeoff in one line
- **Recommendation**: which one and why, in two sentences
```

Present the briefs in chat, then ask via AskUserQuestion (recommendation as
the first option, labeled "(Recommended)"). If there are more than 4
questions, ask the 4 most load-bearing and keep the rest in the dossier's
Open Questions section for the next round.

### Phase 5: Record and iterate

- Append each resolved decision to `docs/decisions.md` in its existing house
  style: system-level `##` heading, the options considered (table if numeric),
  **Decision** line with rationale. This log is why future sessions don't
  re-litigate settled questions — check it *before* drafting a brief, too.
- Turn approved directions into implementation work (normal dev workflow —
  worktrees, `make check`, the verification table in CLAUDE.md).
- After changes land, re-run the affected slice of phases 1-3 and refresh the
  dossier. The loop closes when measured ≈ intent and the rubric holds.

## Writing Guidelines

- **Ground everything.** Constants cite `file.rs:line`; pacing claims cite a
  simulator run; feel claims cite a played screen. "It seems slow" is not
  evidence.
- **Player-eye first.** Every briefing leads with what the *player*
  experiences, then descends into mechanisms. The designer is dissecting a
  game, not a codebase.
- **Deltas over dumps.** In refresh mode, what changed matters more than
  what is. Don't re-narrate the stable parts.
- **Recommend, don't hedge.** Every decision brief has a recommendation.
  "It depends" is analysis left unfinished.
- **Respect the kill-switch.** Act 2 work happens behind `ACT2_ENABLED =
  false`; preview with `QUEST_ACT2=1`, never flip the constant as part of
  this loop.

## When to Use

- Starting or resuming work on a system's design (Act 2 especially) — run a
  refresh so the designer is current before anything gets built.
- After landing a meaningful gameplay change — re-run evidence + fun phases.
- When the designer asks any form of "how is X shaping up / is it fun / what
  changed / walk me through it".
- Before enabling a dark-shipped system — full loop as the go/no-go review.

## When NOT to Use

- Pure implementation tasks with settled design — just build and verify.
- Whole-game balance regression checks — that's the **balance-sim** skill.
- UI polish verification — that's the **drive-game** skill.
