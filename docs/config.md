# Configuration

Defaults work with no config file. Optional overrides live in `~/.zaino/config.toml`, or
`./.zaino/config.toml` for a single project.

```toml
model = "frontier"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

Any value can be set for one run without a file:

```shell
zaino -c model=fast "quick question"
```

See [example-config.md](./example-config.md) for a fuller file.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
