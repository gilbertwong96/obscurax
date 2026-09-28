# AGENTS.md

## Project Overview

Obscurax is an Elixir binding for the [obscura](https://github.com/h4ckf0r0day/obscura) headless browser engine, built with Rustler. Each page runs on its own OS thread with a dedicated V8 isolate, enabling true parallel JavaScript execution without blocking the BEAM.

## Architecture

- **Elixir layer** (`lib/obscurax/`) — synchronous-feeling API wrapping async NIF calls
  - `browser.ex` — browser lifecycle (`new/1`, `new_page/1`, `cookies/1`)
  - `page.ex` — navigation, V8 eval, element ops, observers, interception
  - `callback.ex` — GenServer wrapping user callback functions for request/response observers
  - `cookie_store.ex` — cookie CRUD backed by obscura's `CookieJar`
  - `error.ex` — structured error type
- **Rust NIF layer** (`native/obscurax/src/`) — Rustler NIFs bridging to obscura
  - `page_thread.rs` — per-page OS thread with its own `current_thread` tokio runtime, V8 isolate and HTTP connection pool
  - `page.rs` — page operations (goto, evaluate, query, observers, interception)
  - `browser.rs` — browser launch and page management
  - `callback.rs` — message-passing bridge between Rust async and Elixir processes
  - `cookie.rs`, `error.rs`, `atoms.rs` — supporting modules
- **Precompiled NIFs** — `rustler_precompiled` downloads prebuilt binaries at compile time; no Rust toolchain required for end users

## Build & Development

### Prerequisites

- Elixir 1.18+
- Erlang/OTP 26+
- Rust 1.91+ (only for `OBSCURAX_BUILD=true` or local dev)
- C++ compiler (clang/gcc/MSVC)

### Common commands

```bash
# Install deps
mix deps.get

# Compile (downloads precompiled NIF by default)
mix compile

# Force build from source (needs Rust toolchain)
OBSCURAX_BUILD=true mix compile

# Run tests (hits real network — https://example.com)
mix test

# Full CI suite (format, credo, dialyzer, coverage, etc.)
mix ci

# Rust lint + format (run from native crate)
cd native/obscurax && cargo fmt --check && cargo clippy -- -D warnings
```

### CI

`mix ci` runs: compile, format check, credo --strict, deps audit, xref graph, dialyzer, ex_dna, reach dead-code/smells, test --cover (80% threshold).

GitHub Actions:
- `.github/workflows/ci.yml` — Elixir tests on ubuntu-24.04/OTP 29 and macos-14/OTP 27
- `.github/workflows/release.yml` — precompile NIF binaries for 4 targets, publish on tag push

## Supported NIF Targets

| Target | OS | Notes |
|--------|-----|-------|
| `aarch64-apple-darwin` | macOS ARM | Native build on macos-14 |
| `aarch64-unknown-linux-gnu` | Linux ARM | Cross-compiled, `cross-version: from-source` for glibc with `memfd_create` |
| `x86_64-unknown-linux-gnu` | Linux x86_64 | Native build on ubuntu-24.04 |
| `x86_64-pc-windows-msvc` | Windows | Native build on windows-2022 |

Not supported (no rusty_v8 prebuilt binaries): musl, riscv64, windows-gnu. Intel macOS (`x86_64-apple-darwin`) is dropped — users can build from source with `OBSCURAX_BUILD=true`.

## Key Design Decisions

- **NIF 2.17 only** — Elixir 1.18 requires OTP 26+ which has NIF 2.17. No need for older NIF versions.
- **`panic = "unwind"` in release profile** — obscura's V8 anti-panic protocol requires unwinding.
- **Dirty CPU schedulers** — V8 evaluation runs on dirty CPU NIFs so it never blocks BEAM schedulers.
- **Per-page OS thread** — each page gets its own OS thread with a dedicated tokio `current_thread` runtime and V8 isolate for true parallelism.
- **`current_thread` is required, not a preference** — `deno_unsync` masks V8's `!Send` isolate futures as `Send` and relies on a single-threaded runtime to keep them pinned to one thread. A `multi_thread` runtime trips its `debug_assert!` in debug builds and is silent unsoundness once the assert is compiled out in release.
- **Per-page connection pool** — hyper drives each pooled connection with a task on the runtime that created it, and page runtimes are short-lived, so a pool shared across pages lets `Page.close/1` kill a sibling page's connection (`runtime dropped the dispatch task`). Pages therefore build their context from `BrowserContext::isolated_copy` and get their own `ObscuraHttpClient`. The cookie jar and robots cache stay browser-wide.
- **Lower-level obscura crates, not the `obscura` facade** — the per-page context has to be built directly, and the facade's `Browser.context`/`Page.inner` are private. Uses `obscura-browser` + `obscura-net`.
- **Upstream obscura, pinned tag** — depends on `h4ckf0r0day/obscura@v0.2.3` (deno_core 0.412, v8 150.4.0). The `gilbertwong96/obscura@upgrade-deno-core` fork existed only to carry v8 past 149.4.0 for the Rusty_v8 `local-dynamic` TLS fix; upstream has since moved past that itself, so the fork is obsolete. Pinning an immutable tag (not a branch) also stops the dependency from drifting out from under `Cargo.lock`. A git dependency is required — the `obscura` crate on crates.io is an unrelated project.
- **v8 150.4.0** — includes [rusty_v8 PR #1911](https://github.com/denoland/rusty_v8/pull/1911) which switches from `initial-exec` to `local-dynamic` TLS model, fixing `R_X86_64_TPOFF32` relocation errors when linking V8 into a shared library on Linux.

## Code Style

### Elixir
- `mix format` enforced (CI fails on unformatted code)
- `~w()` sigil for word lists
- Bang variants (`!`) raise `Obscurax.Error`, non-bang return `{:ok, _}` / `{:error, _}`
- Suppress expected test logs with `ExUnit.CaptureLog.capture_log/1`

### Rust
- `cargo fmt --check` and `cargo clippy -- -D warnings` enforced
- NIF functions return `Result<T, rustler::Error>` for error propagation
- Use `serde` for complex types crossing the NIF boundary

## Testing

- Tests hit real network (`https://example.com`) — no local test server
- 80 tests, 80% coverage threshold enforced
- The `[error]` log from "callback survives a raising user function" test is suppressed via `capture_log`
- The Nif module is excluded from coverage (NIF stubs)

## File Layout

```
lib/obscurax/          Elixir API
lib/obscurax_nif.ex    NIF module (RustlerPrecompiled config)
native/obscurax/src/   Rust NIF source
native/obscurax/Cargo.toml
.github/workflows/     ci.yml, release.yml
test/                  ExUnit tests
checksum-*.exs         Precompiled NIF checksums
```
