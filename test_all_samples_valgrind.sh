#!/bin/bash
COMPILER="./target/debug/idris_native"
SAMPLES_DIR="idris2_ref/samples"

cargo build --quiet

echo "IDRIS NATIVE - SAMPLE EXECUTION & VALGRIND REPORT"
echo "================================================="

for f in $SAMPLES_DIR/*.idr; do
    name=$(basename "$f")
    echo -n "Testing $name... "
    
    # Try to compile
    $COMPILER "$f" --no-qtt > /dev/null 2>&1
    
    if [ $? -eq 0 ]; then
        echo "COMPILE SUCCESS"
        bin="${f%.idr}_bin"
        if [ -f "$bin" ]; then
            echo "--- VALGRIND OUTPUT ---"
            valgrind --leak-check=full --error-exitcode=1 "$bin" 2>&1 | grep -E "ERROR SUMMARY|definitely lost|segfault|Segmentation fault"
            
            # Also show the actual output if it doesn't crash
            echo "--- PROGRAM OUTPUT ---"
            timeout 2s "$bin"
            echo "-----------------------"
            rm "$bin"
        else
            echo "ERROR: Binary not found after success"
        fi
    else
        echo "COMPILE FAILURE"
    fi
    echo ""
done
