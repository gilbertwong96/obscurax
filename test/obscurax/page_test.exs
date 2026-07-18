defmodule Obscurax.PageTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Error, Page}

  setup do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    on_exit(fn -> Page.close(page) end)
    {:ok, page: page}
  end

  defp await_goto(page, url) do
    assert :ok = Page.goto(page, url)
  end

  # ── Happy path ────────────────────────────────────────────

  test "goto/2 navigates and url/1 returns the url", %{page: page} do
    :ok = Page.goto(page, "https://example.com")
    assert {:ok, url} = Page.url(page)
    assert String.starts_with?(url, "https://example.com")
  end

  test "goto!/2 returns :ok on success", %{page: page} do
    assert :ok = Page.goto!(page, "https://example.com")
  end

  test "evaluate/2 returns JS values", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:ok, "Example Domain"} = Page.evaluate(page, "document.title")
    assert {:ok, 2} = Page.evaluate(page, "1 + 1")
  end

  test "evaluate!/2 returns the value on success", %{page: page} do
    await_goto(page, "https://example.com")
    assert "Example Domain" = Page.evaluate!(page, "document.title")
  end

  test "content/1 returns the page HTML", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:ok, html} = Page.content(page)
    assert html =~ "Example Domain"
  end

  test "query_selector/2 returns a node id", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:ok, node_id} = Page.query_selector(page, "h1")
    assert is_integer(node_id)
  end

  test "query_selector/2 returns nil for missing selector", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:ok, nil} = Page.query_selector(page, ".nonexistent")
  end

  test "wait_for_selector/3 returns a node id", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:ok, node_id} = Page.wait_for_selector(page, "h1", 5_000)
    assert is_integer(node_id)
  end

  test "wait_for_selector!/3 returns a node id on success", %{page: page} do
    await_goto(page, "https://example.com")
    assert is_integer(Page.wait_for_selector!(page, "h1", 5_000))
  end

  test "wait_for_selector/3 times out with Obscurax.Error", %{page: page} do
    await_goto(page, "https://example.com")
    assert {:error, %Error{kind: :timeout}} = Page.wait_for_selector(page, ".nonexistent", 100)
  end

  test "wait_for_selector!/3 raises Obscurax.Error on timeout", %{page: page} do
    await_goto(page, "https://example.com")

    assert_raise Error, fn ->
      Page.wait_for_selector!(page, ".nonexistent", 100)
    end
  end

  test "element_text/2 returns text content", %{page: page} do
    await_goto(page, "https://example.com")
    {:ok, node_id} = Page.query_selector(page, "h1")
    assert {:ok, "Example Domain"} = Page.element_text(page, node_id)
  end

  test "element_attribute/3 returns attribute values", %{page: page} do
    await_goto(page, "https://example.com")
    {:ok, node_id} = Page.query_selector(page, "a")
    assert {:ok, _href} = Page.element_attribute(page, node_id, "href")
  end

  test "element_attribute/3 returns nil for missing attribute", %{page: page} do
    await_goto(page, "https://example.com")
    {:ok, node_id} = Page.query_selector(page, "h1")
    assert Page.element_attribute(page, node_id, "data-nonexistent") == {:ok, nil}
  end

  test "element_click/2 clicks an element", %{page: page} do
    await_goto(page, "https://example.com")
    {:ok, node_id} = Page.query_selector(page, "a")
    assert :ok = Page.element_click(page, node_id)
  end

  test "settle/2 returns ok", %{page: page} do
    await_goto(page, "https://example.com")
    assert :ok = Page.settle(page, 100)
  end

  test "add_preload_script/2 returns ok", %{page: page} do
    assert :ok = Page.add_preload_script(page, "window.__flag = true")
  end

  test "on_request/2 spawns a callback process", %{page: page} do
    parent = self()

    assert {:ok, pid} =
             Page.on_request(page, fn req ->
               send(parent, {:request, req})
             end)

    assert is_pid(pid)
    Page.goto(page, "https://example.com")
    assert_receive {:request, %{url: url}}, 5_000
    assert url =~ "example.com"
  end

  test "on_response/2 spawns a callback process", %{page: page} do
    parent = self()

    assert {:ok, pid} =
             Page.on_response(page, fn resp ->
               send(parent, {:response, resp})
             end)

    assert is_pid(pid)
    Page.goto(page, "https://example.com")
    assert_receive {:response, %{url: url}}, 5_000
    assert url =~ "example.com"
  end

  test "enable_interception/1 returns ok", %{page: page} do
    assert :ok = Page.enable_interception(page)
  end

  test "close/1 returns ok", %{page: page} do
    assert :ok = Page.close(page)
  end

  # ── Error paths ───────────────────────────────────────────

  test "goto/2 returns navigation error for bad URL", %{page: page} do
    assert {:error, %Error{kind: :navigation}} =
             Page.goto(page, "https://nonexistent.invalid.tld")
  end

  test "goto/2 returns timeout error with very short timeout", %{page: page} do
    assert {:error, %Error{kind: :timeout}} = Page.goto(page, "https://example.com", timeout: 1)
  end

  test "goto!/2 raises on navigation error", %{page: page} do
    assert_raise Error, fn ->
      Page.goto!(page, "https://nonexistent.invalid.tld")
    end
  end

  test "evaluate/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{kind: :page_closed}} = Page.evaluate(page, "1")
  end

  test "evaluate!/2 raises on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert_raise Error, fn -> Page.evaluate!(page, "1") end
  end

  test "url/1 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.url(page)
  end

  test "content/1 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.content(page)
  end

  test "query_selector/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.query_selector(page, "h1")
  end

  test "wait_for_selector/3 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.wait_for_selector(page, "h1", 100)
  end

  test "element_text/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.element_text(page, 1)
  end

  test "element_attribute/3 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.element_attribute(page, 1, "href")
  end

  test "element_click/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.element_click(page, 1)
  end

  test "settle/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.settle(page, 100)
  end

  test "add_preload_script/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.add_preload_script(page, "1")
  end

  test "enable_interception/1 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.enable_interception(page)
  end

  test "on_request/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.on_request(page, fn _ -> :ok end)
  end

  test "on_response/2 returns error on closed page" do
    {:ok, browser} = Obscurax.Browser.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)
    Page.close(page)

    assert {:error, %Error{}} = Page.on_response(page, fn _ -> :ok end)
  end
end
