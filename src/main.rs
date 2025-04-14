fn factorial_iter(num: usize) -> usize {
    return (1..num).fold(1, |acc, x| acc + x);
}

fn factorial_loop(num: usize) -> usize {
    let mut sum = 1;
    for x in 2..num {
        sum += x;
    }

    return sum;
}

fn fibbonaci(n: usize) -> usize {

    match n {
        0 => 0,
        1 => 1,
        2 => 1,
        _ => fibbonaci( n - 1) + fibbonaci(n - 2),
    }
}

fn main() {
    println!("iter: {} , loop: {} , {}", factorial_iter(10), factorial_loop(10), fibbonaci(10));
}