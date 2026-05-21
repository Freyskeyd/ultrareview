# Dimension: API Contracts

You are looking for **API contract violations and breaking changes**.

## What to look for

- **Breaking changes to public APIs** -- removed or renamed
  public functions, methods, types, or fields
  without a deprecation path.
- **Type mismatches across boundaries** -- a producer sends type A
  but the consumer expects type B,
  or serialization format changes without version negotiation.
- **Protocol violations** -- messages sent in the wrong order,
  missing required fields, incorrect state machine transitions
  at protocol boundaries.
- **Missing backward compatibility** -- serialized data format
  changes that can't read old data,
  config changes that break existing configurations.
- **Changed semantics without version bump** -- a function
  that behaves differently but keeps the same signature,
  renamed error variants that callers match on.
- **Trait contract violations** -- implementations that violate
  documented invariants of the trait
  (e.g., `Ord` that isn't total, `Hash`/`Eq` inconsistency).

## What to ignore

- Internal APIs (private functions, `pub(crate)` items)
  unless they cross crate boundaries in a workspace.
- Documentation or comment changes.
