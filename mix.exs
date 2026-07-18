defmodule Obscurax.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/gilbertwong96/obscurax"
  @dev? String.ends_with?(@version, "-dev")
  @force_build? System.get_env("OBSCURAX_BUILD") in ["1", "true"]

  def project do
    [
      app: :obscurax,
      version: @version,
      elixir: "~> 1.20",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases(),
      package: package(),
      description: description(),
      test_coverage: [summary: [threshold: 80], ignore_modules: [Obscurax.Nif]]
    ]
  end

  def application do
    [extra_applications: [:logger, :inets, :ssl]]
  end

  defp deps do
    [
      {:rustler_precompiled, "~> 0.8"},
      {:rustler, "~> 0.38", optional: not (@dev? or @force_build?)},
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

  defp description do
    "Elixir binding for the obscura headless browser engine"
  end

  defp package do
    [
      files: ~w(
        lib
        native/obscurax/src
        native/obscurax/Cargo.*
        native/obscurax/rust-toolchain.toml
        native/obscurax/rustfmt.toml
        checksum-*.exs
        mix.exs
        README.md
        LICENSE
      ),
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url}
    ]
  end

  def cli do
    [
      preferred_envs: [ci: :test]
    ]
  end
end
