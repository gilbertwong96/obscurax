defmodule Obscurax.CookieStore do
  @moduledoc "Cookie management for a browser session."

  @type t :: %__MODULE__{ref: reference()}

  defstruct [:ref]

  alias Obscurax.{Error, Nif}

  @spec set(t(), String.t(), String.t()) :: :ok | {:error, Error.t()}
  def set(%__MODULE__{ref: ref}, set_cookie, url) do
    case Nif.cookie_set(ref, set_cookie, url) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec get_all(t()) :: {:ok, [map()]} | {:error, Error.t()}
  def get_all(%__MODULE__{ref: ref}) do
    case Nif.cookie_get_all(ref) do
      {:ok, cookies} -> {:ok, cookies}
      {:error, error} -> {:error, error}
    end
  end

  @spec get_for_url(t(), String.t()) :: {:ok, [map()]} | {:error, Error.t()}
  def get_for_url(%__MODULE__{ref: ref}, url) do
    case Nif.cookie_get_for_url(ref, url) do
      {:ok, cookies} -> {:ok, cookies}
      {:error, error} -> {:error, error}
    end
  end

  @spec save(t(), String.t()) :: :ok | {:error, Error.t()}
  def save(%__MODULE__{ref: ref}, path) do
    case Nif.cookie_save(ref, path) do
      :ok -> :ok
      {:error, error} -> {:error, error}
    end
  end

  @spec load(t(), String.t()) :: {:ok, non_neg_integer()} | {:error, Error.t()}
  def load(%__MODULE__{ref: ref}, path) do
    case Nif.cookie_load(ref, path) do
      {:ok, count} -> {:ok, count}
      {:error, error} -> {:error, error}
    end
  end
end
