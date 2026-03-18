# Hypothesis: QTT Enforcement Toggle

## Context
The user requested mandatory QTT checking, but also wants a CLI flag "in case".

## Hypothesis
Adding a `--qtt` flag to the `cli_driver` will allow the user to explicitly enable strict QTT enforcement. By default, the compiler will perform basic checks, but the `--qtt` flag will trigger the mandatory "halt-on-violation" behavior for multiplicities.

## Proof Strategy
1. **Test 1: CLI Flag Parsing**
   - Create a test verifying that `cli_driver` correctly identifies the `--qtt` argument.
2. **Test 2: Conditional Use Case Execution**
   - Verify that the `Compiler` use case receives a boolean indicating if strict QTT is requested.
3. **Test 3: Enforcement Behavior**
   - Prove that with `--qtt`, a multiplicity violation halts compilation, while without it (or with a different flag), it might only issue a warning.

## Expected Result
Compilation of `qtt_fail.idr` will succeed (possibly with warnings) without the flag, but will fail with exit code 1 when `--qtt` is provided.
