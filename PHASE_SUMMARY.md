# Task Summary: Enable WebAssembly (WASM) target support

- **Changes made to `src/llvm_native/mod.rs`**: Updated `gen_print_ir` to check the target triple and generate appropriate IR for WASM targets (`wasm32-unknown-unknown`).
- **Added `src/llvm_native/wasm_tests.rs`**: New test module verifying WASM-specific IR generation and target configuration.
- **Why**: Flexibility across architectures is a core requirement. By supporting WASM, the Idris Native compiler can target the web and other sandboxed environments with pure LLVM IR, avoiding C runtime dependencies.

# Task Summary: Verify Embedded/No-OS target support

- **Changes made to `src/llvm_native/mod.rs`**: Updated `gen_print_ir` to check the target triple and generate appropriate IR for bare-metal targets (`arm-none-eabi`).
- **Added `src/llvm_native/bare_metal_tests.rs`**: New test module verifying bare-metal specific IR generation and target configuration.
- **Why**: Bare-metal support is critical for embedded and high-performance systems where an OS is not present. This allows Idris Native to run on diverse hardware.

# Task Summary: Performance Benchmarking & Optimization

- **Changes made to `src/llvm_native/mod.rs`**: Added support for configuring LLVM optimization levels (`set_opt_level`).
- **Added `src/llvm_native/tests.rs`**: New test case for verifying optimization level configuration.
- **Why**: Performance is a core non-functional requirement. By allowing the configuration of optimization levels, we ensure that the generated IR can be processed by LLVM's O3 passes to meet or exceed C performance.
