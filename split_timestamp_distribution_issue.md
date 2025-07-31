## Title: Splits span wide time ranges when backfilling historical data from multiple time periods

### Description

When backfilling logs from various time periods simultaneously, splits are not well-distributed by timestamp. Most splits end up spanning nearly the entire time range instead of being partitioned into smaller time windows. This leads to inefficient time-based pruning during queries.

### Current Behavior

1. **Split creation is based on document count/size, not time windows**:
   - Splits are created when reaching `split_num_docs_target` (default: ~10M docs)
   - Or when memory limit (`heap_size`) is reached
   - No time-window based split creation logic exists

2. **Timestamp range expansion**:
   - The `record_timestamp` function (`indexer.rs:248-254`) expands each split's time range to include min/max of all documents
   - When backfilling mixed-time logs, a single split can span days/weeks/months

3. **Merge policy assumes time-ordered input**:
   - `StableLogMergePolicy` is optimized for time-ordered log streams
   - Sorts splits by reverse time order before merging
   - Not optimal for out-of-order backfill scenarios

### Expected Behavior

Splits should have bounded time ranges even during backfill operations, enabling efficient time-based pruning during queries.

### Reproduction Steps

1. Set up a Quickwit index with timestamp field
2. Backfill historical logs from multiple time periods (e.g., logs from different days/weeks)
3. Observe split metadata - most splits will have very wide time ranges

### Code Analysis

Key code locations:
- Split creation: `quickwit-indexing/src/actors/indexer.rs:460-470`
- Timestamp recording: `quickwit-indexing/src/actors/indexer.rs:248-254`
- Split attributes: `quickwit-indexing/src/models/split_attrs.rs`
- Merge policy: `quickwit-indexing/src/merge_policy/stable_log_merge_policy.rs`

### Proposed Solutions

1. **Time-based split boundaries**:
   - Add configuration for maximum time window per split
   - Create new split when document timestamp exceeds current split's time window
   - Consider adding `max_time_range_secs` to `IndexingSettings`

2. **Backfill mode**:
   - Add special indexing mode for historical data ingestion
   - Buffer and sort documents by timestamp before indexing
   - Create splits with non-overlapping time ranges

3. **Time-based partitioning**:
   - Extend current partition concept (currently based on fields like tenant_id)
   - Add automatic time-based partitioning option
   - Route documents to splits based on time buckets

### Impact

- Query performance: Poor time-based pruning leads to scanning more data than necessary
- Storage efficiency: Merging splits with wide time ranges reduces effectiveness of time-based retention
- Resource usage: More splits need to be opened during time-range queries

### Environment

- Quickwit version: main branch
- Use case: Backfilling historical logs/metrics data

### Related Issues

- **#5848**: "Janitor is deleting splits before the retention period expires" - This critical bug highlighted how wide time range splits can cause data loss issues
- **#5850**: "fix: Janitor list splits query" - The fix ensures proper retention handling, but wide time range splits still cause inefficiencies

#### Impact of wide time range splits:

**Before PR #5850 (with bug):**
- Splits containing both old and recent data were deleted prematurely
- Data loss occurred when any document in a split exceeded retention period
- Example: A split with data from 2023-2025 would be entirely deleted once 2023 data expired

**After PR #5850 (bug fixed):**
- Splits are retained correctly but storage efficiency suffers
- Wide time range splits cannot be deleted until ALL data expires
- Query performance degraded as time-based pruning becomes less effective
- Example: A split with 99% expired data and 1% recent data must be kept entirely

This makes proper time-based split partitioning even more important for both data retention correctness and system efficiency.