# Codex Token Monitor

[中文](./README.md)

Codex Token Monitor is a local desktop app for monitoring Codex token usage on your own machine. It reads local Codex session logs and summarizes usage by day, hour, session, input/output tokens, cached input tokens, reasoning output tokens, and tool-output risk signals.

## Features

- Live dashboard: today's tokens, last hour, last 5 hours, and active sessions.
- Trend views: last 24 hours, last 30 days, and last 12 weeks.
- Session table: sort by total tokens, input, output, tool calls, tool output size, and last seen time.
- Session detail: inspect recorded inputs and per-turn token usage.
- Alerts: high session usage, high hourly usage, and large tool output.
- System tray: run in the background, open the dashboard, pause/resume monitoring, and quit.
- Bilingual UI: switch between English and Chinese in the app.

## Data Source

The app only reads local Codex logs and does not upload your data.

- Windows: `C:\Users\<user>\.codex\sessions`
- macOS/Linux: `~/.codex/sessions`

The MVP focuses on local log monitoring. Tool output size is a risk indicator, not an exact measurement of tool token usage.

## Development

Requirements:

- Node.js 20 or later
- Rust stable
- System dependencies required by Tauri 2

Install dependencies:

```powershell
npm install
```

Run in development mode:

```powershell
npm run tauri dev
```

Run tests:

```powershell
npm test
```

Build the frontend:

```powershell
npm run build
```

Build the Windows installer:

```powershell
npm run tauri build
```

Installer artifacts are usually generated under:

```text
src-tauri/target/release/bundle/nsis/
```

## Release Installers

This repository includes a GitHub Actions release workflow. Push a `v*` tag to build the Windows NSIS installer and upload the `.exe` file to a GitHub Release.

Example:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

You can also run the `Release` workflow manually from GitHub Actions and provide the tag to publish. See [RELEASE.md](./RELEASE.md) for details.

## License

This project is licensed under the [MIT License](./LICENSE).

## Note

Codex Token Monitor is a community tool, not an official OpenAI product. It analyzes usage fields from local logs only. Actual billing and quota data should be verified with official account and platform data.
