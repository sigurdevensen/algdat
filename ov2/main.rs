use std::time::Instant;

// metode 1
fn linear_exp(base: f64, exponent: i32) -> f64 {
    if exponent == 0 {
        return 1.0;
    }

    if exponent == 1 {
        return base;
    }

    return base * linear_exp(base, exponent - 1);
}

// metode 2
fn log_exp(base: f64, exponent: i32) -> f64 {
    if exponent == 0 {
        return 1.0;
    }

    if exponent == 1 {
        return base;
    }

    if exponent & 1 == 0 {  //raskere enn % 2 == 0
        return log_exp(base * base, exponent >> 1);
    } else {
        return base * log_exp(base * base, (exponent - 1) >> 1);
    }
}

// metode 3
fn builtin_exp(base: f64, exponent: i32) -> f64 {
    return base.powf(exponent as f64);
}

fn main() {
    let base = 1.000001;
    let ns = [10, 100, 1000, 5000, 10000];

    let iterations = 1000;

    println!("{:<8} {:>14} {:>16} {:>12}", "n", "linear (ns)", "logarithmic (ns)", "builtin (ns)");
    println!("{}", "-".repeat(55));

    for &n in &ns {
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = linear_exp(base, n);
        }
        let t1 = start.elapsed().as_nanos() / iterations;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = log_exp(base, n);
        }
        let t2 = start.elapsed().as_nanos() / iterations;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = builtin_exp(base, n);
        }
        let t3 = start.elapsed().as_nanos() / iterations;

        println!("{:<8} {:>14} {:>16} {:>12}", n, t1, t2, t3);
    }

    let test_base = 5.0;
    let test_exp = 11;
    let expected = 48828125.0;
    assert_eq!(linear_exp(test_base, test_exp), expected, "linear_exp failed");
    assert_eq!(log_exp(test_base, test_exp), expected, "log_exp failed");
    assert_eq!(builtin_exp(test_base, test_exp), expected, "builtin_exp failed");
    println!("\nAlle metoder gir riktig resultat for {}^{} = {}", test_base, test_exp, expected);
}
