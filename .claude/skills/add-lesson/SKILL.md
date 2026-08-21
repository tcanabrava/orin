---
name: add-lesson
description: Author a new Harmonicon lesson (assets/lessons/<unit>/<lesson>/). Use when adding curriculum content, a new pass criterion, or a chart-backed drill. Covers the manifest schema, chart layout, three-locale keys, prerequisite integrity and what is honestly scoreable.
---

# Authoring a lesson

Curriculum design lives in `docs/lessons_plan.md`; engine detail in
`crates/harmonicon-song/CLAUDE.md`. This is the mechanical checklist.

## First: is it actually scoreable?

The engine judges *pitch*, plus amplitude/timing patterns for a few
modifiers. It cannot tell puckering from tongue blocking. Classify the
lesson honestly:

- **Scored** — an existing primitive judges it directly.
- **Scored via proxy** — the technique is unverifiable but a musical
  outcome requiring it is not (tongue blocking → the octave splits it
  enables).
- **Instructional only** — text/diagram, passed with Mark-as-Done.

Don't build scoring machinery for the third category.

## Layout

```
assets/lessons/<NN>_<unit>/<NN>_<lesson>/
  lesson.json
  song/chart.harpchart      # only for chart-backed lessons
```

The `NN_` prefixes are the ordering mechanism — the scan sorts by
directory name.

## lesson.json

Validated against `assets/lesson_schema.dtd.json`, which is
`additionalProperties: false`, so a typo fails loudly.

```json
{
  "id": "stable-id",
  "unit": "scales",
  "title_key": "lesson-stable-id-title",
  "body_key": "lesson-stable-id-body",
  "chart": "song/chart.harpchart",
  "prerequisites": ["earlier-lesson-id"],
  "pass_criteria": { "type": "scale-adherence", "threshold": 0.8 }
}
```

- **`id` is permanent** — it is the profile key and other lessons'
  prerequisite reference. Never rename a shipped one.
- `title_key`/`body_key` are Fluent **keys**, never display text. Add them
  in all three locales (see the `add-locale-string` skill).
- Optional: `progression`, `scale`, `diagram`, `position_cycle` — all
  schema-enforced enums/booleans seeded into jam resources on Start.

## Pass criteria

| Type | Judged from | Where |
|---|---|---|
| `accuracy` | overall weighted accuracy | results screen |
| `technique` | one `SongStats` bucket (`bend`, `wah-wah`, `clean-attack`, …) | results screen |
| `scale-adherence` | `ImprovStats::adherence` | **open jam**, pause-menu "Finish Lesson" |
| `chord-tone-adherence` | `ImprovStats::chord_tone_adherence` | same |
| `phrase-discipline` | `ImprovStats::phrase_discipline` | same |

The last three route into `GameplayMode::JamSession` and never reach the
results screen — there is no score for an open jam. They need a chart
anyway (a single marker `TrackItem` satisfies the schema's `minItems: 1`
and gives the progress bar a length).

## Prerequisites

`lessons::is_unlocked` gates on them, and `tests/asset_layout.rs` checks
every referenced id exists. Under `--features dev` everything shows
unlocked so you can jump straight to a lesson while iterating — that
bypass is in the menu, not in `is_unlocked`, which stays fully tested.

## Verify

```bash
cargo test --workspace
```

`tests/asset_layout.rs` validates the schema, the chart, file
completeness, prerequisite integrity **and** that every locale key exists
in all three languages. If it passes, the lesson is wired correctly.
