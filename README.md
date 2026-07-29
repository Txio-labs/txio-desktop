# txio Desktop

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

The txio web app, wrapped as a desktop binary. Same API client, same visual transaction builder for Sui — just outside the browser.

Built on Electron, packaged for Linux/macOS/Windows via `electron-builder`.

---

## Prerequisites

- Node.js and npm

## Running in development

```bash
npm install
npm start
```

## Building

```bash
npm install
npm run build
```

This hands off to `electron-builder` (see the `build` config in
`package.json`) to package the executable for your platform: AppImage on
Linux, nsis on Windows, dmg on macOS.

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for bug reports, enhancement
suggestions, and the PR process. Please also read our
[Code of Conduct](./CODE_OF_CONDUCT.md).

## License

MIT. See [LICENSE](./LICENSE).
