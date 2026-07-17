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

  defp await_goto(page, url) do
    id = Nif.page_goto(page, url)

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
      {:obscurax_result, ^id, :error, msg} -> flunk("goto failed: #{msg}")
    after
      30_000 -> flunk("goto timed out")
    end
  end

  test "evaluate returns JS values", %{page: page} do
    await_goto(page, "https://example.com")

    assert {:ok, "Example Domain"} = Nif.page_evaluate(page, "document.title")
    assert {:ok, 2} = Nif.page_evaluate(page, "1 + 1")
  end

  test "content returns the page HTML", %{page: page} do
    await_goto(page, "https://example.com")

    assert {:ok, html} = Nif.page_content(page)
    assert html =~ "Example Domain"
  end

  test "query_selector returns a node id", %{page: page} do
    await_goto(page, "https://example.com")

    assert {:ok, node_id} = Nif.page_query_selector(page, "h1")
    assert is_integer(node_id)
  end

  test "element_text returns text content", %{page: page} do
    await_goto(page, "https://example.com")

    {:ok, node_id} = Nif.page_query_selector(page, "h1")
    assert {:ok, "Example Domain"} = Nif.page_element_text(page, node_id)
  end

  test "element_attribute returns attribute values", %{page: page} do
    await_goto(page, "https://example.com")

    {:ok, node_id} = Nif.page_query_selector(page, "a")
    assert {:ok, _href} = Nif.page_element_attribute(page, node_id, "href")
  end

  test "element_click clicks an element", %{page: page} do
    await_goto(page, "https://example.com")

    {:ok, node_id} = Nif.page_query_selector(page, "a")
    assert :ok = Nif.page_element_click(page, node_id)
  end

  test "wait_for_selector times out for missing element", %{page: page} do
    await_goto(page, "https://example.com")

    assert {:error, %Obscurax.Error{kind: :timeout}} =
             Nif.page_wait_for_selector(page, ".nonexistent", 100)
  end

  test "settle returns ok", %{page: page} do
    await_goto(page, "https://example.com")

    assert :ok = Nif.page_settle(page, 100)
  end

  test "add_preload_script returns ok", %{page: page} do
    assert :ok = Nif.page_add_preload_script(page, "window.__flag = true")
  end
end
