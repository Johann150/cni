# cni_format

This is a parser and serializer library for the [CNI configuration format](https://github.com/libuconf/cni/), compatible with version 0.1.0. It also provides the recommended API functions.

This crate is dependency-free (except for testing).

## Reference Compliance

`ini` and all `ext` elements have a feature flag by the same name. They are enabled by default where indicated.

- `core`: 29/29
- `ini`: fully compliant (default)
- `ext`: more-keys (not default)
