defmodule Obscurax.ErrorTest do
  use ExUnit.Case, async: true

  alias Obscurax.Error

  test "exception/1 builds a struct" do
    err = Error.exception(kind: :timeout, message: "timed out", context: %{ms: 100})
    assert %Error{kind: :timeout, message: "timed out", context: %{ms: 100}} = err
  end

  test "message/1 formats kind and message" do
    err = Error.exception(kind: :navigation, message: "network error", context: %{})
    assert Error.message(err) == "[navigation] network error"
  end

  test "can be raised and rescued" do
    assert_raise Error, "[js_eval] SyntaxError", fn ->
      raise Error.exception(kind: :js_eval, message: "SyntaxError", context: %{})
    end
  end
end
