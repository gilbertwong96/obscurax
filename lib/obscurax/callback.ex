defmodule Obscurax.Callback do
  @moduledoc """
  GenServer that owns a user callback and dispatches incoming obscura
  callback messages (passive observers) to it.
  """

  use GenServer

  require Logger

  @type kind :: :request | :response

  def start_link(page, kind, fun) when kind in [:request, :response] do
    # Use start/3 (not start_link/3) so that an init failure (e.g. page closed)
    # returns {:error, reason} to the caller instead of crashing the caller.
    GenServer.start(__MODULE__, {page, kind, fun})
  end

  @impl true
  def init({page, kind, fun}) do
    callback_id = System.unique_integer([:positive])
    callback_pid = self()

    case register_callback(page, kind, callback_id, callback_pid) do
      :ok ->
        {:ok, %{page: page, kind: kind, fun: fun, callback_id: callback_id}}

      {:error, error} ->
        {:stop, error}
    end
  end

  defp register_callback(page, :request, callback_id, callback_pid) do
    case Obscurax.Nif.page_on_request(page, callback_id, callback_pid) do
      :ok -> :ok
      {:error, _} = error -> error
    end
  end

  defp register_callback(page, :response, callback_id, callback_pid) do
    case Obscurax.Nif.page_on_response(page, callback_id, callback_pid) do
      :ok -> :ok
      {:error, _} = error -> error
    end
  end

  @impl true
  def handle_info({:obscurax_request, _callback_id, req}, %{kind: :request, fun: fun} = state) do
    safe_dispatch(fun, req)
    {:noreply, state}
  end

  def handle_info({:obscurax_response, _callback_id, resp}, %{kind: :response, fun: fun} = state) do
    safe_dispatch(fun, resp)
    {:noreply, state}
  end

  def handle_info(_msg, state) do
    {:noreply, state}
  end

  defp safe_dispatch(fun, arg) do
    fun.(arg)
  rescue
    exception ->
      Logger.error(
        "Obscurax callback raised: #{Exception.format(:error, exception, __STACKTRACE__)}"
      )
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
