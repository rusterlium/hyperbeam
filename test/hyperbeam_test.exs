defmodule HyperbeamTest do
  use ExUnit.Case
  doctest Hyperbeam

  test "greets the world" do
    assert Hyperbeam.hello() == :world
  end
end
