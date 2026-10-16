defmodule Obscurax.TestFixture do
  @moduledoc false

  # Served as a data: URL so DOM behaviour tests exercise our own markup and
  # never break when a live third-party page changes.
  @html """
  <!doctype html>
  <html>
  <head>
    <meta charset="utf-8">
    <title>Example Domain</title>
  </head>
  <body>
    <h1>Example Domain</h1>
    <p id="lead">This domain is for use in documentation examples.</p>
    <a id="link" href="#more">More information</a>
  </body>
  </html>
  """

  @url "data:text/html;base64," <> Base.encode64(@html)

  def goto!(page), do: Obscurax.Page.goto!(page, @url)

  @doc """
  The same markup written to a temp file, as a `file://` URL.

  Used to exercise opt-in local file access.
  """
  def file_url do
    path = Path.join(System.tmp_dir!(), "obscurax_test_fixture.html")
    File.write!(path, @html)
    "file://" <> path
  end
end
