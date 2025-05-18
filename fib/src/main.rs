fn next_fib(a: u64, b: u64) -> (u64, u64) {
    (b, a + b)
}

fn fib_loop(n: u8) {
    let mut a = 1;
    let mut b = 1;
    let mut i = 2u8;

    loop {
        (a, b) = next_fib(a, b);
        i += 1;
        println!("next val is {}", b);

        if i >= n {
            break;
        }
    }
}


fn fib_while(n: u8) {
    let (mut a, mut b, mut i) = (1, 1, 2u8);

    while i < n {
        (a, b) = next_fib(a, b);
        i += 1;

        println!("next val is {}", b);
    }
}


fn fib_for(n: u8) {
    let (mut a, mut b) = (1, 1);

    for _i in 2..n {
        (a, b) = next_fib(a, b);
        println!("next val is {}", b);
    }
}

struct Fibnacci {
    a: u64,
    b: u64,
}

impl Fibnacci {
    fn new() -> Self {
        Fibnacci {a: 1, b: 1}
    }
}

impl Iterator for Fibnacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.b;
        (self.a, self.b) = (self.b, self.a + self.b);
        Some(next)
    }
}

fn main() {
    let n = 10;
    fib_loop(n);
    fib_while(n);
    fib_for(n);

    println!("fibnacci iterator");

    for val in Fibnacci::new().take(n as usize) {
        println!("next val is {}", val);
    }
}