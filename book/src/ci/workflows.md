# CI Workflows

## Automated Quality Gates

Roxi uses GitHub Actions to run unit tests and the five dialect conformance suites on every push and pull request.

---

## Conformance Configuration

The workflow is configured in `.github/workflows/conformance.yml`:

```yaml
name: Conformance Gate
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        run: rustup update stable
      - name: Run Conformance Suites
        run: |
          cargo test --test sparql11_conformance
          cargo test --test datalog_conformance
          cargo test --test n3_conformance
          cargo test --test shacl_conformance
          cargo test --test shex_conformance
```
