// expect: 871178
// Which start below 1,000 has the longest Collatz trajectory, and how many
// steps does it take? Returned packed as start * 1000 + steps
// (871 takes 178 steps to reach 1, so the answer is 871178).

fn steps(n: i64) -> i64 {
    let count = 0;
    while n != 1 {
        if n % 2 == 0 {
            n = n / 2;
        } else {
            n = 3 * n + 1;
        }
        count = count + 1;
    }
    return count;
}

fn main() -> i64 {
    let best = 1;
    let best_steps = 0;
    let n = 1;
    while n < 1000 {
        let s = steps(n);
        if s > best_steps {
            best = n;
            best_steps = s;
        } else {
            best = best;
        }
        n = n + 1;
    }
    return best * 1000 + best_steps;
}
