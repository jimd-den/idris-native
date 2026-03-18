# Hypothesis: Syntax Parser Consumption Error

## Context
The `Parser` is failing to correctly handle the transition between a type signature and a function definition in `ackermann.idr`. 

## Hypothesis
The `parse_pi` method (or a method it calls) is consuming tokens that belong to the subsequent function definition (specifically the function name `ack`), leading to a "Expected identifier in definition, got Assign" error when `parse_def` attempts to start.

## Proof Strategy
1. **Test 1: Signature Boundary Verification**
   - Create a test that parses only a type signature and verifies that the `current` token index is exactly at the end of the signature.
2. **Test 2: Definition Start Verification**
   - Create a test that parses a signature and then immediately checks if the next token is the expected function name for the definition.
3. **Test 3: Pi-Term Stop Condition**
   - Verify if `parse_expr` (called by `parse_pi`) is over-consuming tokens (e.g., greedily consuming the next identifier).

## Expected Result
If the hypothesis is correct, Test 1 or 2 will fail by showing that the parser's internal index has advanced past the signature into the definition name.
