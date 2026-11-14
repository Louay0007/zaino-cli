# Authentication

Zaino authenticates with an API key starting with `zn-`.

```shell
zaino login --with-api-key
```

The key is written to `$ZAINO_HOME/.env` (default `~/.zaino/.env`, mode `600`), which the CLI
loads on every start. `zaino logout` removes it.

Alternatively export it yourself:

```shell
export ZAINO_API_KEY=zn-...
```

`ZAINO_HOME` relocates the whole home directory, including `auth.json`, sessions and logs.

The CLI cannot fall back to a ChatGPT subscription: the `zaino` provider reads only
`ZAINO_API_KEY`, and `auth.json` is never consulted for it.
