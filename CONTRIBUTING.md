# Contributing to Revm

Thank you for your interest in contributing to Revm! This document provides guidelines and instructions for contributing to the project.

## Ways to Contribute

There are many ways to contribute to Revm:

- 🐛 **Bug Reports**: Report bugs you encounter
- 📝 **Documentation**: Improve docs, add examples, fix typos
- 💻 **Code**: Bug fixes, new features, performance improvements
- ✅ **Testing**: Write tests, improve test coverage
- 📖 **Examples**: Add usage examples
- 👀 **Code Review**: Review pull requests from other contributors

## Getting Started

### Prerequisites

- Rust (latest stable version recommended)
- Git
- `clang` (required for `c-kzg` or `secp256k1` feature flags)

### Development Setup

1. **Fork the repository** on GitHub

2. **Clone your fork**:
```bash
   git clone https://github.com/YOUR_USERNAME/revm.git
   cd revm
```

3. **Build the project**:
```bash
   cargo build --release
```

4. **Run tests**:
```bash
   cargo test
   cargo nextest run --workspace
```

## Before You Start

- **Search existing issues and PRs** to avoid duplicates
- **Open an issue first** for major changes to discuss your approach
- **Ask questions** on our [Telegram group](https://t.me/+Ig4WDWOzikA3MzA0)

## Development Guidelines

### Code Standards

Before submitting your changes, ensure you:

1. **Format your code**:
```bash
   cargo fmt --all
```

2. **Check for linting issues**:
```bash
   cargo clippy --workspace --all-targets --all-features
```

3. **Run the typo checker**:
```bash
   typos
```

4. **Run all tests**:
```bash
   cargo nextest run --workspace
   cargo nextest run --workspace --no-default-features
   cargo nextest run --workspace --all-features
```

5. **Test no_std builds** (if relevant):
```bash
   cargo check --target riscv32imac-unknown-none-elf --no-default-features
   cargo check --target riscv64imac-unknown-none-elf --no-default-features
```

### Code Style

- Follow Rust naming conventions
- Write clear, self-documenting code
- Add comments for complex logic
- Include doc comments for public APIs
- Keep functions focused and concise

### Testing

- Add unit tests for new functionality
- Add integration tests where appropriate
- Ensure all existing tests pass
- Aim to maintain or improve code coverage

### Documentation

- Update relevant documentation for changes
- Add examples for new features
- Update the CHANGELOG.md if adding features or fixing bugs
- Keep doc comments up to date

## Pull Request Process

1. **Create a feature branch**:
```bash
   git checkout -b feature/your-feature-name
```

2. **Make your changes** following the guidelines above

3. **Commit your changes**:
```bash
   git add .
   git commit -m "feat: brief description of changes"
```
   
   Use [conventional commit messages](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions/changes
   - `refactor:` for code refactoring
   - `perf:` for performance improvements
   - `chore:` for maintenance tasks

4. **Push to your fork**:
```bash
   git push origin feature/your-feature-name
```

5. **Open a Pull Request** on GitHub with:
   - Clear title and description
   - Reference to related issues (if any)
   - Screenshots/examples (if applicable)
   - List of changes made

6. **Respond to review feedback** promptly and make requested changes

## Project Structure

Understanding the codebase structure:

- `crates/revm`: Main crate with re-exports
- `crates/primitives`: Primitive types and constants
- `crates/bytecode`: Bytecode analysis, EOF validation, opcode tables
- `crates/interpreter`: Opcode execution and interpreter internals
- `crates/context-interface`: Context, environment, journal, frame stack traits
- `crates/context`: Default context, journal, and `Evm` container
- `crates/handler`: Mainnet execution flow, frames, validation, APIs
- `crates/database-interface`: Database traits
- `crates/database`: Database implementations
- `crates/state`: Account/storage/state types
- `crates/precompile`: Precompiled contracts
- `crates/inspector`: Tracing and inspector APIs
- `crates/statetest-types`: Ethereum state test types
- `bins/revme`: CLI for tests and validation
- `examples`: API usage examples
- `book/src`: Documentation

## Resources

- **Documentation**: [https://bluealloy.github.io/revm/](https://bluealloy.github.io/revm/)
- **Code Documentation**: [https://bluealloy.github.io/revm/docs/revm/](https://bluealloy.github.io/revm/docs/revm/)
- **Telegram**: [https://t.me/+Ig4WDWOzikA3MzA0](https://t.me/+Ig4WDWOzikA3MzA0)
- **Issues**: [GitHub Issues](https://github.com/bluealloy/revm/issues)

## Code of Conduct

We expect all contributors to:

- Be respectful and inclusive
- Welcome newcomers
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards others

## Questions?

If you have questions about contributing:

- Open a [GitHub Discussion](https://github.com/bluealloy/revm/discussions)
- Ask on [Telegram](https://t.me/+Ig4WDWOzikA3MzA0)
- Email: [dragan0rakita@gmail.com](mailto:dragan0rakita@gmail.com)

## Security

For security-related issues, please see our [Security Policy](README.md#security) and contact [dragan0rakita@gmail.com](mailto:dragan0rakita@gmail.com) directly.

## License

By contributing to Revm, you agree that your contributions will be licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

Thank you for contributing to Revm! 🚀
