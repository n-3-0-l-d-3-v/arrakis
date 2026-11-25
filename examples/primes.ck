// expect: 1229
// How many primes are below 10,000? Trial division with an early exit,
// a `bool`-returning helper and short-circuit `and`.

fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    } else {
        let d = 2;
        let prime = true;
        while prime and d * d <= n {
            if n % d == 0 {
                prime = false;
            } else {
                d = d + 1;
            }
        }
        return prime;
    }
}

fn main() -> i64 {
    let count = 0;
    let n = 2;
    while n < 10000 {
        if is_prime(n) {
            count = count + 1;
        } else {
            count = count;
        }
        n = n + 1;
    }
    return count;
}
