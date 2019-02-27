defmodule Hyper.Mixfile do
  use Mix.Project

  def project do
    [
      app: :hyper,
      compilers: [:rustler] ++ Mix.compilers(),
      version: "0.1.0",
      elixir: "~> 1.5",
      rustler_crates: [hyperbeam: []],
      start_permanent: Mix.env == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Hyper.Application, []}
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.20"},
    ]
  end
end
