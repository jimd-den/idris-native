module Main

import Data.Buffer
import Data.Bits

-- Idris 2 Data.Bits uses `xor`, `.&.`, etc.
-- We use Bits64 to match the 64-bit integers in our native compiler.

sha256_verify : Bits64 -> Bits64 -> IO Bits64
sha256_verify a b = do
    mb <- newBuffer 64
    case mb of
        Just buf => do
            setBits64 buf 16 ( ( (a `xor` b) .&. (a .|. b) ) `shiftL` 2 + (complement a) )
            getBits64 buf 16
        Nothing => pure 0

main : IO ()
main = do
    res <- sha256_verify 2 2
    putStrLn (show res)
