## 0.2.0

### Breaking

- `ExtractMap::default` is now implemented for all build hashers, not just `RandomState`

### Added

- `ExtractMap::allocation_size` has been added to return the allocated size behind the internal `HashTable`
- `typesize::TypeSize` is now implemented behind a feature gate

### Internals

- `ExtractMap` is now based internally on `HashTable`, instead of `HashSet`

## 0.1.2

- Simplifed `serde::Deserialize` implementation for `ExtractMap`

## 0.1.1

- Documented MSRV as 1.70
- Added documentation for methods missing it
- Added Clone for Iter
- Added Debug implementation for Iter, IterMut, IntoIter
