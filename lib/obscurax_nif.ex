defmodule Obscurax.Nif do
  use Rustler,
    otp_app: :obscurax,
    crate: :obscurax,
    mode: if(Mix.env() == :prod, do: :release, else: :debug)

  def hello, do: :erlang.nif_error(:nif_not_loaded)

  def browser_new(_opts), do: :erlang.nif_error(:nif_not_loaded)
  def browser_new_page(_browser, _pid), do: :erlang.nif_error(:nif_not_loaded)
  def browser_cookies(_browser), do: :erlang.nif_error(:nif_not_loaded)

  def page_goto(_page, _url), do: :erlang.nif_error(:nif_not_loaded)
  def page_url(_page), do: :erlang.nif_error(:nif_not_loaded)
  def page_close(_page), do: :erlang.nif_error(:nif_not_loaded)

  def page_evaluate(_page, _expr), do: :erlang.nif_error(:nif_not_loaded)
  def page_content(_page), do: :erlang.nif_error(:nif_not_loaded)
  def page_query_selector(_page, _selector), do: :erlang.nif_error(:nif_not_loaded)
  def page_wait_for_selector(_page, _selector, _timeout_ms), do: :erlang.nif_error(:nif_not_loaded)
  def page_settle(_page, _max_ms), do: :erlang.nif_error(:nif_not_loaded)
  def page_add_preload_script(_page, _script), do: :erlang.nif_error(:nif_not_loaded)
  def page_element_text(_page, _node_id), do: :erlang.nif_error(:nif_not_loaded)
  def page_element_attribute(_page, _node_id, _name), do: :erlang.nif_error(:nif_not_loaded)
  def page_element_click(_page, _node_id), do: :erlang.nif_error(:nif_not_loaded)

  def page_on_request(_page, _callback_id, _callback_pid), do: :erlang.nif_error(:nif_not_loaded)
  def page_on_response(_page, _callback_id, _callback_pid), do: :erlang.nif_error(:nif_not_loaded)
  def page_off_request(_page, _id), do: :erlang.nif_error(:nif_not_loaded)
  def page_off_response(_page, _id), do: :erlang.nif_error(:nif_not_loaded)
  def page_enable_interception(_page, _callback_pid), do: :erlang.nif_error(:nif_not_loaded)
  def reply_intercept(_page, _intercept_id, _decision), do: :erlang.nif_error(:nif_not_loaded)
end
