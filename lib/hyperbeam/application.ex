defmodule Hyperbeam.Application do
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = [
      Hyperbeam.Server,
    ]

    opts = [strategy: :one_for_one, name: Hyperbeam.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
