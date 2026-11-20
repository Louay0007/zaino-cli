# Changelog

## 1.0.1

- Fix `npm install -g zainocli` producing a CLI that could not launch. The 1.0.0
  wrapper declared optional dependencies on six platform packages, but only the
  wrapper was ever published, so every install failed at startup with
  `Missing optional dependency zainocli-darwin-arm64`. The wrapper now declares
  only the platforms a release actually publishes, and refuses to be staged with
  none.
- Publish the macOS platform packages. Staging downloaded darwin artifacts under
  the bare target name while `package-macos` uploads them with a `-packaged`
  suffix, so the darwin packages were never built.
- Publish the Linux platform packages. Staging required every target to be
  present before using the artifacts already downloaded by the workflow, so it
  fell back to fetching a workflow run it could not resolve.
- npm publication no longer waits on the macOS signing and notarization chain.
  That chain is optional; when it fails, the darwin packages are skipped and the
  remaining platforms still publish.
- Ship executable macOS binaries. Without signing credentials the staging step
  matched its input with a `"${binary}*"` glob, which selected the
  `<binary>-<target>-unsigned.zst` archive, so the darwin packages contained
  zstd-compressed data instead of Mach-O executables and `zaino` failed with
  `ENOEXEC`. The exact name is matched first, the archive is decompressed when
  it is the only input, and a result that still carries the zstd magic number is
  rejected before publication.
- Publish each package on its first release. Adding Windows signing is not
  needed for the four built platforms to appear: a missing tarball is skipped
  with a warning instead of aborting the publish, an npm name that the registry
  has never seen no longer fails the dist-tag lookup, the release runs when the
  repository has no published release yet, and scoped packages publish with
  explicit public access.

## 1.0.0 (2026-11-14)

- Fork of OpenAI Codex CLI, rebranded as Zaino.
- Default provider is `zaino`: Responses API at `https://api.zaino.soon.it/v1`.
- Auth via `ZAINO_API_KEY` (`zn-`), stored in `$ZAINO_HOME/.env` by `zaino login --with-api-key`.
- Home directory is `~/.zaino`; project config is `./.zaino/config.toml`.
- Bundled catalog ships four aliases: `fast`, `smart`, `frontier`, `fable` (1M context each).
- Telemetry disabled by default.
- Published to npm as `zainocli`; the `zaino` name is reserved on npm.
