# Contributing to txio-desktop

First off, thank you for considering contributing! It's people like you that bring txio to the desktop.

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check the [existing issues](https://github.com/Txio-labs/txio-desktop/issues). When you do open one, please include:

* A clear and descriptive title
* Your OS/platform (Linux/macOS/Windows) and build target (AppImage/dmg/nsis)
* The exact steps that reproduce the problem
* What you expected to happen vs. what actually happened

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. Please include:

* A clear and descriptive title
* A step-by-step description of the suggested enhancement
* Why it would be useful to most txio users

### Pull Requests

* Do not include issue numbers in the PR title.
* Before merging, automated checks must pass: build and packaging.
* End all files with a newline.
* If you're touching `package.json`'s `build` config (electron-builder targets/icons), test a local package build for at least one platform before opening the PR.

## Development Setup

```bash
npm install
npm start        # runs Electron against src/index.js
npm run build     # packages via electron-builder (electron-builder.build config in package.json)
```

This app wraps the txio web frontend as a desktop binary — it shares the same API client and UI as the browser app, just outside the browser.

## Styleguides

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line

### Code Style

* Keep platform-specific packaging config (`build.linux`, `build.win`, `build.mac` in `package.json`) changes isolated from application code changes in the same PR where possible
