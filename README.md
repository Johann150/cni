# cni_format

This is a parser and serializer library for the [CNI configuration format](https://github.com/libuconf/cni/), compatible with version 0.1.0. It also provides the recommended API functions.

This crate is dependency-free (except for testing).

The recommended API and serializer can be en-/disabled with the feature flags `api` or `serializer` respectively. Only the API is enabled by default to speed up compilation.

## Reference Compliance

`ini` and all `ext` elements have a feature flag by the same name. Note that nothing outside of `core` is enabled by default, but can be enabled by the respective feature flags.

- `core`: 29/29
- `ini`: fully compliant
- `ext`: more-keys
