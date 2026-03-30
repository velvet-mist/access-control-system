# Contributing

## Code Style
- **Python**: Black (`pip install black`), isort, mypy. Docstrings: Google style.
- **Rust**: rustfmt (`cargo fmt`), clippy (`cargo clippy`).

## Development Workflow
1. Branch: `feat/doc-api` etc.
2. Install deps:
   - Python: `cd controller && pip install -r requirements.txt -e .`
   - Rust: `cd rust-adapter && cargo build`
3. Test:
   ```bash
   pytest controller/  # Add tests/
   cargo test
   ```
4. Docs:
   - Python: Sphinx (`pip install sphinx sphinx-rtd-theme`, `sphinx-quickstart`, `make html`)
   - Rust: `cargo doc --open`
5. Lint/Docs:
   ```bash
   black .
   cargo fmt
   cargo clippy
   ```

## PR Checklist
- [ ] Tests pass
- [ ] Linting/docs updated
- [ ] Docker builds/runs
- No secrets in code

## Releases
- Bump versions in Cargo.toml/pyproject.toml
- `docker compose build`
- Tag: `git tag v0.1.0`

Questions? Open issue.

