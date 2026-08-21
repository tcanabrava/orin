---
name: add-locale-string
description: Add or change a user-visible string in Harmonicon. Use whenever writing text a player will read — a button label, a status message, lesson body text — or when a build fails with a localization-enforcement error. Covers the three-locale parity rule and Fluent's argument syntax.
---

# Adding a user-visible string

`build.rs` fails the build on a raw string literal reaching a `Text`
constructor or a known label sink, so this is not optional.

## Steps

1. Add the key to **all three** locale files — parity is enforced by
   `localization::tests::locales_define_the_same_keys`:
   - `assets/locales/en-US/main/ui.ftl`
   - `assets/locales/pt-BR/main/ui.ftl`
   - `assets/locales/es-ES/main/ui.ftl`

   Keep it in the section its neighbours are in; the files are grouped by
   screen, and the test only checks parity, not placement.

2. Read it with `loc.msg("key")`, never a literal.

## Strings with variables

Use Fluent's own syntax and `msg_args`, not `format!`:

```ftl
jam-position-label = Position: {$position}
```

```rust
loc.msg_args("jam-position-label", &[("position", position.label().to_string())])
```

`msg_args` builds a real `FluentArgs` and resolves through Fluent's
`format_pattern`. It also strips the FSI/PDI bidi-isolation marks Fluent
wraps every argument in — those are meant for prose mixing scripts and
would otherwise leak invisible characters into rendered and logged text.
`msg` (no args) skips all of that.

## Gotchas

- A long literal wrapped onto its own line, or split by a `\` line
  continuation, is still caught — `build.rs` looks ahead. Don't try to
  smuggle one past it.
- Lesson manifests store Fluent **keys** (`title_key`/`body_key`), never
  display text. Authoring a lesson means adding its keys here by hand.
- The locale list is a fixed `LOCALES` const, not a directory scan — wasm's
  HTTP asset reader cannot enumerate a directory.
  `locales_const_matches_the_assets_directory` keeps the const honest.

## Verify

```bash
cargo test --workspace           # parity + the const-vs-directory check
cargo build                      # build.rs literal enforcement
```
