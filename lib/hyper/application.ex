defmodule Hyper.Application do
  @moduledoc false

  use Application

  def start(_type, _args) do
    children = [
      Hyper.Server,
    ]

    opts = [strategy: :one_for_one, name: Hyper.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
