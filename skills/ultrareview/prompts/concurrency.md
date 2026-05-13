# Dimension: Concurrency

You are looking for **concurrency and synchronization bugs**.

## What to look for

- **Race conditions** -- shared mutable state accessed
  without synchronization, data races.
- **TOCTOU (time-of-check-time-of-use)** -- a value is checked
  then used later without holding the relevant lock,
  allowing another thread to change it between check and use.
- **Deadlocks** -- lock ordering violations,
  holding lock A while waiting for lock B
  when another thread holds B and waits for A.
- **Missing synchronization** -- atomic operations that should be
  combined but aren't, missing memory barriers,
  incorrect `Ordering` on atomics.
- **Incorrect atomic ordering** -- using `Relaxed` where
  `Acquire`/`Release` or `SeqCst` is needed.
- **Shared mutable state without protection** -- `Rc` where `Arc`
  is needed, `RefCell` across threads, `unsafe Send/Sync` impls
  that are incorrect.
- **Async hazards** -- holding a mutex guard across `.await`,
  blocking in async context, `Send` bound violations.

## What to ignore

- Single-threaded code with no async and no shared state.
- Performance implications of synchronization choices
  (another reviewer covers this).
