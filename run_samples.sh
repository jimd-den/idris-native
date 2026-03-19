#!/bin/bash
cargo build --quiet
echo "Testing Prims.idr..."
./target/debug/idris_native idris2_ref/samples/Prims.idr --no-qtt
if [ $? -eq 0 ]; then
    echo "SUCCESS: Prims.idr compiled."
else
    echo "FAILURE: Prims.idr failed to compile."
fi
