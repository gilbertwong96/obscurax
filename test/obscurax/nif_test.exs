defmodule Obscurax.NifTest do
  use ExUnit.Case, async: true

  test "hello NIF loads and returns :world" do
    assert Obscurax.Nif.hello() == :world
  end
end
