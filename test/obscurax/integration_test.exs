defmodule Obscurax.IntegrationTest do
  use ExUnit.Case, async: true

  setup do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    on_exit(fn -> Obscurax.Page.close(page) end)
    {:ok, browser: browser, page: page}
  end

  test "full flow: goto, evaluate, content, query, click", %{page: page} do
    :ok = Obscurax.Page.goto(page, "https://example.com")
    assert {:ok, "Example Domain"} = Obscurax.Page.evaluate(page, "document.title")
    assert {:ok, html} = Obscurax.Page.content(page)
    assert html =~ "Example Domain"

    {:ok, node_id} = Obscurax.Page.query_selector(page, "h1")
    assert {:ok, "Example Domain"} = Obscurax.Page.element_text(page, node_id)
  end

  test "bang variants raise Obscurax.Error", %{page: page} do
    :ok = Obscurax.Page.goto(page, "https://example.com")

    assert_raise Obscurax.Error, fn ->
      Obscurax.Page.wait_for_selector!(page, ".nonexistent", 100)
    end
  end

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
    assert Enum.all?(titles, &(&1 == "Example Domain"))
  end
end
