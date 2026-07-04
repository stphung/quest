# docs/explorations/

**Pre-commitment thinking.** This is the scratch home for design exploration
that hasn't yet crystallized into an OpenSpec change — brainstorms, spikes,
"what should Act 3 even be" notes, option comparisons, open questions.

It exists because OpenSpec is *change-shaped*: every OpenSpec artifact belongs to
a shippable unit. Thinking that precedes any such unit needs somewhere to live
so it isn't homeless or forced into premature proposal ceremony.

## How it works

- Drop a dated note here: `YYYY-MM-DD-<topic>.md`. Keep it lightweight.
- Use `/opsx:explore` as the *mode* for this thinking; park written output here.
- When an exploration **graduates**, it becomes a change (`/opsx:propose`) whose
  `design.md` carries the thinking forward — or it becomes a section in a
  `docs/dossiers/<system>.md` if it's cross-cutting and long-lived.
- Prune freely. Unlike the archive (permanent) and dossiers (living), these
  notes are disposable once they've graduated or gone stale.

## Where things go (the map)

See [`../README.md`](../README.md) for the full "where does this doc go?"
decision guide. In short:

| Kind of thing | Home |
|---|---|
| What the game *does* now | `openspec/specs/` |
| Design + plan for one shippable change | that change's `design.md` / `tasks.md` (archived on ship) |
| Pre-commitment exploration | **here** (`docs/explorations/`) |
| Evolving per-system / world design | `docs/dossiers/` |
| A resolved decision + rationale | `docs/decisions.md` |
