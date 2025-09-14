// collection of useful algorithms I try to keep as generic as possible

/// greatest common divider for integer
pub fn gcd(a: i64, b: i64) -> i64 {
    gcd_i128(a as i128, b as i128) as i64
}

pub fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs()
}

/// extended greatest common divider for integer
/// If gcd(a, b) == 1, than x can be used to calculate the inverse of a mod b:
/// inv_a_mod_b = x mod b
/// if only positive remainder is required, use rem_euclid() instead of % for mod
pub fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    let (gcd, x, y) = egcd_i128(a as i128, b as i128);
    (gcd as i64, x as i64, y as i64)
}

pub fn egcd_i128(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = egcd_i128(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

/// modinv calculate the first inverse of a mod m

pub fn modinv(a: i64, m: i64) -> Option<i64> {
    modinv_i128(a as i128, m as i128).map(|r| r as i64)
}
pub fn modinv_i128(a: i128, m: i128) -> Option<i128> {
    let (g, x, _) = egcd_i128(a, m);
    if g != 1 && g != -1 { return None; }
    Some(x.rem_euclid(m))
}


/// modpow: calculate (base^exp) % mod
/// (base^exp) % modulus, iterative, O(log exp).
/// Warning: big values of exp and modulus may result in overflow errors
pub fn modpow(base: i64, exp: i64, modulus: i64) -> i64 {
    assert!(base >= 0);
    assert!(exp >= 0);
    assert!(modulus >= 0);
    modpow_u128(base as u128, exp as u128, modulus as u128) as i64
}

pub fn modpow_u128(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 { return 0; }
    let mut result = 1 % modulus;
    base %= modulus;

    while exp > 0 {
        if (exp & 1) == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exp >>= 1;
    }
    result
}


/// collecting all possible sub groups with n elements of a group with m elements and m > n
use std::cmp::Ordering;

pub fn collect_all_n_from_m_elements<T: Copy>(
    main_group: &[T],
    num_to_collect: usize,
) -> Vec<Vec<T>> {
    let mut collections = Vec::new();
    if num_to_collect == 0 {
        return collections;
    }
    match num_to_collect.cmp(&main_group.len()) {
        Ordering::Greater => (),
        Ordering::Equal => {
            collections.push(main_group.to_owned());
        }
        Ordering::Less => {
            let mut current_collection = Vec::new();
            recursive_collection_of_elements(
                main_group,
                num_to_collect,
                &mut current_collection,
                &mut collections,
            );
        }
    }

    collections
}

fn recursive_collection_of_elements<T: Copy>(
    mg: &[T],
    num_to_collect: usize,
    current_collection: &mut Vec<T>,
    collections: &mut Vec<Vec<T>>,
) {
    if mg.is_empty() {
        return;
    }
    for index in 0..mg.len() {
        current_collection.push(mg[index]);
        if current_collection.len() == num_to_collect {
            collections.push(current_collection.to_owned());
        } else {
            let sliced_mg = &mg[index + 1..];
            if sliced_mg.len() + current_collection.len() >= num_to_collect {
                recursive_collection_of_elements(
                    sliced_mg,
                    num_to_collect,
                    current_collection,
                    collections,
                );
            }
        }
        current_collection.pop();
    }
}

/// get all possible combinations of a range of numbers as iterator
use std::collections::VecDeque;
pub struct RangeCombinations {
    start: i64,
    end: i64,
    combination: VecDeque<i64>,
    iter_finished: bool,
}

impl RangeCombinations {
    pub fn new(start: i64, end: i64) -> Self {
        Self {
            start,
            end,
            combination: VecDeque::new(),
            iter_finished: start > end,
        }
    }
    fn set_next(&mut self) -> bool {
        if self.combination.is_empty() {
            self.add_entry();
            false
        } else if self.set_last() {
            true
        } else {
            self.add_entry();
            false
        }
    }
    fn set_last(&mut self) -> bool {
        // there are no further combinations, if no number left to pop
        let Some(last) = self.combination.pop_back() else { return true;};
        if last + 1 > self.end {
            // reached end of range -> move one backwards
            self.set_last()
        } else if let Some(next) = (last + 1..=self.end)
            .filter(|n| !self.combination.contains(n))
            .next()
        {
            self.combination.push_back(next);
            false
        } else {
            // could not find possible next number -> move one backwards
            self.set_last()
        }
    }
    fn add_entry(&mut self) {
        // add entry, if some number from range is missing
        if let Some(next) = (self.start..=self.end)
            .filter(|n| !self.combination.contains(n))
            .next()
        {
            self.combination.push_back(next);
            self.add_entry();
        }
    }
}

impl Iterator for RangeCombinations {
    type Item = VecDeque<i64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_finished {
            return None;
        }
        if self.set_next() {
            self.iter_finished = true;
            None
        } else {
            Some(self.combination.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_all_n_from_m_elements() {
        let main_group = [1, 2, 3, 4, 5];
        let num_to_collect = 3;
        let collections = collect_all_n_from_m_elements(&main_group, num_to_collect);

        assert_eq!(collections.len(), 10);
        assert_eq!(collections[0], [1, 2, 3]);
        assert_eq!(collections[1], [1, 2, 4]);
        assert_eq!(collections[2], [1, 2, 5]);
        assert_eq!(collections[3], [1, 3, 4]);
        assert_eq!(collections[4], [1, 3, 5]);
        assert_eq!(collections[5], [1, 4, 5]);
        assert_eq!(collections[6], [2, 3, 4]);
        assert_eq!(collections[7], [2, 3, 5]);
        assert_eq!(collections[8], [2, 4, 5]);
        assert_eq!(collections[9], [3, 4, 5]);
    }

    #[test]
    fn test_range_combinations() {
        let mut range_combinations = RangeCombinations::new(0, 2);
        assert_eq!(range_combinations.next(), Some([0, 1, 2].into()));
        assert_eq!(range_combinations.next(), Some([0, 2, 1].into()));
        assert_eq!(range_combinations.next(), Some([1, 0, 2].into()));
        assert_eq!(range_combinations.next(), Some([1, 2, 0].into()));
        assert_eq!(range_combinations.next(), Some([2, 0, 1].into()));
        assert_eq!(range_combinations.next(), Some([2, 1, 0].into()));
        assert_eq!(range_combinations.next(), None);

    }
}
