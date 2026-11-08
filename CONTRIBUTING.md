# Contributing to Quick Shelf

Thank you for your interest in contributing to Quick Shelf.

## Guidelines

- Keep the focus light: Every addition should be minimal in memory and CPU usage.
- Wayland first: Maintain compatibility with Wayland compositors using the Layer Shell protocol.
- No heavy runtime overhead: Avoid background polling when idle. Use file monitors and Unix socket events.
- Test workflows: Ensure the GitHub Actions build workflow passes for all PRs.

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/bitzzdev/quickShelf.git
   cd quickShelf
   ```

2. Install system development headers:
   ```bash
   sudo apt-get install libgtk-3-dev libgtk-layer-shell-dev pkg-config
   ```

3. Run checks:
   ```bash
   cargo check
   cargo test
   ```

4. Format code:
   ```bash
   cargo fmt --check
   ```

## Pull Request Process

1. Create a branch for your fix or feature.
2. Commit with descriptive messages explaining the rationale and changes.
3. Open a pull request against the `main` or `dev` branch.
