// expect: 232792560
// The smallest number divisible by every integer 1..20, via Euclid's
// algorithm: lcm(a, b) = a / gcd(a, b) * b. Exercises `%`, `/` and loops.

fn gcd(a: i64, b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    return a;
}

fn main() -> i64 {
    let acc = 1;
    let k = 2;
    while k <= 20 {
        acc = acc / gcd(acc, k) * k;
        k = k + 1;
    }
    return acc;
}
