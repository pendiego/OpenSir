# Security policy

## Reporting

Report privately through a GitHub security advisory: https://github.com/pendiego/OpenSir/security/advisories/new

If that page is not available, open an issue titled "Private security contact" with no details. The maintainer will reply there with a private channel.

Do not put vulnerability details in a public issue.

## Scope

- Anything that could make OpenSir write wrong, partial or out-of-range data to a controller, or send a command that could harm it.
- Ways to make the app write to an unsupported or read-only model.
- The Tauri command surface (IPC), file import/export, and the webview content policy.

Include the controller model, connection (cable, 2.4G dongle, Bluetooth), firmware and the steps to reproduce. You can expect a first reply within a week.

## Supported versions

Only the latest release and `main` get fixes.
