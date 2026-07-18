defmodule Obscurax.ErrorTest do
  use ExUnit.Case, async: true

  alias Obscurax.Error

  test "exception/1 builds a struct" do
    err = Error.exception(kind: :timeout, message: "timed out", context: %{selector: ".btn"})

    assert %Error{kind: :timeout, message: "timed out", context: %{selector: ".btn"}} = err
  end

  test "message/1 formats kind and message" do
    err =
      Error.exception(kind: :navigation, message: "network error", context: %{url: "https://x"})

    assert Error.message(err) == "[navigation] network error"
  end

  test "can be raised and rescued" do
    assert_raise Error, "[js_eval] SyntaxError", fn ->
      raise Error.exception(kind: :js_eval, message: "SyntaxError", context: %{})
    end
  end

  test "default context is an empty map" do
    err = Error.exception(kind: :no_page, message: "no page")
    assert err.context == %{}
  end

  test "wait_for_selector timeout error carries selector and timeout_ms" do
    {:ok, browser} = Obscurax.Nif.browser_new(%{})
    {:ok, page} = Obscurax.Nif.browser_new_page(browser, self())

    id = Obscurax.Nif.page_goto(page, "https://example.com")

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
    after
      30_000 -> flunk("timeout")
    end

    assert {:error, %Error{kind: :timeout, context: ctx}} =
             Obscurax.Nif.page_wait_for_selector(page, ".nonexistent", 100)

    assert ctx.selector == ".nonexistent"
    assert ctx.timeout_ms == 100

    Obscurax.Nif.page_close(page)
  end
end
