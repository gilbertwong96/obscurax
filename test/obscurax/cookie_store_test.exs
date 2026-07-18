defmodule Obscurax.CookieStoreTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore}

  setup do
    {:ok, browser} = Browser.new()
    store = Browser.cookies(browser)
    {:ok, store: store}
  end

  test "set/3 sets a cookie", %{store: store} do
    assert :ok = CookieStore.set(store, "session=abc; Path=/", "https://example.com")
  end

  test "get_all/1 returns all cookies", %{store: store} do
    :ok = CookieStore.set(store, "session=abc; Path=/", "https://example.com")
    assert {:ok, cookies} = CookieStore.get_all(store)
    assert Enum.any?(cookies, &(&1.name == "session"))
  end

  test "get_all/1 returns empty list when no cookies", %{store: store} do
    assert {:ok, []} = CookieStore.get_all(store)
  end

  test "get_for_url/2 returns cookies for a domain", %{store: store} do
    :ok = CookieStore.set(store, "token=xyz; Path=/", "https://example.com")
    assert {:ok, cookies} = CookieStore.get_for_url(store, "https://example.com")
    assert Enum.any?(cookies, &(&1.name == "token"))
  end

  test "get_for_url/2 returns empty list for unknown domain", %{store: store} do
    assert {:ok, []} = CookieStore.get_for_url(store, "https://nonexistent.test")
  end

  test "save/2 writes cookies to a file", %{store: store} do
    :ok = CookieStore.set(store, "persist=1; Path=/", "https://example.com")
    path = Path.join(System.tmp_dir!(), "obscurax_cookies_#{System.unique_integer()}.json")
    on_exit(fn -> File.rm(path) end)
    assert :ok = CookieStore.save(store, path)
    assert File.exists?(path)
  end

  test "load/2 loads cookies from a file", %{store: store} do
    path = Path.join(System.tmp_dir!(), "obscurax_cookies_#{System.unique_integer()}.json")
    on_exit(fn -> File.rm(path) end)
    :ok = CookieStore.set(store, "loaded=1; Path=/", "https://example.com")
    :ok = CookieStore.save(store, path)
    assert {:ok, count} = CookieStore.load(store, path)
    assert count >= 1
  end

  # ── Error paths ───────────────────────────────────────────

  test "set/3 returns error for invalid URL", %{store: store} do
    assert {:error, %Obscurax.Error{}} = CookieStore.set(store, "session=abc", "not-a-url")
  end

  test "get_for_url/2 returns error for invalid URL", %{store: store} do
    assert {:error, %Obscurax.Error{}} = CookieStore.get_for_url(store, "not-a-url")
  end
end
