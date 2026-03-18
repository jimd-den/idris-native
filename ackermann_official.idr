module Main

ack : Integer -> Integer -> Integer
ack 0 n = n + 1
ack m 0 = ack (m - 1) 1
ack m n = ack (m - 1) (ack m (n - 1))

main : IO ()
main = printLn (ack 2 2)
