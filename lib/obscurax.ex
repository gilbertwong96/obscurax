defmodule Obscurax do
  @moduledoc """
  Elixir binding for the [obscura](https://github.com/h4ckf0r0day/obscura) headless
  browser engine, built with Rustler.

  Each page runs on its own OS thread with a dedicated V8 isolate, so multiple
  pages execute JavaScript in true parallel without blocking the BEAM.

  ## Quick start

      {:ok, browser} = Obscurax.Browser.new()
      {:ok, page} = Obscurax.Browser.new_page(browser)
      :ok = Obscurax.Page.goto(page, "https://example.com")
      {:ok, title} = Obscurax.Page.evaluate(page, "document.title")
      Obscurax.Page.close(page)

  ## Concurrency

  Pages from the same browser run in parallel — V8 evaluation in one page
  never blocks another:

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
  """

  defdelegate new(opts \\ []), to: Obscurax.Browser
  defdelegate new_page(browser), to: Obscurax.Browser
  defdelegate cookies(browser), to: Obscurax.Browser

  @doc """
  Reply to an intercepted request with a decision.

  This is the NIF-level primitive behind `Obscurax.Page.enable_interception/2`.
  Use it when you drive interception yourself: call the Nif module's
  `page_enable_interception/2` with the pid that should receive
  `{:obscurax_intercept, ref, req}`, then answer each request here.

      Nif.page_enable_interception(page.ref, self())

      receive do
        {:obscurax_intercept, ref, %{url: url}} ->
          Obscurax.reply_intercept(page, ref, :continue)
      end

  Because obscura also intercepts the main document request, a process that
  awaits `Obscurax.Page.goto/3` cannot also answer these messages — the
  navigation waits for a reply the caller will only send after it returns. Use
  `Obscurax.Page.enable_interception/2`, which answers from its own process.
  """
  @spec reply_intercept(Obscurax.Page.t(), reference(), term()) ::
          :ok | {:error, Obscurax.Error.t()}
  def reply_intercept(%Obscurax.Page{ref: ref}, intercept_id, decision) do
    Obscurax.Nif.reply_intercept(ref, intercept_id, decision)
  end
end
