defmodule Obscurax.Error do
  @moduledoc """
  Structured error for all Obscurax operations.

  All NIF operations return `{:ok, result}` or `{:error, %Obscurax.Error{}}`.
  Bang variants (`goto!/2`, etc.) raise this as an exception.
  """

  @type kind ::
          :navigation
          | :js_eval
          | :timeout
          | :element_not_found
          | :no_page
          | :page_closed
          | :internal

  @type t :: %__MODULE__{
          kind: kind(),
          message: String.t(),
          context: map()
        }

  defexception [:kind, :message, :context]

  @impl true
  def exception(opts) do
    %__MODULE__{
      kind: Keyword.fetch!(opts, :kind),
      message: Keyword.fetch!(opts, :message),
      context: Keyword.get(opts, :context, %{})
    }
  end

  @impl true
  def message(%__MODULE__{kind: kind, message: msg}) do
    "[#{kind}] #{msg}"
  end
end
