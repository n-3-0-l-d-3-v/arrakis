---
status: done
phase: 2
---

# 014 — Prefix scan inside a transaction

`TransactionalStore`/`Transaction` exposed get/put/delete only; the raw
`Store` had `scan`/`scan_at`. A downstream consumer (choam's SQL layer)
needs table scans that are snapshot-isolated and see their own buffered
writes.

## Done
- [x] `Transaction::scan(prefix)`: snapshot view (`scan_at`) merged with the transaction's own buffered puts/deletes, in key order
- [x] Unit tests: merge with own writes/deletes; scan pinned to snapshot
- [x] Mutation-checked: ignoring buffered deletes fails the merge test
- [x] Additive only: no existing API or behaviour changed
