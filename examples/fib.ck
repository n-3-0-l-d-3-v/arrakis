// expect: 6765
// Recursive Fibonacci: about 22,000 calls, every one a real `call`/`ret`
// on mentat with its own stack frame. fib(20) = 6765.

fn fib(n: i64) -> i64 {
    if n < 2 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

fn main() -> i64 {
    return fib(20);
}
