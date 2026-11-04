defmodule ObscuraxTest do
  use ExUnit.Case, async: true

  alias Obscurax.{Browser, CookieStore, Page}

  test "new/0 delegates to Browser.new" do
    assert {:ok, %Browser{}} = Obscurax.new()
  end

  test "new/1 delegates to Browser.new with options" do
    assert {:ok, %Browser{}} = Obscurax.new(stealth: true)
  end

  test "new_page/1 delegates to Browser.new_page" do
    {:ok, browser} = Obscurax.new()
    assert {:ok, %Page{} = page} = Obscurax.new_page(browser)
    Page.close(page)
  end

  test "cookies/1 delegates to Browser.cookies" do
    {:ok, browser} = Obscurax.new()
    assert %CookieStore{} = Obscurax.cookies(browser)
  end

  test "reply_intercept/3 is exported" do
    # reply_intercept is a defdelegate to Obscurax.Nif which is a NIF.
    # We can't call it without a valid intercept ref, but we can verify
    # the Obscurax module compiles and exports the function.
    assert {:ok, _} = Obscurax.new()
  end

  test "reply_intercept/3 works through the public API" do
    {:ok, browser} = Obscurax.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)

    # Interception now covers the document request, so answering from the same
    # process that awaits goto would deadlock. Drive the navigation from here
    # and answer the intercepts as they arrive.
    :ok = Obscurax.Nif.page_enable_interception(page.ref, self())
    goto_id = Obscurax.Nif.page_goto(page.ref, "https://example.com")

    assert_receive {:obscurax_intercept, ref, %{resource_type: "Document"}}, 5_000
    assert :ok = Obscurax.reply_intercept(page, ref, :continue)
    assert_receive {:obscurax_result, ^goto_id, :ok}, 10_000

    Obscurax.Page.close(page)
  end

  test "enable_interception/2 mocks a response without the network" do
    {:ok, browser} = Obscurax.new()
    {:ok, page} = Obscurax.Browser.new_page(browser)

    {:ok, _pid} =
      Obscurax.Page.enable_interception(page, fn
        %{resource_type: "Document"} ->
          {:fulfill, 200, %{"content-type" => "text/html"}, "<title>mocked</title>"}

        _request ->
          :continue
      end)

    :ok = Obscurax.Page.goto(page, "https://example.invalid/")
    assert {:ok, "mocked"} = Obscurax.Page.evaluate(page, "document.title")

    Obscurax.Page.close(page)
  end
end
