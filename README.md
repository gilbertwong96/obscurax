# Obscurax

Elixir binding for the [obscura](https://github.com/h4ckf0r0day/obscura) headless browser engine, built with [Rustler](https://github.com/rusterlium/rustler).

Obscurax wraps the full obscura API — browser launch, page navigation, V8 JavaScript evaluation, element operations, request/response observers, and request interception — behind a synchronous-feeling Elixir API. Each page runs on its own OS thread with a dedicated V8 isolate, so multiple pages execute JavaScript in true parallel.

## Installation

The package is not on Hex yet. Add it as a git dependency:

```elixir
def deps do
  [
    {:obscurax, git: "https://github.com/gilbertwong96/obscurax.git", branch: "main"}
  ]
end
```

### Build requirements

- **Rust 1.91+** ([rustup](https://rustup.rs)) — rustler 0.38 MSRV
- **C++ compiler** — clang/gcc on Linux, Xcode Command Line Tools on macOS (required for the V8 build)
- **git** — to fetch obscura as a Cargo dependency

The first `mix compile` takes ~5 minutes because V8 builds from source. Subsequent builds are fast — the V8 artifact is cached by Cargo.

> **Note:** obscurax currently depends on a [fork of obscura](https://github.com/gilbertwong96/obscura/tree/expose-node-id) (`expose-node-id` branch) that exposes `Element::node_id()` for element operations. This will switch back to upstream once the PR is merged.

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

## Features

- **Full obscura API** — Browser, Page, Element operations, CookieStore
- **Synchronous-feeling Elixir API** — async navigation and V8 evaluation under the hood, clean `{:ok, result}` / `{:error, %Obscurax.Error{}}` returns
- **Dirty CPU scheduler** — V8 evaluation runs on dirty CPU schedulers so it never blocks the BEAM
- **Passive observers** — `on_request/2` and `on_response/2` fire messages into a callback process for every network event
- **Request interception** — pause, inspect, modify, mock, or block requests via `enable_interception/1` and `reply_intercept/3`
- **Stealth mode** (opt-in) — `Obscurax.Browser.new(stealth: true)` to reduce detection

## Concurrency

Each `Obscurax.Page` runs on its own OS thread with a dedicated `current_thread` tokio runtime and V8 isolate. Pages from the same browser (or different browsers) run in true parallel — V8 evaluation in one page never blocks another:

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

## License

MIT
