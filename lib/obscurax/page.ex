defmodule Obscurax.Page do
  @moduledoc """
  A browser tab/page — navigation, V8 evaluation, element ops, and callbacks.

  Each page runs on a dedicated OS thread with its own `current_thread` tokio
  runtime and V8 isolate. The synchronous-feeling API blocks the caller while
  async operations run on the page thread; V8 evaluation is scheduled on dirty
  CPU schedulers so it never blocks the BEAM.

  ## Navigation and evaluation

      :ok = Obscurax.Page.goto(page, "https://example.com")
      {:ok, title} = Obscurax.Page.evaluate(page, "document.title")
      {:ok, html} = Obscurax.Page.content(page)

  ## Element operations

  Element operations take a `node_id` returned by `query_selector/2` or
  `wait_for_selector/3`:

      {:ok, node_id} = Obscurax.Page.query_selector(page, "h1")
      {:ok, text} = Obscurax.Page.element_text(page, node_id)
      :ok = Obscurax.Page.element_click(page, node_id)

  ## Request observers

      {:ok, _pid} = Obscurax.Page.on_request(page, fn req ->
        IO.inspect(req.url)
      end)

  ## Request interception

      :ok = Obscurax.Page.enable_interception(page)
      receive do
        {:obscurax_intercept, ref, %{url: url}} ->
          :ok = Obscurax.reply_intercept(page, ref, :continue)
      end
  """

  @type t :: %__MODULE__{ref: reference()}

  defstruct [:ref]

  alias Obscurax.{Error, Nif}

  @default_timeout 30_000

  @spec goto(t(), String.t(), keyword()) :: :ok | {:error, Error.t()}
  def goto(%__MODULE__{ref: ref}, url, opts \\ []) do
    timeout = Keyword.get(opts, :timeout, @default_timeout)
    id = Nif.page_goto(ref, url)

    receive do
      {:obscurax_result, ^id, :ok} ->
        :ok

      {:obscurax_result, ^id, :error, msg} ->
        {:error, Error.exception(kind: :navigation, message: msg, context: %{url: url})}
    after
      timeout ->
        {:error, Error.exception(kind: :timeout, message: "goto timed out", context: %{url: url})}
    end
  end

  @spec goto!(t(), String.t(), keyword()) :: :ok
  def goto!(page, url, opts \\ []) do
    case goto(page, url, opts) do
      :ok -> :ok
      {:error, error} -> raise error
    end
  end

  @spec evaluate(t(), String.t()) :: {:ok, term()} | {:error, Error.t()}
  def evaluate(%__MODULE__{ref: ref}, expr) do
    case Nif.page_evaluate(ref, expr) do
      {:ok, value} -> {:ok, value}
      {:error, error} -> {:error, error}
    end
  end

  @spec evaluate!(t(), String.t()) :: term()
  def evaluate!(page, expr) do
    case evaluate(page, expr) do
      {:ok, value} -> value
      {:error, error} -> raise error
    end
  end

  @spec url(t()) :: {:ok, String.t()} | {:error, Error.t()}
  def url(%__MODULE__{ref: ref}) do
    case Nif.page_url(ref) do
      {:ok, url} -> {:ok, url}
      {:error, error} -> {:error, error}
    end
  end

  @spec content(t()) :: {:ok, String.t()} | {:error, Error.t()}
  def content(%__MODULE__{ref: ref}) do
    case Nif.page_content(ref) do
      {:ok, html} -> {:ok, html}
      {:error, error} -> {:error, error}
    end
  end

  @spec query_selector(t(), String.t()) :: {:ok, non_neg_integer() | nil} | {:error, Error.t()}
  def query_selector(%__MODULE__{ref: ref}, selector) do
    case Nif.page_query_selector(ref, selector) do
      {:ok, node_id} -> {:ok, node_id}
      nil -> {:ok, nil}
      {:error, error} -> {:error, error}
    end
  end

  @spec wait_for_selector(t(), String.t(), non_neg_integer()) ::
          {:ok, non_neg_integer()} | {:error, Error.t()}
  def wait_for_selector(%__MODULE__{ref: ref}, selector, timeout_ms) do
    case Nif.page_wait_for_selector(ref, selector, timeout_ms) do
      {:ok, node_id} -> {:ok, node_id}
      {:error, error} -> {:error, error}
    end
  end

  @spec wait_for_selector!(t(), String.t(), non_neg_integer()) :: non_neg_integer()
  def wait_for_selector!(page, selector, timeout_ms) do
    case wait_for_selector(page, selector, timeout_ms) do
      {:ok, node_id} -> node_id
      {:error, error} -> raise error
    end
  end

  @spec element_text(t(), non_neg_integer()) :: {:ok, String.t()} | {:error, Error.t()}
  def element_text(%__MODULE__{ref: ref}, node_id) do
    case Nif.page_element_text(ref, node_id) do
      {:ok, text} -> {:ok, text}
      {:error, error} -> {:error, error}
    end
  end

  @spec element_attribute(t(), non_neg_integer(), String.t()) ::
          {:ok, String.t()} | nil | {:error, Error.t()}
  def element_attribute(%__MODULE__{ref: ref}, node_id, name) do
    case Nif.page_element_attribute(ref, node_id, name) do
      {:ok, value} -> {:ok, value}
      nil -> nil
      {:error, error} -> {:error, error}
    end
  end

  @spec element_click(t(), non_neg_integer()) :: :ok | {:error, Error.t()}
  def element_click(%__MODULE__{ref: ref}, node_id) do
    case Nif.page_element_click(ref, node_id) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec settle(t(), non_neg_integer()) :: :ok | {:error, Error.t()}
  def settle(%__MODULE__{ref: ref}, max_ms) do
    case Nif.page_settle(ref, max_ms) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec add_preload_script(t(), String.t()) :: :ok | {:error, Error.t()}
  def add_preload_script(%__MODULE__{ref: ref}, script) do
    case Nif.page_add_preload_script(ref, script) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec on_request(t(), (map() -> term())) :: {:ok, pid()} | {:error, Error.t()}
  def on_request(%__MODULE__{ref: ref}, fun) do
    case Obscurax.Callback.start_link(ref, :request, fun) do
      {:ok, pid} ->
        {:ok, pid}

      {:error, %Error{} = error} ->
        {:error, error}

      {:error, _} ->
        {:error, Error.exception(kind: :internal, message: "callback start failed", context: %{})}
    end
  end

  @spec on_response(t(), (map() -> term())) :: {:ok, pid()} | {:error, Error.t()}
  def on_response(%__MODULE__{ref: ref}, fun) do
    case Obscurax.Callback.start_link(ref, :response, fun) do
      {:ok, pid} ->
        {:ok, pid}

      {:error, %Error{} = error} ->
        {:error, error}

      {:error, _} ->
        {:error, Error.exception(kind: :internal, message: "callback start failed", context: %{})}
    end
  end

  @spec enable_interception(t()) :: :ok | {:error, Error.t()}
  def enable_interception(%__MODULE__{ref: ref}) do
    case Nif.page_enable_interception(ref, self()) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec close(t()) :: :ok
  def close(%__MODULE__{ref: ref}) do
    Nif.page_close(ref)
    :ok
  end
end
