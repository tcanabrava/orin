# Introduction

This book is a contributor-facing description of Harmonicon's **system
architecture** — how the codebase is put together, how its major
subsystems talk to each other, and *why* they were built the way they
were. It is not a player's guide (that's `docs/book/`, the mdBook under
the repository's `docs/` directory) and it is not a line-by-line API
reference (that's what doc comments and `cargo doc` are for). It sits
between those two: the level of detail you'd want before making a
non-trivial change to the engine, or before reviewing someone else's
non-trivial change to it.

## Who this is for

Anyone who wants to work on Harmonicon's Rust codebase, at any level of
familiarity with Bevy or with this project specifically:

- A **new contributor** orienting themselves before their first change,
  who needs to know which of the ~20 top-level modules owns the thing
  they want to touch, and what invariants they need to not break.
- A **returning contributor** who worked on one subsystem months ago and
  needs a refresher on how it fits into the rest of the game before
  touching it again.
- **A reviewer** checking whether a pull request respects the
  architecture's existing boundaries (dependency direction, where pure
  logic vs. ECS systems belong, which resource owns which piece of
  state) rather than working around them.

## How this book is organized

Each chapter covers one architectural concern, roughly ordered from the
most foundational (how the app is built out of Bevy plugins, how
top-level state is organized) to the most specific (how one particular
feature — the Song Editor, Jam Session, Lessons — is put together
internally). Diagrams are [PlantUML](https://plantuml.com/), rendered
inline by [`mdbook-plantuml`](https://github.com/sytsereitsma/mdbook-plantuml)
at build time — see [this book's own `README.md`](../../README.md) (in
`contributing/`) for how to build it locally, since that needs a
PlantUML installation this book's own dependency tree doesn't pull in
for you.

Every chapter tries to explain **design rationale**, not just structure:
where there was a real alternative and a reason one was chosen over the
other, that reason is written down. Where the codebase's own extensive
doc comments (see `CLAUDE.md` at the repository root — Harmonicon's own
"guidance for Claude Code" file, itself a dense architectural reference)
already carry that reasoning, this book draws on it directly, but
organized by *concept* rather than by the chronological order features
were added in.

## What this book is not

- **Not a tutorial for Bevy itself.** It assumes working familiarity with
  Bevy's ECS (entities, components, resources, systems, states, plugins)
  and won't re-explain those primitives from scratch — see
  [Bevy's own documentation](https://bevyengine.org/learn/) for that.
- **Not exhaustive API documentation.** Function signatures and struct
  fields change; this book describes shapes and relationships that are
  meant to stay stable across many such changes. When you need the exact
  current signature of something, read the code — every module this book
  references is named so you can find it.
- **Not a substitute for `CLAUDE.md`.** That file is deliberately
  exhaustive, chronologically organized, and optimized for an AI coding
  agent to load in full on every session; it is the single most
  information-dense description of "how does X actually work" in this
  repository, and it is kept rigorously up to date as a hard project
  rule. This book is a different cut through much of the same knowledge:
  organized by concept for a human reading it start-to-finish or
  jumping to one chapter, with diagrams, and with more room to explain
  *why* at a level `CLAUDE.md`'s bullet-point style doesn't always leave
  space for.

## Keeping this book current

Architecture documentation rots the moment it stops being read as part of
the normal course of making changes. If you land a change that moves a
type between modules, introduces a new top-level subsystem, or reverses
a dependency direction this book describes, update the relevant chapter
in the same change — the same discipline `CLAUDE.md` and the player's
guide already hold themselves to (see that file's own instructions: keep
planning docs current, prune what's no longer true rather than letting it
accumulate as stale history).
