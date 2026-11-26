# Security policy

## Reporting

Report privately through a GitHub security advisory: https://github.com/pendiego/OpenSir/security/advisories/new

Please do not open a public issue for these.

## Scope

- Anything that could make OpenSir write wrong, partial or out-of-range data to a controller, or send a command that could harm it.
- Ways to make the app write to an unsupported or read-only model.
- The Tauri command surface (IPC), file import/export, and the webview content policy.

Include the controller model, connection (cable, 2.4G dongle, Bluetooth), firmware and the steps to reproduce. You can expect a first reply within a week.

## Supported versions

Only the latest release and `main` get fixes.
