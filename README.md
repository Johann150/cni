# cni_format

This is a parser and serializer library for the [CNI configuration format](https://github.com/libuconf/cni/), compatible with version 0.1.0. It also provides the recommended API functions.

This crate is dependency-free (except for testing).

The recommended API and serializer can be en-/disabled with the feature flags `api` or `serializer` respectively. Only the API is enabled by default to speed up compilation.

## Reference Compliance

`ini` and all `ext` elements have a feature flag by the same name. Note that nothing outside of `core` is enabled by default, but can be enabled by the respective feature flags.

- `core`: 29/29
- `ini`: fully compliant
- `ext`: more-keys

## tooling

There are some helpful tools included with the library as examples of how the library can be used. These can be run with e.g. `cargo run --example dumper -- src/test/cni/tests/core/key/01.cni`.

Note that some of the examples require you to enable features when compiling. For example like this `cargo run --example formatter --feature serializer -- src/test/cni/tests/core/key/01.cni`.

The linter is implemented completely separately from the library and can be found as a subcrate in the `linter` directory. By switching to that directory, you can use cargo on it normally.
