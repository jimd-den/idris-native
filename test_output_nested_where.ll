target triple = "x86_64-pc-linux-gnu"

declare i32 @puts(i8*)
declare i8* @malloc(i64)
declare void @free(i8*)
declare i64 @strlen(i8*)
declare i8* @memcpy(i8*, i8*, i64)

@str_buffer = global [1024 x i8] zeroinitializer

define i64 @getLine() {
  %ptr = getelementptr [1024 x i8], [1024 x i8]* @str_buffer, i64 0, i64 0
  ; Simplified: return the global buffer pointer
  %res = ptrtoint i8* %ptr to i64
  ret i64 %res
}

define void @putStr(i64 %s_int) {
  %s = inttoptr i64 %s_int to i8*
  %void = call i32 @puts(i8* %s)
  ret void
}

define void @putStrLn(i64 %s_int) {
  %s = inttoptr i64 %s_int to i8*
  %void = call i32 @puts(i8* %s)
  ret void
}

define i64 @concat(i64 %s1_int, i64 %s2_int) {
entry:
  %s1 = inttoptr i64 %s1_int to i8*
  %s2 = inttoptr i64 %s2_int to i8*
  %len1 = call i64 @strlen(i8* %s1)
  %len2 = call i64 @strlen(i8* %s2)
  %total_len = add i64 %len1, %len2
  %alloc_len = add i64 %total_len, 1
  %new_str = call i8* @malloc(i64 %alloc_len)
  
  ; Copy first string
  %void1 = call i8* @memcpy(i8* %new_str, i8* %s1, i64 %len1)
  
  ; Copy second string
  %dest2 = getelementptr i8, i8* %new_str, i64 %len1
  %void2 = call i8* @memcpy(i8* %dest2, i8* %s2, i64 %len2)
  
  ; Null terminator
  %term_ptr = getelementptr i8, i8* %new_str, i64 %total_len
  store i8 0, i8* %term_ptr
  
  %res = ptrtoint i8* %new_str to i64
  ret i64 %res
}

define void @print_int(i64 %n) {
entry:
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %zero, label %not_zero

zero:
  %char_zero = add i8 48, 0
  %buf_zero = alloca i8, i32 2
  store i8 %char_zero, i8* %buf_zero
  %next_zero = getelementptr i8, i8* %buf_zero, i32 1
  store i8 10, i8* %next_zero
  %void_zero = call i32 @write(i32 1, i8* %buf_zero, i32 2)
  ret void

not_zero:
  %abs_n = call i64 @llvm.abs.i64(i64 %n, i1 true)
  %is_neg = icmp slt i64 %n, 0
  br i1 %is_neg, label %print_minus, label %convert

print_minus:
  %minus_sign = alloca i8
  store i8 45, i8* %minus_sign
  %void_minus = call i32 @write(i32 1, i8* %minus_sign, i32 1)
  br label %convert

convert:
  %buf = alloca i8, i32 21
  %end_ptr = getelementptr i8, i8* %buf, i32 20
  store i8 10, i8* %end_ptr
  %res_ptr = call i8* @int_to_str(i64 %abs_n, i8* %end_ptr)
  %len = ptrtoint i8* %end_ptr to i64
  %start = ptrtoint i8* %res_ptr to i64
  %msg_len = sub i64 %len, %start
  %final_len = add i64 %msg_len, 1
  %len_i32 = trunc i64 %final_len to i32
  %void_final = call i32 @write(i32 1, i8* %res_ptr, i32 %len_i32)
  ret void
}

declare i32 @write(i32, i8*, i32)
declare i64 @llvm.abs.i64(i64, i1)

define i8* @int_to_str(i64 %n, i8* %buf) {
entry:
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %done, label %loop

loop:
  %curr_n = phi i64 [ %n, %entry ], [ %next_n, %loop ]
  %curr_ptr = phi i8* [ %buf, %entry ], [ %next_ptr, %loop ]
  %rem = urem i64 %curr_n, 10
  %next_n = udiv i64 %curr_n, 10
  %char = trunc i64 %rem to i8
  %char_val = add i8 %char, 48
  %next_ptr = getelementptr i8, i8* %curr_ptr, i32 -1
  store i8 %char_val, i8* %next_ptr
  %loop_cond = icmp eq i64 %next_n, 0
  br i1 %loop_cond, label %done, label %loop

done:
  %final_ptr = phi i8* [ %buf, %entry ], [ %next_ptr, %loop ]
  ret i8* %final_ptr
}
define i64 @"z"(i64 %w) {
  %2 = add i64 %y, %w
  ret i64 %2
}

define i64 @"c"(i64 %x) {
  %1 = add i64 42, %x
  ret i64 %1
}

define i64 @"d"(i64 %y) {
  %7 = add i64 %y, 1
  %8 = call i64 @"z"(i64 %y)
  %9 = add i64 %7, %8
  %10 = call i64 @"c"(i64 %9)
  ret i64 %10
}

define i64 @"main"() {
  %3 = inttoptr i64 %d to i64 (i64 )*
  %4 = call i64 %3(i64 2)
  %5 = inttoptr i64 %print to i64 (i64 )*
  %6 = call i64 %5(i64 %4)
  ret i64 %6
}


define i32 @main() {
  %res = call i64 @"d"(i64 2)
  call void @print_int(i64 %res)
  ret i32 0
}
