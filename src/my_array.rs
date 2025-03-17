use std::iter::FromIterator;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MyArray<T, const N: usize> {
    items: [T; N],
    n_items: usize,
}

impl<T: Copy + Clone + Default, const N: usize> MyArray<T, N> {
    /// Creates a new, empty `MyArray`.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let my_array: MyArray<i32, 10> = MyArray::new();
    /// assert_eq!(my_array.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            items: [T::default(); N],
            n_items: 0,
        }
    }

    /// Initializes a `MyArray` with a given item and number of initial items.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let my_array: MyArray<i32, 10> = MyArray::init(5, 3);
    /// assert_eq!(my_array.as_slice(), &[5, 5, 5]);
    /// ```
    pub fn init(init_item: T, n_init_items: usize) -> Self {
        Self {
            items: [init_item; N],
            n_items: n_init_items,
        }
    }

    /// Pushes an item to the end of the array.
    /// 
    /// # Panics
    /// Panics if the array is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.push(1);
    /// assert_eq!(my_array.as_slice(), &[1]);
    /// ```
    pub fn push(&mut self, item: T) {
        if self.n_items == N {
            panic!("line {}", line!());
        }
        self.items[self.n_items] = item;
        self.n_items += 1;
    }

    /// Pops an item from the end of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.push(1);
    /// let popped = my_array.pop();
    /// assert_eq!(popped, Some(1));
    /// assert_eq!(my_array.len(), 0);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.n_items == 0 {
            return None;
        }
        self.n_items -= 1;
        Some(self.items[self.n_items])
    }

    /// Inserts an item at a specified index.
    /// 
    /// # Panics
    /// Panics if the index is out of bounds or the array is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// my_array.insert(2, 6);
    /// assert_eq!(my_array.as_slice(), &[0, 1, 6, 2, 3, 4]);
    /// ```
    pub fn insert(&mut self, index: usize, item: T) {
        if index > self.n_items || self.n_items == N {
            panic!("line {}", line!());
        }
        self.items[index..].rotate_right(1);
        self.items[index] = item;
        self.n_items += 1;
    }

    /// Replaces an item at a specified index and returns the old item.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.push(1);
    /// let old_item = my_array.replace(0, 2);
    /// assert_eq!(old_item, Some(1));
    /// assert_eq!(my_array.as_slice(), &[2]);
    /// ```
    pub fn replace(&mut self, index: usize, item: T) -> Option<T> {
        if index >= self.n_items {
            return None;
        }
        let result = self.items[index];
        self.items[index] = item;
        Some(result)
    }

    /// Removes an item at a specified index and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// let removed = my_array.remove(2).unwrap();
    /// assert_eq!(my_array.as_slice(), &[0, 1, 3, 4]);
    /// assert_eq!(removed, 2);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.n_items {
            return None;
        }
        let result = self.items[index];
        self.items[index..].rotate_left(1);
        self.n_items -= 1;
        Some(result)
    }

    /// Clears all items from the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.push(1);
    /// my_array.flush();
    /// assert_eq!(my_array.len(), 0);
    /// ```
    pub fn flush(&mut self) {
        self.n_items = 0;
    }

    /// Gets a reference to an item at a specified index.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// assert_eq!(my_array.get(2), Some(&2));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.n_items {
            return None;
        }
        Some(&self.items[index])
    }

    /// Gets a mutable reference to an item at a specified index.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// if let Some(item) = my_array.get_mut(2) {
    ///     *item = 5;
    /// }
    /// assert_eq!(my_array.get(2), Some(&5));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.n_items {
            return None;
        }
        Some(&mut self.items[index])
    }

    /// Gets a reference to the last item in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// assert_eq!(my_array.get_last(), Some(&4));
    /// ```
    pub fn get_last(&mut self) -> Option<&T> {
        if self.n_items == 0 {
            return None;
        }
        Some(&self.items[self.n_items - 1])
    }

    /// Gets a slice of the array starting at a specified index with a specified length.
    /// 
    /// # Panics
    /// Panics if the length is zero or the slice is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// assert_eq!(my_array.get_slice(1, 3), &[1, 2, 3]);
    /// ```
    pub fn get_slice(&self, index: usize, len: usize) -> &[T] {
        if len == 0 {
            panic!("line {}", line!());
        }
        if index + len > self.n_items {
            panic!("line {}", line!());
        }
        &self.items[index..index + len]
    }

    /// Gets a slice of all items in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// assert_eq!(my_array.as_slice(), &array[..]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.items[..self.n_items]
    }

    /// Gets a mutable slice of all items in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// let slice = my_array.as_slice_mut();
    /// slice[2] = 5;
    /// assert_eq!(my_array.get(2), Some(&5));
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.items[..self.n_items]
    }

    /// Appends a slice of items to the end of the array.
    /// 
    /// # Panics
    /// Panics if the array does not have enough capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.append_slice(&[1, 2, 3]);
    /// assert_eq!(my_array.as_slice(), &[1, 2, 3]);
    /// ```
    pub fn append_slice(&mut self, slice: &[T]) {
        if self.n_items + slice.len() > N {
            panic!("line {}", line!());
        }
        for (i, item) in slice.iter().enumerate() {
            self.items[self.n_items + i] = *item;
        }
        self.n_items += slice.len();
    }

    /// Sets an item at a specified index and returns a reference to the new item.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let mut my_array: MyArray<i32, 10> = MyArray::new();
    /// my_array.push(1);
    /// let item = my_array.set(0, 2);
    /// assert_eq!(item, Some(&2));
    /// ```
    pub fn set(&mut self, index: usize, item: T) -> Option<&T> {
        if index >= self.n_items {
            return None;
        }
        self.items[index] = item;
        Some(&self.items[index])
    }

    /// Returns the number of items in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let my_array: MyArray<i32, 10> = MyArray::new();
    /// assert_eq!(my_array.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.n_items
    }

    /// Returns true if the array is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let my_array: MyArray<i32, 10> = MyArray::new();
    /// assert!(my_array.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.n_items == 0
    }

    /// Returns the remaining capacity of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let my_array: MyArray<i32, 10> = MyArray::new();
    /// assert_eq!(my_array.remaining_len(), 10);
    /// ```
    pub fn remaining_len(&self) -> usize {
        N - self.n_items
    }

    /// Returns an iterator over the items in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// let mut iter = my_array.iter();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().take(self.n_items)
    }

    /// Returns a mutable iterator over the items in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_lib::my_array::MyArray;
    /// let array = [0, 1, 2, 3, 4];
    /// let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
    /// for item in my_array.iter_mut() {
    ///     *item += 1;
    /// }
    /// assert_eq!(my_array.as_slice(), &[1, 2, 3, 4, 5]);
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().take(self.n_items)
    }
}

impl<T: Copy + Clone + Default, const N: usize> Index<usize> for MyArray<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.n_items);
        &self.items[index]
    }
}

impl<T: Copy + Clone + Default, const N: usize> IndexMut<usize> for MyArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.n_items);
        &mut self.items[index]
    }
}

impl<T: Copy + Clone + Default, const N: usize> FromIterator<T> for MyArray<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut my_array: MyArray<T, N> = MyArray::new();

        for i in iter {
            my_array.push(i);
        }
        my_array
    }
}

impl<T: Copy + Clone + Default, const N: usize> Default for MyArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect() {
        let array = [0, 1, 2, 3, 4];
        let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
        assert_eq!(my_array.as_slice(), &array[..]);
    }

    #[test]
    fn test_collect_empty_collection() {
        let array: [i32; 0] = [];
        let my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
        assert_eq!(my_array.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_collect_to_large_collection() {
        let array = [0, 1, 2, 3, 4];
        let _my_array: MyArray<i32, 4> = array.iter().map(|i| *i).collect();
    }

    #[test]
    fn test_remove() {
        let array = [0, 1, 2, 3, 4];
        let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
        let removed = my_array.remove(2).unwrap();
        assert_eq!(my_array.as_slice(), &[0, 1, 3, 4]);
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_insert() {
        let array = [0, 1, 2, 3, 4];
        let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
        my_array.insert(2, 6);
        assert_eq!(my_array.as_slice(), &[0, 1, 6, 2, 3, 4]);
    }

    #[test]
    fn test_index() {
        let array = [0, 1, 2, 3, 4];
        let mut my_array: MyArray<i32, 10> = array.iter().map(|i| *i).collect();
        assert_eq!(my_array[2], 2);
        my_array[2] = 5;
        assert_eq!(my_array[2], 5);
    }
}
