defmodule Obscurax do
  @moduledoc """
  Elixir binding for the [obscura](https://github.com/h4ckf0r0day/obscura) headless
  browser engine.

  ## Quick start

      {:ok, browser} = Obscurax.Browser.new()
      {:ok, page} = Obscurax.Browser.new_page(browser)
      :ok = Obscurax.Page.goto(page, "https://example.com")
      {:ok, title} = Obscurax.Page.evaluate(page, "document.title")
      Obscurax.Page.close(page)
  """

  defdelegate new(opts \\ []), to: Obscurax.Browser
  defdelegate new_page(browser), to: Obscurax.Browser
  defdelegate cookies(browser), to: Obscurax.Browser
  defdelegate reply_intercept(page, ref, decision), to: Obscurax.Nif
end
