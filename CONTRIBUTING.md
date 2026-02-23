# Contributing to cambi

Thanks for contributing.

## Code style rules

### Declaration order (required)

To keep the code easier to scan, avoid forward references:

1. **Types first**: if a struct/enum uses another type, declare the referenced type **before** the user type.
2. **Functions next**: if a function calls another function in the same module, place the callee **before** the caller.

In short: define dependencies first, dependents after.

## Development checks

Before opening a PR, run:

```sh
cargo fmt
cargo test
```
