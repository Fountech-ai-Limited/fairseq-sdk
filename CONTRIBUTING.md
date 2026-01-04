# Contributing to Fairseq

Thank you for your interest in contributing to Fairseq! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@fairseq.io.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, SDK version)
- Relevant code snippets or error messages

### Suggesting Features

Feature requests are welcome. Please provide:

- A clear description of the feature
- The problem it solves
- Example use cases
- Any implementation ideas you have

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit with clear messages
8. Push to your fork
9. Open a Pull Request

### Commit Messages

We follow conventional commits:

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation changes
- `test:` test additions/changes
- `refactor:` code refactoring
- `chore:` maintenance tasks

Example: `feat: add batch proof generation`

## Development Setup

```bash
# Clone the repo
git clone https://github.com/Fountech-ai-Limited/fairseq-sdk.git
cd fairseq-sdk

# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_prover
```

## Code Style

- Follow Rust idioms and best practices
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation for public APIs
- Add tests for new functionality

## Questions?

- Documentation: https://fairseq.io/docs
- Discord: https://discord.gg/fairseq
- Email: engineering@fairseq.io

## License

By contributing, you agree that your contributions will be licensed under the MIT/Apache-2.0 dual license.
