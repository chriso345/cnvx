# Tests

This test suite light provides integration and smoke tests for the cnvx library.

Its purpose is to ensure that the system builds and runs end-to-end without major errors, including:
- compiling vendored utilities
- processing Netlib LP benchmarks
- running the solver pipeline successfully

These tests are not exhaustive and do not aim to fully validate numerical correctness or algorithmic completeness. Instead, they are designed to catch regressions and major integration failures.
