sha256_verify a b = 
    let state = buffer 8 in
    let block = buffer 64 in
    -- Initial state
    let s0 = setBits64 state 0 100 in
    let s1 = setBits64 state 1 200 in
    -- Bitwise manipulation
    let val = ( a `xor` b ) .&. ( a .|. b ) in
    let shifted = val `shiftL` 2 in
    let combined = shifted + ( complement a ) in
    -- Buffer store/load
    let st = setBits64 state 2 combined in
    getBits64 state 2
