defmodule Obscurax.IntegrationTest do
  use ExUnit.Case, async: true

  alias Obscurax.TestFixture

  setup do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    on_exit(fn -> Obscurax.Page.close(page) end)
    {:ok, browser: browser, page: page}
  end

  test "full flow: goto, evaluate, content, query, click", %{page: page} do
    :ok = TestFixture.goto!(page)
    assert {:ok, "Example Domain"} = Obscurax.Page.evaluate(page, "document.title")
    assert {:ok, html} = Obscurax.Page.content(page)
    assert html =~ "Example Domain"

    {:ok, node_id} = Obscurax.Page.query_selector(page, "h1")
    assert {:ok, "Example Domain"} = Obscurax.Page.element_text(page, node_id)
  end

  test "bang variants raise Obscurax.Error", %{page: page} do
    TestFixture.goto!(page)

    assert_raise Obscurax.Error, fn ->
      Obscurax.Page.wait_for_selector!(page, ".nonexistent", 100)
    end
  end

  # v0.2.5 scopes localStorage to the browser context, so pages of one browser
  # share it instead of each page getting an incognito store.
  test "localStorage is shared by pages of one browser", %{browser: browser, page: page} do
    :ok = Obscurax.Page.goto(page, "https://example.com")
    assert {:ok, _} = Obscurax.Page.evaluate(page, "localStorage.setItem('k', 'shared')")

    {:ok, other} = Obscurax.Browser.new_page(browser)
    on_exit(fn -> Obscurax.Page.close(other) end)
    :ok = Obscurax.Page.goto(other, "https://example.com")

    assert {:ok, "shared"} = Obscurax.Page.evaluate(other, "localStorage.getItem('k')")
  end

  # Navigates over the real network on purpose: this guards the per-page
  # connection pool, which only matters when pages fetch concurrently.
  test "concurrent pages from different processes", %{browser: browser} do
    tasks =
      for _ <- 1..3 do
        Task.async(fn ->
          {:ok, page} = Obscurax.Browser.new_page(browser)
          :ok = Obscurax.Page.goto(page, "https://example.com")
          {:ok, title} = Obscurax.Page.evaluate(page, "document.title")
          Obscurax.Page.close(page)
          title
        end)
      end

    titles = Task.await_many(tasks, 30_000)
    assert length(titles) == 3
    assert [first | rest] = titles
    assert first != ""
    assert Enum.all?(rest, &(&1 == first))
  end
end
