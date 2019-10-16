defmodule Hyper.Server do
  use GenServer

  require Logger

  defstruct read: nil, shutdown: nil

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def init(opts) do
    {:ok, shutdown, read} = Hyper.Native.start(opts)
    :ok = Hyper.Native.batch_read(read)
    {:ok, %__MODULE__{read: read, shutdown: shutdown}}
  end

  def handle_info({:request, requests}, state) do
    for request <- requests do
      Task.start(fn -> Hyper.Server.handle_request(request) end)
    end

    :ok = Hyper.Native.batch_read(state.read)

    {:noreply, state}
  end

  def handle_request(req) do
    #Logger.info(fn -> "Processing request on #{inspect self()}" end)
    :ok = Hyper.Native.send_resp(req.resource, "Hello from the #{inspect self()}")
  end

  def terminate(_, state) do
    Hyper.stop(state.shutdown)
    :ok
  end
end
