# Hypothesis: QTT Linear Usage Violation

## Context
The mandatory QTT multiplicity check is failing for both `ackermann_test.idr` and `sha256_test.idr`, stating that linear variables (like `m` and `a`) are "used incorrectly".

## Hypothesis
The current `QttChecker::count_usage` implementation is too strict for certain Idris 2 patterns. Specifically:
1. **Recursion/Calls**: A variable passed as an argument to a function call (including recursive calls) is being counted as a "usage", but if it's used in multiple recursive branches or sub-expressions of a call, it might be exceeding the limit of 1.
2. **Control Flow (If/Case)**: For linear variables (multiplicity 1), they must be used exactly once *overall*. However, in a branching structure like `if/then/else`, a linear variable should be allowed to be used once in *each* branch, as only one branch will actually execute. Our current `count_usage` for `If` simply adds the usages of the branches together (`then + else`), which is incorrect for QTT.

## Proof Strategy
1. **Test 1: Linear Variable in If-Branches**
   - Create a test case where a linear variable is used once in the `then` branch and once in the `else` branch.
   - Verify if `check_multiplicity` returns `false` (it should return `true` for a correct QTT implementation).
2. **Test 2: Variable in Recursive Call**
   - Verify how usages are counted in `ack m n = ack (m-1) n`.

## Expected Result
Test 1 will prove that `count_usage` incorrectly sums usages across mutual exclusive branches, which is the root cause of the `ack` and `sha256_verify` failures.
