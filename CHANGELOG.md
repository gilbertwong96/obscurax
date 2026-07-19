# Changelog

## v0.1.0 (2026-07-19)

Initial release.

### Added

- **Browser API** — `Obscurax.Browser.new/1`, `new_page/1`, `cookies/1` with stealth mode support
- **Page API** — navigation (`goto/2`, `url/1`), V8 JavaScript evaluation (`evaluate/2`), DOM content (`content/1`), element operations (`query_selector/2`, `wait_for_selector/3`, `element_text/2`, `element_attribute/3`, `element_click/2`), preload scripts (`add_preload_script/2`), and page settle (`settle/2`)
- **Passive observers** — `on_request/2` and `on_response/2` fire messages into a callback GenServer for every network event
- **Request interception** — `enable_interception/1` and `reply_intercept/3` to pause, inspect, modify, mock, or block requests
- **CookieStore** — full cookie CRUD (`set/3`, `get_all/1`, `get_for_url/2`, `save/2`, `load/2`)
- **Structured errors** — `Obscurax.Error` exception with `kind`, `message`, and `context` struct typed by error kind
- **Bang variants** — all public functions have `!` variants that raise `Obscurax.Error`
- **Per-page OS thread** — each page runs on a dedicated OS thread with its own tokio `current_thread` runtime and V8 isolate for true parallelism
- **Dirty CPU schedulers** — V8 evaluation runs on dirty CPU NIFs so it never blocks the BEAM
- **Precompiled NIF distribution** — `rustler_precompiled` downloads prebuilt binaries at compile time; no Rust toolchain required for end users
- **Callback GenServer** — `Obscurax.Callback` wraps user functions with fault tolerance (survives raises)

### Build & CI

- Rustler 0.38 + obscura (deno_core 0.408, v8 149.4.0)
- NIF 2.17 only (Elixir 1.18+ / OTP 26+)
- 4 precompiled targets: `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`
- CI on ubuntu-24.04/OTP 29 and macos-14/OTP 27
- `mix ci` alias: compile, format check, credo --strict, deps audit, xref graph, dialyzer, ex_dna, reach dead-code/smells, test --cover (80% threshold)
- `cargo fmt --check` + `cargo clippy -- -D warnings` enforced

### Known Limitations

- Tests hit real network (`https://example.com`) — no local test server
- `goto/2` timeout leaks messages — no cancellation mechanism yet
- `browser_new_page` returns `Ok` before V8 init completes — failures surface as `page_closed` later
- No musl/riscv64/windows-gnu support — rusty_v8 has no prebuilt binaries for these targets
- No Intel macOS (`x86_64-apple-darwin`) precompiled binary — build from source with `OBSCURAX_BUILD=true`
- Currently depends on obscura fork (`gilbertwong96/obscura@upgrade-deno-core`) — will switch to upstream once [obscura PR #445](https://github.com/h4ckf0r0day/obscura/pull/445) is merged
