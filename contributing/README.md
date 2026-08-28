# Harmonicon — System Architecture book

This is a contributor-facing mdBook describing Harmonicon's internal
system architecture: how the codebase is put together, how its major
subsystems talk to each other, and why. It's aimed at people working on
the Rust codebase, as opposed to `docs/book/`, which is the *player's*
guide.

## Building it

You need two things beyond a normal Rust toolchain: `mdbook` itself, and
a PlantUML renderer (for the diagrams, via the
[`mdbook-plantuml`](https://github.com/sytsereitsma/mdbook-plantuml)
preprocessor).

```bash
# mdbook
cargo install mdbook

# mdbook-plantuml (the preprocessor that turns ```plantuml code blocks
# into inline SVG diagrams at build time)
cargo install mdbook-plantuml
```

`mdbook-plantuml` needs an actual PlantUML installation to invoke. It
auto-detects either a `plantuml` executable on `PATH`, or `java -jar
plantuml.jar`. Install PlantUML itself with your platform's package
manager:

```bash
# Arch Linux
sudo pacman -S plantuml

# Debian/Ubuntu
sudo apt install plantuml

# macOS (Homebrew)
brew install plantuml
```

If you'd rather not install a system package, PlantUML also ships as a
plain jar that just needs a JRE — download it from
[plantuml.com/download](https://plantuml.com/download) and point
`book.toml` at it:

```toml
[preprocessor.plantuml]
plantuml-cmd = "java -jar /path/to/plantuml.jar"
```

**Graphviz is required too, and installing PlantUML may not bring it.**
PlantUML lays out everything except sequence diagrams by shelling out to
`dot`; Debian/Ubuntu make graphviz a *Recommends* of `plantuml`, so
`--no-install-recommends` (or a minimal container) leaves you without it.
The symptom is not an error — PlantUML exits 0 and renders an image whose
text reads "Cannot find Graphviz", citing a hardcoded `/opt/local/bin/dot`
that has nothing to do with your machine. `dot -V` is the quick check.

Note the **kebab-case** key. `mdbook-plantuml`'s config derives
`rename_all = "kebab-case"` with `#[serde(default)]`, so a snake_case
`plantuml_cmd` isn't rejected — it's silently ignored, and you get the
default back with no warning.

Then, from this directory:

```bash
mdbook build   # writes static HTML to contributing/book/ (gitignored)
mdbook serve   # local dev server with live rebuild, http://localhost:3000
```

`book.toml` sets `fail-on-error = true`, so a diagram that doesn't parse
fails the build instead of leaving a blank space on the page. That matters
because this book is published (see below) and a missing diagram is
indistinguishable from a chapter that never had one.

## Published

Both this book and the player's guide deploy to GitHub Pages on every push
to `main` that touches them — `.github/workflows/pages.yaml`. This one
lands at `/architecture/`, the player's guide at `/guide/`.

## Keeping it current

If you land a change that moves a type between modules, introduces a
new top-level subsystem, or reverses a dependency direction this book
describes, update the relevant chapter in the same change — see the
book's own [Introduction](src/introduction.md) for why this matters and
how it relates to `CLAUDE.md` and `docs/book/`.
