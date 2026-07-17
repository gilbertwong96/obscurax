defmodule Obscurax.PageTest do
  use ExUnit.Case, async: true

  alias Obscurax.Nif

  setup do
    {:ok, browser} = Nif.browser_new(%{})
    {:ok, page} = Nif.browser_new_page(browser, self())
    on_exit(fn -> Nif.page_close(page) end)
    {:ok, page: page}
  end

  test "goto navigates and url returns the url", %{page: page} do
    url = "https://example.com"
    id = Nif.page_goto(page, url)

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
      {:obscurax_result, ^id, :error, msg} -> flunk("goto failed: #{msg}")
    after
      30_000 -> flunk("goto timed out")
    end

    assert {:ok, current} = Nif.page_url(page)
    assert String.starts_with?(current, "https://example.com")
  end

  test "close returns ok", %{page: page} do
    assert :ok = Nif.page_close(page)
  end
end
