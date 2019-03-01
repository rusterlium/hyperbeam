defmodule Hyper.Server do
  use GenServer

  require Logger

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def init(opts) do
    {:ok, rx} = Hyper.Native.start(opts)
    {:ok, rx}
  end

  def handle_info({:request, req}, state) do
    Task.start(fn -> Hyper.Server.handle_request(req) end)

    {:noreply, state}
  end

  def handle_request(req) do
    #Logger.info(fn -> "Processing request on #{inspect self()}" end)
    :ok = Hyper.Native.send_resp(req.resource, "Hello from the #{inspect self()}")
  end

  def terminate(_, rx) do
    Hyper.stop(rx)
    :ok
  end
end
