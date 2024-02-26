# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- None

# 0.1.19 (26. February, 2023)

- Allow a `glam` version range between `0.22` and `0.25`, inclusive.

# 0.1.18 (22. February, 2023)

- Update internal dependencies

# 0.1.17 (23. January, 2023)

- Update internal dependencies

# 0.1.16 (11. September, 2023)

- **added:** Add `reflect_eq` ([#126])

[#126]: https://github.com/EmbarkStudios/mirror-mirror/pull/126

# 0.1.15 (08. August, 2023)

- **added:** Implement serialization traits for `ScalarOwned` ([#117])
- **fixed:** Use `core::any::type_name` to determine `type_name` for `ScalarType` variants ([#124])

[#124]: https://github.com/EmbarkStudios/mirror-mirror/pull/124
[#117]: https://github.com/EmbarkStudios/mirror-mirror/pull/117

# 0.1.14 (11. April, 2023)

- **change:** Make use of deterministic hashes for `NodeId` and `TypeDescriptor` ([#115])

[#115]: https://github.com/EmbarkStudios/mirror-mirror/pull/115

# 0.1.13 (29. March, 2023)

- **fixed:** Require less strict `speedy` version ([#114])

[#114]: https://github.com/EmbarkStudios/mirror-mirror/pull/114

# 0.1.12 (21. March, 2023)

- **change:** Update to syn 2.0 ([#113])

[#113]: https://github.com/EmbarkStudios/mirror-mirror/pull/113

# 0.1.11 (20. March, 2023)

- **added:** Implement `Reflect`, and friends, for `Infallible` ([#111])
- **change:** Update to syn 2.0 ([#112])

[#111]: https://github.com/EmbarkStudios/mirror-mirror/pull/111
[#112]: https://github.com/EmbarkStudios/mirror-mirror/pull/112

# 0.1.10 (03. March, 2023)

- **fixed:** Fully qualify `FromReflect` in generated code ([#107])

[#107]: https://github.com/EmbarkStudios/mirror-mirror/pull/107

# 0.1.9 (24. February, 2023)

- **added:** Add `StructValue::with_capacity`,
  `TupleStructValue::with_capacity`, and `TupleValue::with_capacity` ([#106])
- **added:** Add `EnumValue::new_struct_variant_with_capacity` and
  `EnumValue::new_struct_variant_with_capacity` constructors ([#106])
- **fixed:** In `Reflect::to_value` for enums, only generate a catch all branch
  if the enum has a variant with `#[reflect(skip)]` ([#105])
- **added:** Add a `has_default_value` method to types in `type_info` ([#104])

[#105]: https://github.com/EmbarkStudios/mirror-mirror/pull/105
[#104]: https://github.com/EmbarkStudios/mirror-mirror/pull/104
[#106]: https://github.com/EmbarkStudios/mirror-mirror/pull/106

# 0.1.8 (23. February, 2023)

- **added:** Make `Key` and `KeyPath` impl `Hash` ([#103])

[#103]: https://github.com/EmbarkStudios/mirror-mirror/pull/103

# 0.1.7 (23. February, 2023)

- **added:** Implement `PartialEq`, `Eq`, `Hash` for types in `type_info` ([#100] [#101])

[#100]: https://github.com/EmbarkStudios/mirror-mirror/pull/100
[#101]: https://github.com/EmbarkStudios/mirror-mirror/pull/101

# 0.1.6 (16. February, 2023)

- **fixed:** Fix inconsistent ordering when iterating over fields in struct
  values and struct types. Same for struct variants ([#98])

[#98]: https://github.com/EmbarkStudios/mirror-mirror/pull/98

# 0.1.5 (14. February, 2023)

- **added:** Add visitor API ([#92])
- **added:** Add `fields_len` methods to the following types ([#94])
    - `StructType`
    - `TupleStructType`
    - `TupleType`
    - `Variant`
    - `StructVariant`
    - `TupleStructVariant`
- **added:** Add `EnumType::variants_len` ([#94])
- **added:** Support setting a default value for `OpaqueType` ([#97])
- **added:** Support pretty printing types ([#95])

[#92]: https://github.com/EmbarkStudios/mirror-mirror/pull/92
[#94]: https://github.com/EmbarkStudios/mirror-mirror/pull/94
[#95]: https://github.com/EmbarkStudios/mirror-mirror/pull/95
[#97]: https://github.com/EmbarkStudios/mirror-mirror/pull/97

# 0.1.4 (13. February, 2023)

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
