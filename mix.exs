defmodule Hyperbeam.Mixfile do
  use Mix.Project

  def project do
    [
      app: :hyperbeam,
      compilers: [:rustler] ++ Mix.compilers(),
      version: "0.1.0",
      elixir: "~> 1.9",
      rustler_crates: [hyperbeam: []],
      start_permanent: Mix.env == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Hyperbeam.Application, []}
    ]
  end

  defp deps do
    [
      {:rustler, github: "rusterlium/rustler", sparse: "rustler_mix", branch: "master"},
    ]
  end
end
