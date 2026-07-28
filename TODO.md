# TODO

Open, actionable items only — once something lands, delete it from here
rather than annotating it done (git log and commit messages are the
historical record; see `CLAUDE.md`).

## Song editor

Found on a harmonica-player/audio/UX pass over the editor (2026-07-27);
see `CLAUDE.md`'s "Song editor: known gaps" bullet for the full detail
behind each. Roughly in priority order:

- [ ] **Manual note placement can't represent swing/triplet timing.** The
  grid snaps to straight 16ths only (`TICKS_PER_BEAT = 4`); there's no
  triplet or shuffle-aware subdivision to click onto, even though shuffle
  is this game's core blues feel elsewhere (`MetronomeFeel::Shuffle`).
  Only Record mode's live-mic capture (unquantized onsets) can currently
  land a note off the straight grid — hand-charting an authentic shuffle
  groove by clicking isn't possible. Lower priority than the items above;
  worth doing once there's an appetite for a genuine grid-resolution
  rework (`TICKS_PER_BEAT` is baked into a lot of tick-math elsewhere).

## Content

- [ ] **Only one bundled example artist** (`assets/songs/Example Artist`,
  three example songs used for 2D/3D/fallback testing). Ship a starter
  pack of public-domain blues heads/riffs across difficulties before wider
  release. **Deliberately not attempted unsupervised**: authoring
  rights-clear, well-judged chart content needs real musical judgment.
- [ ] **Lessons content, Unit 4 "jazz" (0.6).** Wave 2 (harmonica-basics
  extensions, bar-counting drills, the train trio, and the new Unit 3
  blues-vocabulary unit — licks via call-and-response, chord-tone/
  minor-blues/phrase-discipline improvisation) is fully shipped, and Unit
  4's own engine prerequisites (jazz chord-tone tables, the jazz-blues
  `Progression` variant) are now done too — see `docs/lessons_plan.md`.
  What's left is authoring the actual jazz unit content. Original
  arpeggio/vocabulary drills are the safe-to-author subset; actual
  jazz-standard repertoire needs the same rights judgment as the item
  above.
