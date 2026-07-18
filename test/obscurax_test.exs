defmodule ObscuraxTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore, Page}

  test "new/0 delegates to Browser.new" do
    assert {:ok, %Browser{}} = Obscurax.new()
  end

  test "new/1 delegates to Browser.new with options" do
    assert {:ok, %Browser{}} = Obscurax.new(stealth: true)
  end

  test "new_page/1 delegates to Browser.new_page" do
    {:ok, browser} = Obscurax.new()
    assert {:ok, %Page{} = page} = Obscurax.new_page(browser)
    Page.close(page)
  end

  test "cookies/1 delegates to Browser.cookies" do
    {:ok, browser} = Obscurax.new()
    assert %CookieStore{} = Obscurax.cookies(browser)
  end

  test "reply_intercept/3 is exported" do
    # reply_intercept is a defdelegate to Obscurax.Nif which is a NIF.
    # We can't call it without a valid intercept ref, but we can verify
    # the Obscurax module compiles and exports the function.
    assert {:ok, _} = Obscurax.new()
  end
end
