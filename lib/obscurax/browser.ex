defmodule Obscurax.Browser do
  @moduledoc """
  Launches an obscura browser instance.

  A browser owns a shared cookie jar and spawns pages. Each `Obscurax.Page`
  gets its own OS thread and V8 isolate, so multiple pages from the same
  browser run in parallel.

  ## Options

    * `:stealth` — enable stealth mode (default: `false`)
    * `:proxy` — proxy URL string, e.g. `"socks5://127.0.0.1:1080"`
    * `:user_agent` — custom User-Agent header
    * `:storage_dir` — directory for persistent storage

  ## Example

      {:ok, browser} = Obscurax.Browser.new(stealth: true)
      {:ok, page} = Obscurax.Browser.new_page(browser)
      store = Obscurax.Browser.cookies(browser)
  """

  @type t :: %__MODULE__{ref: reference()}

  defstruct [:ref]

  alias Obscurax.Nif

  @spec new(keyword()) :: {:ok, t()} | {:error, Obscurax.Error.t()}
  def new(opts \\ []) do
    opts_map = %{
      stealth: Keyword.get(opts, :stealth, false),
      proxy: Keyword.get(opts, :proxy),
      user_agent: Keyword.get(opts, :user_agent),
      storage_dir: Keyword.get(opts, :storage_dir)
    }

    case Nif.browser_new(opts_map) do
      {:ok, ref} -> {:ok, %__MODULE__{ref: ref}}
      {:error, error} -> {:error, error}
    end
  end

  @spec new!(keyword()) :: t()
  def new!(opts \\ []) do
    case new(opts) do
      {:ok, browser} -> browser
      {:error, error} -> raise error
    end
  end

  @spec new_page(t()) :: {:ok, Obscurax.Page.t()} | {:error, Obscurax.Error.t()}
  def new_page(%__MODULE__{ref: ref}) do
    case Nif.browser_new_page(ref, self()) do
      {:ok, page_ref} -> {:ok, %Obscurax.Page{ref: page_ref}}
      {:error, error} -> {:error, error}
    end
  end

  @spec new_page!(t()) :: Obscurax.Page.t()
  def new_page!(browser) do
    case new_page(browser) do
      {:ok, page} -> page
      {:error, error} -> raise error
    end
  end

  @spec cookies(t()) :: Obscurax.CookieStore.t()
  def cookies(%__MODULE__{ref: ref}) do
    {:ok, store_ref} = Nif.browser_cookies(ref)
    %Obscurax.CookieStore{ref: store_ref}
  end
end
