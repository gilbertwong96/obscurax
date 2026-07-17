defmodule Obscurax.CallbackTest do
  use ExUnit.Case, async: true

  alias Obscurax.Nif

  setup do
    {:ok, browser} = Nif.browser_new(%{})
    {:ok, page} = Nif.browser_new_page(browser, self())
    on_exit(fn -> Nif.page_close(page) end)
    {:ok, page: page}
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

  test "on_request fires a message for page requests", %{page: page} do
    :ok = Nif.page_on_request(page, 1, self())

    await_goto(page, "https://example.com")

    assert_receive {:obscurax_request, 1, %{url: url, method: "GET"}}, 5_000
    assert url =~ "example.com"
  end

  test "enable_interception intercepts JS fetch and reply_intercept resolves it",
       %{page: page} do
    :ok = Nif.page_enable_interception(page, self())

    await_goto(page, "https://example.com")

    {:ok, _} =
      Nif.page_evaluate(page, "fetch('/').then(r => r.status).catch(e => 'err:' + e)")

    assert_receive {:obscurax_intercept, ref, %{url: url}}, 5_000
    assert url =~ "example.com"

    :ok = Nif.reply_intercept(page, ref, :continue)
  end

  test "on_response fires a message for page responses", %{page: page} do
    :ok = Nif.page_on_response(page, 1, self())

    await_goto(page, "https://example.com")

    assert_receive {:obscurax_response, 1, %{url: url, status: status}}, 5_000
    assert url =~ "example.com"
    assert is_integer(status)
  end
end
