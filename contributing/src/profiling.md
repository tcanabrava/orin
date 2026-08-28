# Profiling with Tracy

`cargo run --release --features trace_tracy`, with the Tracy UI
(<https://github.com/wolfpld/tracy>) already open and "Connect" clicked.

## How it hangs together

`trace_tracy` (`Cargo.toml`) just forwards to `bevy/trace_tracy`, which already wraps every ECS system
call in its own `info_span!("system", name = ..)` — most of what shows up
in Tracy needs no manual instrumentation at all. Two things this crate adds
on top:
- `main.rs`'s `LogPlugin` is feature-gated: the everyday filter
  (`"warn,bevy_render::camera=error"`) sets the *default* level below
  `info`, which silently drops every span (Bevy's own and ours) before any
  backend — Tracy included — ever sees them (see
  `LogPlugin::build_filter_layer`, which folds
  `filter`'s own bare directives over `level`). A `trace_tracy` build swaps
  in a filter with no bare-level directive below `info`, so the configured
  `Level::INFO` default actually holds.
- Manual spans cover the paths automatic per-system instrumentation can't
  reach — anything that isn't itself a system call. Two categories so far:
  - **Off the ECS schedule entirely:** the cpal capture callback
  (`audio_input::push_chunks`) runs on its own real-time thread; the only
  custom `AssetLoader` (`song::loader::SongChartLoader::load`) runs as a
  future on the AssetServer's IO task pool. Both get a manual span for
  the same reason — Bevy's per-system spans only wrap systems the
  schedule itself calls, so anything running elsewhere (another thread,
  another executor) is otherwise invisible no matter how expensive it
  is. A span held across an `.await` needs `tracing::Instrument` (via
  `bevy::log::tracing::Instrument`) rather than a plain `.entered()`
  guard — an `EnteredSpan` isn't `Send`, which the loader's returned
  future must be; `SongChartLoader::load` is a thin wrapper that
  instruments a `load_inner` for exactly this reason.
  - **A hot inner loop worth breaking out of its system's own total time:**
  `pipeline::process_audio`'s per-chunk work; `pitch_detect::analyze`'s
  FFT transform and per-algorithm dispatch; `build_nmf_dict` (the
  priciest one-off, rebuilt only when the NMF dictionary goes stale);
  `waveform::analyze_ogg_waveform`/`analyze_wav_waveform` (a whole-file
  decode — also called from the off-schedule asset loader above, so it
  carries both reasons at once).
  Add spans the same way for any other code that runs off the main
  schedule (more asset loaders, decode threads, the asset watcher — though
  `assets_management::watch`'s debouncer thread runs only
  `notify-debouncer-full`'s own code, nothing of ours, so there's nothing
  to instrument there) or burns real time inside a single system call.
