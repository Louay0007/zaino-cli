# Windows follow-up (after the 4-target release publishes)

## Current state
- `zainocli-win32-x64` and `zainocli-win32-arm64` are already defined in
  `codex-cli/scripts/build_npm_package.py` with correct os/cpu fields.
- `.github/workflows/rust-release-windows.yml` builds both targets and its
  runner labels are already fixed to `windows-latest`.
- `rust-release.yml` does NOT call it — the `build-windows` job was removed
  when the workflow was trimmed to the npm publish path.

## Three changes needed
1. Re-add a job to `rust-release.yml`:
     build-windows:
       needs: tag-check
       uses: ./.github/workflows/rust-release-windows.yml
       secrets: inherit
2. Add `build-windows` to `stage-npm-packages.needs`.
3. Add the windows artifact names to the staging download pattern
   (`{x86_64,aarch64}-pc-windows-msvc`).

## Caveat
`aarch64-pc-windows-msvc` on `windows-latest` is a cross-compile (the runner is
x64). It may need the MSVC ARM64 toolchain; x64 Windows is the safe first target.

## Expected cost
~60 min CI cycle, untested path. Do this only after the macOS+Linux release is
green and installable, so a Windows failure cannot block working platforms.
