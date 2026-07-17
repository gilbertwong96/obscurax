defmodule Obscurax.Nif do
  use Rustler,
    otp_app: :obscurax,
    crate: :obscurax,
    mode: if(Mix.env() == :prod, do: :release, else: :debug)

  def hello, do: :erlang.nif_error(:nif_not_loaded)
end
