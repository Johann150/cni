# cni_format

This is a parser and serializer library for the [CNI configuration format](https://github.com/libuconf/cni/), compatible with version 0.1.0. It also provides the recommended API functions.

This crate is dependency-free (except for testing).

The recommended API and serializer are enabled by default, but can be disabled with the feature flags `api` or `serializer` respectively. This may speed up compilation speeds.

## Reference Compliance

`ini` and all `ext` elements have a feature flag by the same name. They are enabled by default where indicated.

- `core`: 29/29
- `ini`: fully compliant (default)
- `ext`: more-keys (not default)
