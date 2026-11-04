defmodule Obscurax.Interceptor do
  @moduledoc """
  Process that answers a page's intercepted requests with a handler function.

  obscura intercepts the main document request as well as subresources, so a
  page with interception enabled pauses *every* navigation until the request is
  answered. `Obscurax.Page.goto/3` blocks its caller while the page navigates,
  and a handler running in that same process could never reply — hence this
  process: `Obscurax.Page.enable_interception/2` starts one and the handler runs
  there, so the caller is free to navigate.

  The handler may block. Anything it raises is logged and the request is
  continued.
  """

  use GenServer

  require Logger

  alias Obscurax.Nif

  @type decision ::
          :continue
          | {:fulfill, non_neg_integer(), %{optional(String.t()) => String.t()}, String.t()}
          | {:fail, String.t()}

  # Unlinked, like `Obscurax.Callback`: a failed init (page already closed) must
  # return `{:error, reason}` instead of an exit signal that kills the caller.
  @spec start(reference(), (map() -> decision())) :: GenServer.on_start()
  def start(page_ref, handler) when is_function(handler, 1) do
    GenServer.start(__MODULE__, {page_ref, handler})
  end

  @impl true
  def init({page_ref, handler}) do
    case Nif.page_enable_interception(page_ref, self()) do
      :ok -> {:ok, %{page_ref: page_ref, handler: handler}}
      {:error, error} -> {:stop, error}
    end
  end

  @impl true
  def handle_info({:obscurax_intercept, intercept_id, request}, state) do
    Nif.reply_intercept(state.page_ref, intercept_id, decide(state.handler, request))
    {:noreply, state}
  end

  def handle_info(_msg, state), do: {:noreply, state}

  defp decide(handler, request) do
    case handler.(request) do
      :continue ->
        :continue

      {:fulfill, status, headers, body} = decision
      when is_integer(status) and status in 100..599 and is_map(headers) and is_binary(body) ->
        decision

      {:fail, reason} = decision when is_binary(reason) ->
        decision

      other ->
        Logger.error(
          "Obscurax interception handler returned #{inspect(other)}, " <>
            "expected :continue, {:fulfill, status, headers, body}, or {:fail, reason}; continuing"
        )

        :continue
    end
  rescue
    # reach:disable-next-line bare_rescue -- user handlers are arbitrary code; log and keep intercepting
    exception ->
      Logger.error(
        "Obscurax interception handler raised: " <>
          Exception.format(:error, exception, __STACKTRACE__)
      )

      :continue
  end
end
