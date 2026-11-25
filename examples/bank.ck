// expect: 323945206
// A bank-transfer workload with an invariant. Three accounts, 500 pseudo-
// random transfers (the MINSTD Park-Miller generator, so it is deterministic;
// its constants fit mentat's 32-bit immediates, see the README's limits),
// overdrafts rejected. Money must be conserved: if the total ever differs
// from 1500 the program returns -1. Result packs the final balances of
// accounts a and b with the number of rejected transfers: a * 10^6 + b * 10^3
// + rejected (here a = 323, b = 945, so c = 232, and 206 were rejected).

fn next(seed: i64) -> i64 {
    return seed * 48271 % 2147483647;
}

fn main() -> i64 {
    let a = 500;
    let b = 500;
    let c = 500;
    let rejected = 0;
    let seed = 42;
    let i = 0;
    while i < 500 {
        seed = next(seed);
        let from = seed % 3;
        seed = next(seed);
        let to = seed % 3;
        seed = next(seed);
        let amount = seed % 200;
        let balance = a;
        if from == 1 {
            balance = b;
        } else {
            if from == 2 {
                balance = c;
            } else {
                balance = a;
            }
        }
        if amount > balance or from == to {
            rejected = rejected + 1;
        } else {
            if from == 0 { a = a - amount; } else { if from == 1 { b = b - amount; } else { c = c - amount; } }
            if to == 0 { a = a + amount; } else { if to == 1 { b = b + amount; } else { c = c + amount; } }
        }
        if a + b + c != 1500 {
            return -1;
        } else {
            i = i + 1;
        }
    }
    return a * 1000000 + b * 1000 + rejected;
}
