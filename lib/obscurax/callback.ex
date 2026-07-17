defmodule Obscurax.Callback do
  @moduledoc """
  GenServer that owns a user callback and dispatches incoming obscura
  callback messages (passive observers) to it.
  """

  use GenServer

  @type kind :: :request | :response

  def start_link(page, kind, fun) when kind in [:request, :response] do
    GenServer.start_link(__MODULE__, {page, kind, fun})
  end

  @impl true
  def init({page, kind, fun}) do
    callback_id = System.unique_integer([:positive])
    callback_pid = self()

    case kind do
      :request -> :ok = Obscurax.Nif.page_on_request(page, callback_id, callback_pid)
      :response -> :ok = Obscurax.Nif.page_on_response(page, callback_id, callback_pid)
    end

    {:ok, %{page: page, kind: kind, fun: fun, callback_id: callback_id}}
  end

  @impl true
  def handle_info({:obscurax_request, _callback_id, req}, %{kind: :request, fun: fun} = state) do
    fun.(req)
    {:noreply, state}
  end

  def handle_info({:obscurax_response, _callback_id, resp}, %{kind: :response, fun: fun} = state) do
    fun.(resp)
    {:noreply, state}
  end

  def handle_info(_msg, state) do
    {:noreply, state}
  end

  @impl true
  def terminate(_reason, %{page: page, kind: kind, callback_id: callback_id}) do
    case kind do
      :request -> Obscurax.Nif.page_off_request(page, callback_id)
      :response -> Obscurax.Nif.page_off_response(page, callback_id)
    end

    :ok
  end
end
