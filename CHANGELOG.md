# Changelog

## 0.1.0 (unreleased)

- Fork of OpenAI Codex CLI, rebranded as Zaino.
- Default provider is `zaino`: Responses API at `https://api.zaino.soon.it/v1`.
- Auth via `ZAINO_API_KEY` (`zn-`), stored in `$ZAINO_HOME/.env` by `zaino login --with-api-key`.
- Home directory is `~/.zaino`; project config is `./.zaino/config.toml`.
- Bundled catalog ships four aliases: `fast`, `smart`, `frontier`, `fable` (1M context each).
- Telemetry disabled by default.
