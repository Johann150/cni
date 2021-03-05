# cni_format

This is a parser and serializer library for the [CNI configuration format](https://github.com/libuconf/cni/). It also provides the recommended API functions.

This crate is dependency-free (except for testing).

## Reference Compliance
Please note that the specification for the CNI format has not yet been stabilized.

`ini` and all `ext` elements have a feature flag by the same name. They are enabled by default where indicated.

- `core`: 34/34
- `ini`: fully compliant (default)
- `ext`: more-keys (not default)
