// collection of useful algorithms I try to keep as generic as possible

/// greatest common divider for integer
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
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
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = egcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
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
}
