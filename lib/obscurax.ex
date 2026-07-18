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

  Call this inside your `{:obscurax_intercept, ref, req}` receive handler:

      :ok = Obscurax.Page.enable_interception(page)
      receive do
        {:obscurax_intercept, ref, %{url: url}} ->
          Obscurax.reply_intercept(page, ref, :continue)
      end
  """
  @spec reply_intercept(Obscurax.Page.t(), reference(), term()) ::
          :ok | {:error, Obscurax.Error.t()}
  def reply_intercept(%Obscurax.Page{ref: ref}, intercept_id, decision) do
    Obscurax.Nif.reply_intercept(ref, intercept_id, decision)
  end
end
