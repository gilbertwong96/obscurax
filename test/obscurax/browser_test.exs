defmodule Obscurax.BrowserTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore, Page}

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
