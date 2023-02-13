# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- **added:** Implement `Hash` to `Value`

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
