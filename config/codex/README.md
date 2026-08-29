# Codex OpenTelemetry

This package installs `~/.local/bin/codex-otel-configure`. The helper materializes Codex's native OTLP/HTTP metrics and trace exporters into the host-local `~/.codex/config.toml`.

The credential is never stored in this repository. The helper reads the same protected base64 Basic-auth payload used by OpenCode:

```text
${XDG_STATE_HOME:-$HOME/.local/state}/opencode/otel-basic-auth
```

## Configure

After installing/logging in to Codex, activate this package from the `devenv` checkout:

```bash
cd ~/devenv
pnpm --filter @rafaeltab-devenv/codex-config run activate
```

Then materialize and verify the host-local configuration:

```bash
chmod 600 "${XDG_STATE_HOME:-$HOME/.local/state}/opencode/otel-basic-auth"
codex-otel-configure --dry-run
codex-otel-configure apply
codex-otel-configure check
```

`apply` preserves unrelated Codex settings, manages only its marked telemetry block, writes atomically, and forces `~/.codex/config.toml` to mode `600`. It refuses to replace unmanaged `[analytics]` or `[otel...]` sections.

`check` verifies:

- the secret and generated config permissions;
- that the generated block matches the current credential;
- that the installed Codex CLI parses the config;
- authenticated access to both `/v1/metrics` and `/v1/traces`.

Use `codex-otel-configure check --no-network` for an offline config-only check.

## Privacy defaults

- Metrics: enabled.
- Traces: enabled.
- Structured OTEL logs: disabled (`exporter = "none"`).
- User prompt logging: disabled (`log_user_prompt = false`).

Codex currently supports static exporter headers but not OpenCode's dynamic header-helper mechanism. The generated credential therefore lives only in the host-local `~/.codex/config.toml`, duplicated once for each exporter, and is refreshed safely by rerunning `codex-otel-configure apply` after credential rotation.
