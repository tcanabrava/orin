# The Gameplay Clock

Every timing-sensitive system in scored gameplay — where a falling note
currently is, whether it's inside its hit window, what the HUD displays,
where the phrase/song-progress overlays are drawn — reads its notion of
"now" from exactly one place: `GameplayClock` (`gameplay/clock.rs`).
This chapter explains what that clock actually is, why it isn't simply
"the real-time elapsed since the song started," and the API design that
makes its one hard invariant difficult to violate by accident.

## Why not just use real elapsed time?

The naive approach — track `Instant::now() - song_start` and treat that
as the authoritative song position — breaks the moment the *audio*
itself doesn't advance in perfect lock-step with wall-clock time. In
practice it never does: audio decoders have startup latency, backends
occasionally hitch, and a `bevy_audio`/`rodio` sink's own reported
playback position is the one source of truth for "where is the actual
sound the player is hearing right now." A note judged against
wall-clock time while the *audio* is a few milliseconds ahead or behind
reads as mistimed even when the player's timing was perfect — the
judgment is comparing the player against the wrong reference.

So `GameplayClock` doesn't track wall-clock time directly; once music
starts, it tracks the **audio sink's own position**, with wall-clock
delta as a fallback for the periods where there's no sink to anchor to
at all (the pre-song countdown, Jam Session, before the sink has
produced its first position report).

## The anchoring algorithm

```plantuml
@startuml
title tick_clock's per-frame decision
start
:dt = Time::delta_secs_f64();
if (should_anchor_to_sink?\n(clock >= 0, music started,\nnot Jam Session, sink not empty)) then (yes)
  :audio_pos = sink.position();
  :drift = audio_pos - (clock + dt);
  if (|drift| > SNAP_THRESHOLD_SECS (0.5s)) then (yes)
    :clock = audio_pos;
    note right: A stall or seek, not\nordinary jitter — snap\noutright rather than\nslowly re-converging\nwhile notes visibly\ndesync for seconds.
  else (no)
    :clock = clock + dt + drift.clamp(±MAX_RATE_ADJUST * dt);
    note right: A proportional nudge\n(±0.5% speed), not a fixed\nstep — inaudible/invisible,\nand doesn't bias every\njudged offset by a constant\namount the way a fixed step\nper frame would.
  endif
else (no)
  :clock = clock + dt;
  note right: Free-run on frame delta:\ncountdown, pause, Jam\nSession, practice speed\nbelow 100%.
endif
stop
@enduml
```

Two constants make this concrete: `MAX_RATE_ADJUST` (0.5%) caps how much
the clock's *rate* — not a one-off jump — can deviate from real time
while gently correcting toward the sink, expressed as a rate rather than
a fixed per-frame step specifically so the correction doesn't bias every
judged hit offset by a constant amount for as long as it's active, and
so it doesn't over- or under-correct depending on the actual frame rate.
`SNAP_THRESHOLD_SECS` (0.5s) is the line between "ordinary jitter, worth
correcting gently" and "a real discontinuity" (a decoder stall, a
backend seek) that should be corrected immediately rather than converged
toward over several seconds of visibly-desynced notes.

**Jam Session deliberately never anchors.** There's no long, fixed-length
track to drift against in free play — Jam Session's music is either a
generated backing loop or a picked song played on repeat, and nothing
about that experience benefits from the sink-anchoring machinery.
`should_anchor_to_sink` excludes `GameplayMode::JamSession` explicitly.

## The encapsulation: why the inner value is private

`GameplayClock`'s inner `f64` is a private tuple field. The type exposes
exactly three ways to change it, and each one documents (or, for the
anchored case, actively enforces) the invariant that matters:

```plantuml
@startuml
title GameplayClock's write API
class GameplayClock {
  - f64
  + get() : f64
  + set_free(t: f64)
  + advance(dt: f64, audio_pos: Option<f64>)
  + rewind_to(t: f64, sink: Option<&AudioSink>)
}
note right of GameplayClock::set_free
  Anchoring guaranteed inactive:
  setup/countdown, Jam Session,
  Bending Trainer.
end note
note right of GameplayClock::advance
  tick_clock's own per-frame
  update — not a jump.
end note
note right of GameplayClock::rewind_to
  Jumps *and* seeks the sink in
  one call. What handle_loop_boundary
  uses — and what any future A–B
  looping UI or practice-speed
  feature must use too.
end note
@enduml
```

The invariant all three exist to protect: **anything that jumps the
clock must also seek the music sink, or suspend anchoring** — because if
it doesn't, the very next `tick_clock` pass sees the sink still sitting
at its old position, computes a large "drift," and drags the clock right
back toward where it just jumped *from*. This is a genuinely easy bug to
write by hand (assign the new time, forget the sink is now stale) and a
confusing one to debug (the symptom is "my rewind gets silently undone
one frame later," with no error or panic pointing at why) — which is
exactly the shape of bug a private field plus a small, invariant-carrying
API is good at ruling out at the type level rather than relying on every
future caller remembering a rule from a comment.

`rewind_to` is the one both existing callers that jump the clock already
use — `handle_loop_boundary` (an A–B loop or a chart's own `loop`
section reaching its end point) is the only one in the codebase today,
but the doc comment above is explicit that this is also the contract any
future practice-speed-change or manual seek feature must follow, not an
incidental detail of the current feature set.

## Reading the clock: the ordering invariant

The other half of the contract, enforced by convention rather than the
type system: **every system that *reads* the clock must be ordered
`.after(GameplayLogic)`** (the `SystemSet` that ticks it — see
[The Plugin Architecture](plugin-architecture.md)). Bevy's scheduler is
free to run unordered systems in any relative sequence, including
parallelized; a clock-reading system that isn't explicitly ordered after
the tick can read last frame's value on some frames and this frame's on
others, which manifests as visible note-movement stutter rather than a
crash — the kind of bug that's easy to introduce and easy to miss in
testing if it only shows up as an occasional single-frame jitter.

```plantuml
@startuml
title GameplayLogic ordering
skinparam componentStyle rectangle

package "GameplayLogic (chained, in this order)" {
  rectangle "tick_clock" as tick
  rectangle "handle_loop_boundary" as loop_boundary
  rectangle "track_current_bar" as bar
  rectangle "collect_pitches" as pitches
  rectangle "update_active_targets" as targets
  rectangle "score_notes" as score
  rectangle "update_score_display" as hud
  rectangle "detect_song_end" as song_end
  rectangle "animate_note_tails" as tails
}

rectangle "Every clock-reading overlay/renderer\n(note movement, phrase overlay, song-progress\nbar, metronome, harmonica overlay, ...)" as readers

tick -down-> loop_boundary
loop_boundary -down-> bar
bar -down-> pitches
pitches -down-> targets
targets -down-> score
score -down-> hud
hud -down-> song_end
song_end -down-> tails
tails -down-> readers : .after(GameplayLogic)
@enduml
```

## Practice speed and pausing

Two more free-running cases fall out of the same `should_anchor_to_sink`
predicate rather than needing special-case code: **practice speed below
100%** (real time-stretched audio isn't implemented, so the sink is
simply paused and the clock free-runs on `Time::delta` scaled by the
practice-speed factor instead — returning to 100% re-seeks the sink to
the clock's current position via `rewind_to` before resuming it, since
the sink sat still the whole time the clock kept moving), and **the
wait-for-note freeze** (`wait_freeze_overlay` — the chart holds at the
next unjudged note until the player plays it; `tick_clock` pauses the
sink and skips advancing the clock at all while a note is "due" under
that mode, and Jam Session never populates `SongNotes`, so the freeze
condition is always trivially false there regardless).
