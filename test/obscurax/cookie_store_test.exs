defmodule Obscurax.CookieStoreTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore}

  setup do
    {:ok, browser} = Browser.new()
    store = Browser.cookies(browser)
    {:ok, store: store}
  end

  test "set and get_all cookies", %{store: store} do
    :ok = CookieStore.set(store, "session=abc; Path=/", "https://example.com")
    assert {:ok, cookies} = CookieStore.get_all(store)
    assert Enum.any?(cookies, &(&1.name == "session"))
  end

  test "get_for_url returns cookies for a domain", %{store: store} do
    :ok = CookieStore.set(store, "token=xyz; Path=/", "https://example.com")
    assert {:ok, cookies} = CookieStore.get_for_url(store, "https://example.com")
    assert Enum.any?(cookies, &(&1.name == "token"))
  end
end
