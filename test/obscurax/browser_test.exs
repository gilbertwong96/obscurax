defmodule Obscurax.BrowserTest do
  use ExUnit.Case, async: true

  alias Obscurax.Nif

  test "browser_new returns a resource" do
    assert {:ok, browser} = Nif.browser_new(%{})
    assert is_reference(browser)
  end

  test "browser_new accepts stealth and proxy options" do
    assert {:ok, browser} = Nif.browser_new(%{stealth: true, proxy: "socks5://127.0.0.1:1080"})
    assert is_reference(browser)
  end

  test "browser_new_page returns a page resource" do
    {:ok, browser} = Nif.browser_new(%{})
    assert {:ok, page} = Nif.browser_new_page(browser, self())
    assert is_reference(page)
  end

  test "browser_cookies returns a cookie store resource" do
    {:ok, browser} = Nif.browser_new(%{})
    assert {:ok, cookies} = Nif.browser_cookies(browser)
    assert is_reference(cookies)
  end
end
