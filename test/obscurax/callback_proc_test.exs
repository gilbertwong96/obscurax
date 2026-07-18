defmodule Obscurax.CallbackProcTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Callback, Nif}

  setup do
    {:ok, browser} = Nif.browser_new(%{})
    {:ok, page} = Nif.browser_new_page(browser, self())
    on_exit(fn -> Nif.page_close(page) end)
    {:ok, page: page}
  end

  test "callback process receives and dispatches on_request", %{page: page} do
    parent = self()

    {:ok, _cb} =
      Callback.start_link(page, :request, fn req ->
        send(parent, {:request, req})
      end)

    id = Nif.page_goto(page, "https://example.com")

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
    after
      30_000 -> flunk("timeout")
    end

    assert_receive {:request, %{url: url}}, 5_000
    assert url =~ "example.com"
  end

  test "callback process receives and dispatches on_response", %{page: page} do
    parent = self()

    {:ok, _cb} =
      Callback.start_link(page, :response, fn resp ->
        send(parent, {:response, resp})
      end)

    id = Nif.page_goto(page, "https://example.com")

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
    after
      30_000 -> flunk("timeout")
    end

    assert_receive {:response, %{url: url}}, 5_000
    assert url =~ "example.com"
  end

  test "terminate deregisters the callback", %{page: page} do
    parent = self()

    {:ok, cb} =
      Callback.start_link(page, :request, fn req ->
        send(parent, {:request, req})
      end)

    ref = Process.monitor(cb)
    Process.unlink(cb)
    GenServer.stop(cb, :shutdown)

    assert_receive {:DOWN, ^ref, :process, ^cb, :shutdown}, 5_000

    id = Nif.page_goto(page, "https://example.com")

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
    after
      30_000 -> flunk("timeout")
    end

    refute_received {:request, _}
  end

  test "handle_info ignores unexpected messages", %{page: page} do
    {:ok, cb} = Callback.start_link(page, :request, fn _ -> :ok end)
    send(cb, {:unexpected, :message})
    # Process should still be alive — the catch-all handle_info ignores it
    assert Process.alive?(cb)
    GenServer.stop(cb, :shutdown)
  end

  test "start_link with invalid kind raises ArgumentError" do
    invalid_kind = String.to_atom("invalid")

    assert_raise FunctionClauseError, fn ->
      Callback.start_link(nil, invalid_kind, fn _ -> :ok end)
    end
  end

  test "callback survives a raising user function", %{page: page} do
    {:ok, cb} =
      Callback.start_link(page, :request, fn _req ->
        raise "boom"
      end)

    # The callback process should survive the raise (try/rescue in handle_info)
    id = Nif.page_goto(page, "https://example.com")

    receive do
      {:obscurax_result, ^id, :ok} -> :ok
    after
      30_000 -> flunk("timeout")
    end

    # Process should still be alive
    assert Process.alive?(cb)
    GenServer.stop(cb, :shutdown)
  end
end
