# Module CLAUDE.md Template

Guidelines for writing module-level CLAUDE.md files optimized for AI agent navigation.

## Template

```markdown
# Module Name

One-line purpose statement.

## Files

| File | Purpose |
|------|---------|
| `types.rs` | Core data structures |
| `logic.rs` | Business logic |
| ... | ... |

## Key Types

Brief descriptions of primary structs/enums with their most important fields.

## How It Works

Core mechanics, state transitions, algorithms. The "mental model" section.

## Integration Points

Which modules call into this one, and which this one calls out to.
Format: `module/file.rs` → `function_name()` — what it does

## Key Constants

Balance numbers, thresholds, rates that matter for understanding behavior.

## Adding / Extending

How to add a new X to this module (if applicable).
```

## Guidelines

- **Not rigid**: Modules can deviate where the template doesn't fit (e.g., challenges/ leads with its step-by-step checklist for adding new minigames)
- **File inventory is mandatory**: Every CLAUDE.md must list all files in the module with one-line purpose descriptions
- **Audience is AI agents**: Optimize for quick comprehension. Lead with purpose, then structure, then details
- **No duplication with root**: Module docs should be self-contained. The root CLAUDE.md links to modules but doesn't duplicate their content
- **Keep it concise**: Aim for 60-150 lines per module doc. Larger modules (core, deep) may need more
- **Integration Points matter**: AI agents need to know which modules interact, so always document cross-module calls
- **Key Constants section**: Only include constants that affect behavior understanding. Don't list every magic number
