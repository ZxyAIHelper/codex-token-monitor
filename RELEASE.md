# Release Guide

This project ships a Tauri desktop app. The current bundle target is Windows NSIS, so release packages are `.exe` installers.

## Local Build

Install dependencies:

```powershell
npm install
```

Run validation:

```powershell
npm test
npm run build
```

Build the installer:

```powershell
npm run tauri build
```

The installer is generated under:

```text
src-tauri/target/release/bundle/nsis/
```

## GitHub Release

The repository includes `.github/workflows/release.yml`.

To publish by tag:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The workflow will:

1. Install Node.js and Rust.
2. Install npm dependencies with `npm ci`.
3. Run `npm test`.
4. Build the Tauri installer with `npm run tauri build`.
5. Upload the NSIS `.exe` as a workflow artifact.
6. Attach the `.exe` to a GitHub Release when the run is triggered by a `v*` tag.

To publish manually:

1. Open the repository on GitHub.
2. Go to `Actions`.
3. Select `Release`.
4. Click `Run workflow`.
5. Enter a tag such as `v0.1.0`.

## Version Checklist

Before creating a release:

- Update `package.json` version.
- Update `src-tauri/Cargo.toml` version.
- Update `src-tauri/tauri.conf.json` version.
- Run `npm test`.
- Run `npm run tauri build` locally if you want to verify the installer before tagging.
