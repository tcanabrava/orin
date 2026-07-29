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
plantuml_cmd = "java -jar /path/to/plantuml.jar"
```

Then, from this directory:

```bash
mdbook build   # writes static HTML to contributing/book/ (gitignored)
mdbook serve   # local dev server with live rebuild, http://localhost:3000
```

## Keeping it current

If you land a change that moves a type between modules, introduces a
new top-level subsystem, or reverses a dependency direction this book
describes, update the relevant chapter in the same change — see the
book's own [Introduction](src/introduction.md) for why this matters and
how it relates to `CLAUDE.md` and `docs/book/`.
