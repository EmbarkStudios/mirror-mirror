# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- **added:** Add `fields_len` methods to the following types ([#94])
    - `StructType`
    - `TupleStructType`
    - `TupleType`
    - `Variant`
    - `StructVariant`
    - `TupleStructVariant`
- **added:** Add `EnumType::variants_len` ([#94])
- **added:** Support pretty printing types ([#95])

[#94]: https://github.com/EmbarkStudios/mirror-mirror/pull/94
[#95]: https://github.com/EmbarkStudios/mirror-mirror/pull/95

# 0.1.4 (08. February, 2023)

- **added:** Implement `Hash` to `Value` ([#93])

[#93]: https://github.com/EmbarkStudios/mirror-mirror/pull/93

# 0.1.3 (08. February, 2023)

- **fixed:** Make `SimpleTypeName` support types defined inside unnamed constants ([#91])

[#91]: https://github.com/EmbarkStudios/mirror-mirror/pull/91

# 0.1.2 (03. February, 2023)

- **added:** Add `impl From<Key> for KeyPath` ([#88])

[#88]: https://github.com/EmbarkStudios/mirror-mirror/pull/88

# 0.1.1 (17. January, 2023)

- **added:** Add `Reflect` impls for [`glam`] types ([#85])
- **added:** Add `Reflect` impls for [`macaw`] types ([#85])

[#85]: https://github.com/EmbarkStudios/mirror-mirror/pull/85
[`glam`]: https://crates.io/crates/glam
[`macaw`]: https://crates.io/crates/macaw

# 0.1.0 (12. January, 2023)

- Initial release.
