# Changelog

## [0.1.1] - 2025-04-26
### Added
- `fs::absolute`, which calls `GetFullPathNameW` on all versions of Windows and does nothing if the path is already
  absolute

### Changed
- Deleting directories now passes an absolute path into Win32 for thread safety.
- Pre-Vista: `fs::canonicalize` now calls `fs::absolute` but still checks if the file exists. The underlying behavior is
  still mostly the same, as it still calls `GetFullPathNameW`, with the difference being that already-absolute paths are
  returned without running `GetFullPathNameW`. This is not considered a breaking change.

[0.1.1]: https://github.com/FishAndRips/minxp/commit/2ec73a6184a8b9f736ddf58cbd2b483df629f7cd

## [0.1.0] - 2025-04-22
### Added
- Initial release
- Increased coverage (ignoring non-core/alloc):
  - env (~80%, was incorrectly marked as 100%)
  - ffi (~80%)
  - fs (~80%)
  - io (~30%)
  - path (100%)

[0.1.0]: https://github.com/FishAndRips/minxp/commit/531431ea98433689a5c4aa3229a298b713d52902
