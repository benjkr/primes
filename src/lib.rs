#![allow(dead_code)]

pub struct Primes {
    pub primes_found: Vec<u64>,
    current: u64,
}

impl Primes {
    pub fn new() -> Self {
        Self {
            primes_found: vec![],
            current: 2,
        }
    }

    pub fn next_prime(&mut self) -> u64 {
        while !self.is_prime(self.current) {
            self.current += 1;
        }
        self.primes_found.push(self.current);
        let prime = self.current;
        self.current += 1;

        prime
    }

    pub fn has_prime(&self, p: u64) -> bool {
        self.current > p && self.primes_found.binary_search(&p).is_ok()
    }

    fn is_prime(&mut self, n: u64) -> bool {
        let s = (n as f64).sqrt() as u64;
        for p in self.primes_found.iter() {
            if *p > s {
                break;
            }
            if n % p == 0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let mut p = Primes::new();
        let v: Vec<u64> = (0..5).map(|_| p.next_prime()).collect();
        assert_eq!(v, [2, 3, 5, 7, 11]);
    }
}
