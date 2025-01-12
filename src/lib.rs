#![allow(dead_code)]

use std::{cell::RefCell, collections::HashSet, rc::Rc};

const MB: u64 = 1_048_576;
const BATCH_SIZE: u64 = 10 * MB;

struct Batch {
    current: u64,
    size: u64,
    offset: u64,
}

impl Batch {
    fn new(batch_size: u64) -> Self {
        Self {
            current: 0,
            size: batch_size,
            offset: 0,
        }
    }

    fn calculate_offset(&mut self) {
        self.offset = self.current * self.size;
    }

    fn set_batch(&mut self, batch: u64) {
        self.current = batch;
        self.calculate_offset();
    }

    fn update_batch(&mut self, f: fn(u64) -> u64) {
        self.current = f(self.current);
        self.calculate_offset();
    }
}

/// The Primes struct is responsible to find and save all the primes (so far).
/// Its design uses batches to calculate the next set of primes with the previus ones.
///
/// Usage:
/// ```
/// use primes::Primes;
///
/// let mut primes = Primes::new();
/// assert!(primes.is_prime(2));
/// assert!(primes.is_prime(3));
/// assert!(primes.is_prime(5));
/// ```
pub struct Primes {
    inner_slice: Box<Vec<bool>>,
    batch: Batch,
    pub primes_found: Rc<RefCell<HashSet<u64>>>,
    pub primes_ordered: Vec<u64>,
}

impl Primes {
    pub fn new() -> Self {
        Self::with_batch_size(BATCH_SIZE)
    }

    pub fn with_batch_size(batch_size: u64) -> Self {
        let mut s = Self {
            inner_slice: Box::new(vec![true; batch_size as usize]),
            primes_found: Rc::new(RefCell::new(HashSet::new())),
            primes_ordered: Vec::new(),
            batch: Batch::new(batch_size),
        };
        s.populate_first_batch();
        s.save_primes();

        s
    }

    pub fn iter(&mut self) -> PrimesIterator<'_> {
        PrimesIterator {
            primes: self,
            current: 0,
        }
    }

    fn populate_first_batch(&mut self) {
        self.inner_slice[0] = false;
        self.inner_slice[1] = false;
        for i in 2..self.inner_slice.len() {
            if self.inner_slice[i] {
                let mut tmp = i + i;
                while tmp < self.batch.size as usize {
                    self.inner_slice[tmp] = false;

                    tmp += i;
                }
            }
        }
    }

    fn save_primes(&mut self) {
        let primes: Vec<u64> = self
            .inner_slice
            .iter()
            .enumerate()
            .filter_map(|(i, is_prime)| {
                if *is_prime {
                    Some(i as u64 + self.batch.offset)
                } else {
                    None
                }
            })
            .collect();

        self.primes_found.borrow_mut().extend(&primes);
        self.primes_ordered.extend(&primes);
    }

    /// This function will calculate and populate the next batch of primes by the specified batch size.
    pub fn populate_next_batch(&mut self) {
        self.batch.update_batch(|current| current + 1);
        self.inner_slice.fill(true); // Reset the slice to all trues

        for prime in self.primes_ordered.iter() {
            let mul = (self.batch.offset as f64 / *prime as f64).ceil() as u64; // How much to multiply prime to reach offset (closest)
            let mut tmp = prime * mul;

            while tmp < self.batch.offset + self.batch.size {
                let indx = tmp - self.batch.offset;
                self.inner_slice[indx as usize] = false;

                tmp += *prime;
            }
        }

        self.save_primes();
    }

    /// Returns weather `n` is prime or not.
    ///
    /// NOTE: If the prime isn't yet checked, it will calculate the batches until it.
    pub fn is_prime(&mut self, n: u64) -> bool {
        while n > (self.batch.current + 1) * BATCH_SIZE {
            self.populate_next_batch();
        }

        self.primes_found.borrow().contains(&n)
    }

    pub fn primes_found_set(&self) -> Rc<RefCell<HashSet<u64>>> {
        self.primes_found.clone()
    }
}

/// Struct to iterate over all the primes until `u64::MAX`.
///
/// Usage:
/// ```
/// use primes::Primes;
///
/// let mut primes = Primes::new();
/// let mut primes_iter = primes.iter().take(3); // First 3 primes
/// assert_eq!(primes_iter.next(), Some(2));
/// assert_eq!(primes_iter.next(), Some(3));
/// assert_eq!(primes_iter.next(), Some(5));
/// assert_eq!(primes_iter.next(), None);
///
/// let mut primes_iter = primes.iter(); // All primes until u64::MAX
/// assert_eq!(primes_iter.next(), Some(2));
/// assert_eq!(primes_iter.next(), Some(3));
/// assert_eq!(primes_iter.next(), Some(5));
/// assert_eq!(primes_iter.next(), Some(7));
/// ```
pub struct PrimesIterator<'a> {
    primes: &'a mut Primes,
    current: usize,
}

impl Iterator for PrimesIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current >= self.primes.primes_ordered.len() {
            self.primes.populate_next_batch();
        }
        let p = self.primes.primes_ordered[self.current];
        self.current += 1;

        Some(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const PRIMES: &'static str = include_str!("./test_static/primes.txt");

    #[test]
    fn test_working() {
        let mut p = Primes::with_batch_size(BATCH_SIZE);
        assert!(p.is_prime(2));
        assert!(p.is_prime(3));
        assert!(!p.is_prime(4));
        assert!(p.is_prime(5));
        assert!(!p.is_prime(6));
        assert!(p.is_prime(7));
        assert!(p.is_prime(1299709));
        assert!(!p.is_prime(1299708));
    }

    #[test]
    fn test_file() {
        let mut p = Primes::with_batch_size(BATCH_SIZE);

        let primes_vec: Vec<u64> = PRIMES
            .lines()
            .map(|l| u64::from_str_radix(l.trim(), 10).unwrap())
            .collect();

        assert_eq!(
            primes_vec,
            p.iter().take(primes_vec.len()).collect::<Vec<u64>>()
        )
    }
}
