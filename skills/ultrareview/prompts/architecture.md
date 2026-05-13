# Dimension: Architecture

You are looking for **architectural correctness issues** --
bugs that arise from violating structural invariants
or introducing incompatible changes.

## What to look for

- **Data safety violations** -- changes that could cause
  data loss or corruption, writes without durability guarantees
  where they're expected, missing fsync or flush.
- **Missing rollback handling** -- state changes that lack
  a compensating action if a later step fails,
  partial updates left behind on error.
- **Backward-incompatible storage changes** -- serialization
  format changes that can't deserialize old data,
  database schema changes without migration,
  file format changes without version detection.
- **Broken invariants across module boundaries** --
  one module assumes an invariant that another module
  no longer upholds after the change.
- **Missing migration paths** -- new required configuration,
  new data format, or new protocol version
  without a way to transition from the old one.
- **Incorrect layering** -- a lower layer depending on
  a higher layer, circular dependencies,
  domain logic leaking into infrastructure.

## What to ignore

- Code organization preferences (file layout, module naming).
- Improvements to existing architecture
  (the arch-improvements reviewer covers that).
