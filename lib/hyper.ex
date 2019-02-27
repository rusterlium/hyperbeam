defmodule Hyper do
  alias Hyper.Native

  def start(opts \\ %{}) do
    Native.start(opts)
  end

  def stop(shutdown_tx) do
    Native.stop(shutdown_tx)
  end
end
