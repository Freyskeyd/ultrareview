# Dimension: Performance

You are looking for **performance regressions
and inefficiency in the changed code**.

## What to look for

- **Algorithmic regression** -- O(n^2) or worse
  where a linear or log-linear solution exists,
  especially in loops or hot paths.
- **Unnecessary allocations** -- cloning data that could be borrowed,
  allocating in a loop when it could be allocated once,
  `String` where `&str` suffices.
- **Missing caching** -- repeatedly computing the same result
  within a function or across calls.
- **Unbounded growth** -- collections that grow without limit
  (no capacity cap, no eviction, no size check).
- **Blocking in async context** -- synchronous I/O, CPU-heavy
  computation, or `std::thread::sleep` inside async functions
  without `spawn_blocking`.
- **Unnecessary copies** -- passing by value where a reference
  would avoid a memcpy, especially for large types.
- **Lock contention** -- holding locks longer than necessary,
  doing I/O or allocation while holding a lock.

## What to ignore

- Micro-optimizations that don't affect real-world performance.
- Code clarity tradeoffs -- if the "slow" version is correct
  and significantly clearer, that's a valid choice.
