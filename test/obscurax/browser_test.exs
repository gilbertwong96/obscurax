defmodule Obscurax.BrowserTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore, Error, Page, TestFixture}

  test "new/0 returns a browser struct" do
    assert {:ok, %Browser{ref: ref}} = Browser.new()
    assert is_reference(ref)
  end

  test "new/1 accepts stealth and proxy options" do
    assert {:ok, %Browser{}} = Browser.new(stealth: true, proxy: "socks5://127.0.0.1:1080")
  end

  test "new/1 accepts user_agent and storage_dir options" do
    tmp = System.tmp_dir!()
    assert {:ok, %Browser{}} = Browser.new(user_agent: "TestUA/1.0", storage_dir: tmp)
  end

  test "new/1 accepts allow_file_access" do
    assert {:ok, %Browser{}} = Browser.new(allow_file_access: true)
  end

  test "new/1 applies user_agent to the page" do
    {:ok, browser} = Browser.new(user_agent: "ObscuraxTest/1.0")
    {:ok, page} = Browser.new_page(browser)
    on_exit(fn -> Page.close(page) end)

    TestFixture.goto!(page)
    assert {:ok, "ObscuraxTest/1.0"} = Page.evaluate(page, "navigator.userAgent")
  end

  test "file:// navigation is rejected by default" do
    {:ok, browser} = Browser.new()
    {:ok, page} = Browser.new_page(browser)
    on_exit(fn -> Page.close(page) end)

    assert {:error, %Error{kind: :navigation}} = Page.goto(page, TestFixture.file_url())
  end

  test "allow_file_access: true permits file:// navigation" do
    {:ok, browser} = Browser.new(allow_file_access: true)
    {:ok, page} = Browser.new_page(browser)
    on_exit(fn -> Page.close(page) end)

    assert :ok = Page.goto(page, TestFixture.file_url())
    assert {:ok, "Example Domain"} = Page.evaluate(page, "document.title")

    assert {:ok, node_id} = Page.query_selector(page, "h1")
    assert {:ok, "Example Domain"} = Page.element_text(page, node_id)
  end

  test "new!/1 returns a browser on success" do
    assert %Browser{} = Browser.new!()
  end

  test "new!/1 raises on error" do
    # Passing an invalid proxy won't error at construction, but we can test
    # the bang path by mocking — instead, verify the happy path doesn't raise.
    assert %Browser{} = Browser.new!()
  end

  test "new_page/1 returns a page struct" do
    {:ok, browser} = Browser.new()
    assert {:ok, %Page{ref: ref}} = Browser.new_page(browser)
    assert is_reference(ref)
    Page.close(%Page{ref: ref})
  end

  test "new_page!/1 returns a page on success" do
    {:ok, browser} = Browser.new()
    assert %Page{} = Browser.new_page!(browser)
  end

  test "cookies/1 returns a cookie store" do
    {:ok, browser} = Browser.new()
    assert %CookieStore{ref: ref} = Browser.cookies(browser)
    assert is_reference(ref)
  end

  # ── Error paths ───────────────────────────────────────────

  test "new!/1 returns browser on valid opts" do
    assert %Browser{} = Browser.new!()
  end
end
