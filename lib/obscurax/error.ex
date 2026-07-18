defmodule Obscurax.Error do
  @moduledoc """
  Structured error for all Obscurax operations.

  All NIF operations return `{:ok, result}` or `{:error, %Obscurax.Error{}}`.
  Bang variants (`goto!/2`, etc.) raise this as an exception.

  The `context` map carries operation-specific details that vary by `kind`:

  | kind                 | context fields                          |
  |----------------------|-----------------------------------------|
  | `:navigation`        | `url`                                   |
  | `:timeout`           | `url` *or* `selector` + `timeout_ms`    |
  | `:element_not_found` | `selector` *and/or* `node_id`          |
  | `:js_eval`           | `expression`                            |
  | `:no_page`           | *(empty)*                               |
  | `:page_closed`       | *(empty)*                               |
  | `:internal`          | *(empty)*                               |

  All context keys are atoms. Fields not relevant to a given `kind` are `nil`.
  """

  @type kind ::
          :navigation
          | :js_eval
          | :timeout
          | :element_not_found
          | :no_page
          | :page_closed
          | :internal

  @type context :: %{
          optional(:url) => String.t(),
          optional(:selector) => String.t(),
          optional(:timeout_ms) => non_neg_integer(),
          optional(:node_id) => non_neg_integer(),
          optional(:expression) => String.t(),
          optional(atom()) => term()
        }

  @type t :: %__MODULE__{
          kind: kind(),
          message: String.t(),
          context: context()
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
