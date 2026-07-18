defmodule Obscurax.MixProject do
  use Mix.Project

  def project do
    [
      app: :obscurax,
      version: "0.1.0",
      elixir: "~> 1.20",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases(),
      test_coverage: [summary: [threshold: 35]]
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:rustler, "~> 0.38", runtime: false},
      # ── Quality (dev/test only) ─────────────────────────────
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_dna, "~> 1.5", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.40", only: :dev, runtime: false},
      {:ex_slop, "~> 0.4", only: [:dev, :test], runtime: false},
      {:mix_audit, "~> 2.1", only: [:dev, :test], runtime: false},
      {:reach, "~> 2.7", only: [:dev, :test], runtime: false},
      {:stream_data, "~> 1.3", only: [:dev, :test], runtime: false}
    ]
  end

  defp aliases do
    [
      ci: [
        "compile --all-warnings --warnings-as-errors",
        "format --check-formatted",
        "credo --strict",
        "deps.unlock --check-unused",
        "deps.audit",
        "xref graph --label compile-connected --fail-above 5",
        "dialyzer",
        "ex_dna",
        "reach.check --dead-code --smells",
        "test --cover"
      ]
    ]
  end

  def cli do
    [
      preferred_envs: [ci: :test]
    ]
  end
end
