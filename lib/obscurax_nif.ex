defmodule Obscurax.Nif do
  use Rustler,
    otp_app: :obscurax,
    crate: :obscurax,
    mode: if(Mix.env() == :prod, do: :release, else: :debug)

  def hello, do: :erlang.nif_error(:nif_not_loaded)

  def browser_new(_opts), do: :erlang.nif_error(:nif_not_loaded)
  def browser_new_page(_browser, _pid), do: :erlang.nif_error(:nif_not_loaded)
  def browser_cookies(_browser), do: :erlang.nif_error(:nif_not_loaded)
end
