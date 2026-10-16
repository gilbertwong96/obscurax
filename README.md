# Obscurax

[![CI](https://github.com/gilbertwong96/obscurax/actions/workflows/ci.yml/badge.svg)](https://github.com/gilbertwong96/obscurax/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/gilbertwong96/obscurax/graph/badge.svg)](https://codecov.io/gh/gilbertwong96/obscurax)
[![Hex.pm](https://img.shields.io/hexpm/v/obscurax.svg)](https://hex.pm/packages/obscurax)
[![Elixir](https://img.shields.io/badge/elixir-1.18+-purple.svg)](https://elixir-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Elixir binding for the [obscura](https://github.com/h4ckf0r0day/obscura) headless browser engine, built with [Rustler](https://github.com/rusterlium/rustler). The engine is pinned to obscura **v0.2.4** (deno_core 0.412, V8 150.4.0).

Obscurax drives a real JavaScript engine from Elixir — but not the way a WebDriver binding does. There is no Chrome to install, no driver process to supervise, and no Node or Python in the loop: the engine is a Rust shared library shipped as a precompiled NIF, and every page gets its own OS thread and V8 isolate so pages execute JavaScript genuinely in parallel.

## At a glance

| | |
|---|---|
| Engine | obscura v0.2.4 — Rust headless browser with V8 150.4.0 embedded |
| Runtime dependencies | `rustler_precompiled` (that's it) |
| External browser or driver | none |
| Elixir / OTP | 1.18+ / OTP 26+ |
| Precompiled targets | 4 (macOS arm64, Linux arm64, Linux x86-64, Windows x86-64) |
| Parallelism model | one OS thread + one V8 isolate per page |

## Why obscurax

- **Pages run in parallel, not in a queue.** Each `Obscurax.Page` owns an OS thread with a dedicated tokio `current_thread` runtime and its own V8 isolate. Evaluating JavaScript in one page never blocks another page, and never serialises behind a single shared browser process.
- **It never blocks the BEAM.** V8 evaluation runs on dirty CPU schedulers, so a slow page cannot stall unrelated processes, and each page keeps its own HTTP connection pool.
- **Nothing external to install or babysit.** No `chromedriver`, no Chrome/Chromium download, no `selenium`/`geckodriver`, no Node, no Python. The engine arrives inside the NIF.
- **Far smaller and faster than headless Chrome.** There is no browser process to launch or to pay for: obscura's own benchmarks put the engine at 30 MB of memory against Chrome's 200+ MB, and page loads at 51–85 ms against ~500–800 ms. See [Performance and footprint](#performance-and-footprint).
- **No Rust toolchain required.** `rustler_precompiled` fetches a SHA-256-verified binary for your platform at compile time. Rust is only needed to build from source.
- **Erlang-shaped API.** Every function returns `{:ok, value}` or `{:error, %Obscurax.Error{}}`; bang variants raise. You get predictable return values instead of a driver protocol.
- **Structured errors you can pattern-match.** `%Obscurax.Error{kind: :element_not_found, message: "...", context: %{selector: "h1"}}` — `kind` is one of `:navigation`, `:js_eval`, `:timeout`, `:element_not_found`, `:no_page`, `:page_closed`, `:internal`, and `context` carries operation-specific details.
- **Request interception is built in.** Pause, inspect, rewrite, mock, or block requests from Elixir with `enable_interception/1` and `reply_intercept/3`.
- **Stealth mode** (opt-in) for detection-sensitive targets, plus **local file access** (opt-in) for testing HTML on disk.

## Performance and footprint

The engine runs in-process: there is no browser to launch, no driver in between, and no IPC between your Elixir code and V8. obscura benchmarks itself against headless Chrome, and obscurax embeds that same engine:

| Metric | obscura | Headless Chrome |
|---|---|---|
| Memory | **30 MB** | 200+ MB |
| Binary size | **~70 MiB** | 300+ MB |
| Page load | **85 ms** | ~500 ms |
| Startup | **Instant** | ~2 s |

Page load by page type:

| Page | obscura | Chrome |
|---|---|---|
| Static HTML | **51 ms** | ~500 ms |
| JS + XHR + fetch | **84 ms** | ~800 ms |
| Dynamic scripts | **78 ms** | ~700 ms |

Embedding the engine rather than driving a browser has two consequences worth knowing:

- There is no browser startup to amortise, supervise, or leak — a page is an OS thread in your node, not a subprocess.
- The engine and V8 share the BEAM's address space, so their memory is part of what your node already reports rather than a second process alongside it.

These are obscura's published figures, from the suite at [h4ckf0r0day/obscura-benchmark](https://github.com/h4ckf0r0day/obscura-benchmark) — WPT conformance, an obstacle course, a real-world corpus, and vs-Chrome speed. They were not re-measured for obscurax.

## Installation

Add `obscurax` to your `mix.exs`:

```elixir
def deps do
  [
    {:obscurax, "~> 0.2.0"}
  ]
end
```

### Build requirements

- **Elixir 1.18+** / **OTP 26+** — a `.mise.toml` pins the toolchain used for development (Elixir 1.20.4 / OTP 29.1.2)
- **Rust 1.91+** ([rustup](https://rustup.rs)) — only needed for `OBSCURAX_BUILD=true` or local development. A `rust-toolchain.toml` is included so rustup automatically installs the correct version.
- **C++ compiler** — clang/gcc on Linux, Xcode Command Line Tools on macOS, MSVC on Windows (required when building from source)

By default, Obscurax downloads a precompiled NIF at compile time — no Rust toolchain required. The precompiled binaries are hosted on [GitHub Releases](https://github.com/gilbertwong96/obscurax/releases) and verified with SHA-256 checksums.

| Precompiled target | Platform |
|---|---|
| `aarch64-apple-darwin` | macOS on Apple silicon |
| `aarch64-unknown-linux-gnu` | Linux on arm64 |
| `x86_64-unknown-linux-gnu` | Linux on x86-64 |
| `x86_64-pc-windows-msvc` | Windows on x86-64 |

To force building from source instead (e.g. for development or unsupported targets), set `OBSCURAX_BUILD=true`:

```bash
OBSCURAX_BUILD=true mix compile
```

The first source build takes ~5 minutes because V8 builds from source. Subsequent builds are fast — the V8 artifact is cached by Cargo.

Rustler compiles the Rust crate into a native shared library (`obscurax.so` on Linux/macOS, `obscurax.dll` on Windows) and places it in `priv/native/`.

## Quick start

```elixir
{:ok, browser} = Obscurax.Browser.new()
{:ok, page} = Obscurax.Browser.new_page(browser)
:ok = Obscurax.Page.goto(page, "https://example.com")
{:ok, title} = Obscurax.Page.evaluate(page, "document.title")
IO.puts(title)
Obscurax.Page.close(page)
```

Bang variants raise `Obscurax.Error` on failure:

```elixir
browser = Obscurax.Browser.new!()
page = Obscurax.Browser.new_page!(browser)
Obscurax.Page.goto!(page, "https://example.com")
"Example Domain" = Obscurax.Page.evaluate!(page, "document.title")
```

Mock a network response without leaving Elixir:

```elixir
:ok = Obscurax.Page.enable_interception(page)

receive do
  {:obscurax_intercept, ref, %{url: url}} ->
    :ok = Obscurax.reply_intercept(page, ref, {:fulfill, 200, %{}, ~s({"ok":true})})
end
```

## Features

### Browser and pages

- `Obscurax.Browser.new/1` — `:stealth`, `:proxy`, `:user_agent`, `:storage_dir`, `:allow_file_access`
- `Obscurax.Browser.new_page/1` — one OS thread and V8 isolate per page
- `Obscurax.Browser.cookies/1` — the browser-wide cookie store

### Navigation and JavaScript

- `Obscurax.Page.goto/3`, `goto!/3` — navigate, with an optional `:timeout` (default 30s)
- `Obscurax.Page.url/1`, `content/1` — final URL and serialised DOM
- `Obscurax.Page.evaluate/2`, `evaluate!/2` — run V8 JavaScript and get the value back
- `Obscurax.Page.add_preload_script/2` — queue a script that runs before the page's own scripts on the next navigation
- `Obscurax.Page.settle/2` — drive the page's event loop so timers, microtasks, and in-flight fetches settle; returns when idle or after the given deadline

### Elements

- `query_selector/2`, `wait_for_selector/3` — find nodes by CSS selector
- `element_text/2`, `element_attribute/3`, `element_click/2` — read and interact with a matched node

### Network observability

- `on_request/2` and `on_response/2` — every request/response is delivered to a callback process
- `enable_interception/1` + `reply_intercept/3` — resolve each intercepted request with `:continue`, `{:fulfill, status, headers, body}` to mock it, or `{:fail, reason}` to block it

### Cookies

- `Obscurax.Browser.cookies/1` returns a `CookieStore` with `set/3`, `get_all/1`, `get_for_url/2`, `save/2`, and `load/2`
- `Browser.new(storage_dir: dir)` persists the cookie jar across runs

## Concurrency

Pages from the same browser run in true parallel — V8 evaluation in one page never blocks another:

```elixir
{:ok, browser} = Obscurax.Browser.new()

urls = ["https://example.com", "https://example.org", "https://example.net"]

tasks =
  for url <- urls do
    Task.async(fn ->
      {:ok, page} = Obscurax.Browser.new_page(browser)
      :ok = Obscurax.Page.goto(page, url)
      {:ok, html} = Obscurax.Page.content(page)
      Obscurax.Page.close(page)
      html
    end)
  end

results = Task.await_many(tasks, 30_000)
```

## How it works

- **One OS thread per page.** Each page carries its own V8 isolate and its own tokio `current_thread` runtime, which is what makes JavaScript execution across pages genuinely parallel rather than interleaved. A `current_thread` runtime is required, not a preference: `deno_unsync` keeps V8's `!Send` isolate futures pinned to one thread by relying on a single-threaded runtime.
- **Dirty CPU NIFs.** JavaScript evaluation runs on dirty CPU schedulers so it never occupies a normal BEAM scheduler.
- **Per-page connection pool.** Because page runtimes are short-lived, each page builds its own HTTP client from an isolated copy of the browser context; the cookie jar and robots cache stay browser-wide. A pool shared across pages would let closing one page drop a runtime that still backed a sibling page's connection.
- **The library is large on purpose.** V8 and the engine are statically linked into the shared library, so the precompiled artifact is around 70 MB. The trade-off is that there is nothing else to install, and no external process to keep alive.

## FAQ

**Does Obscurax need Chrome, Chromium, or a WebDriver?**
No. It binds directly to obscura, a browser engine written in Rust with V8 embedded. There is no Chrome download, no `chromedriver`, no `geckodriver`, and no Node or Python.

**Do I need Rust installed to use it?**
No. `rustler_precompiled` downloads a SHA-256-verified binary for your platform at compile time. Rust 1.91+ and a C++ compiler are only needed with `OBSCURAX_BUILD=true`.

**Can multiple pages really run JavaScript at the same time?**
Yes. Each page has its own OS thread and its own V8 isolate, and JavaScript evaluation runs on dirty CPU schedulers, so pages do not queue behind each other or behind the BEAM.

**Does JavaScript evaluation block my GenServers?**
No. Evaluation runs on dirty CPU schedulers, and each page owns a separate runtime, so unrelated BEAM processes keep running.

**How do I mock or block a request?**
Call `Obscurax.Page.enable_interception/1`, receive `{:obscurax_intercept, ref, req}`, then call `Obscurax.reply_intercept/3` with `:continue`, `{:fulfill, status, headers, body}`, or `{:fail, reason}`.

**How are errors reported?**
Non-bang functions return `{:error, %Obscurax.Error{}}` with a `kind`, a `message`, and a `context` map; bang variants raise the same struct.

**Can I navigate to local HTML files?**
Yes, with `Obscurax.Browser.new(allow_file_access: true)`. The gate is off by default because a `file://` URL is a local filesystem read; a page can never navigate itself from a web origin into `file://` regardless of this option.

**What do I need to change if I upgrade?**
See [CHANGELOG.md](CHANGELOG.md). `0.2.0` moved `file://` navigation behind the `:allow_file_access` opt-in and fixed `Browser.new/1` options that were previously ignored.

## Current limitations

- The engine is not Chromium. Service workers, native media, and some Web APIs are incomplete, and long-tail rendering behaviour differs from Chrome.
- IndexedDB can report successful writes without retaining data between transactions.
- Response bodies are bounded and buffered — there is no incremental network streaming.
- There are no precompiled binaries for musl, riscv64, or `windows-gnu` (rusty_v8 publishes no builds for them), nor for Intel macOS. Those targets build from source with `OBSCURAX_BUILD=true`.

## License

MIT
