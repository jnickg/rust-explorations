use std::ops::{Index, IndexMut};

/// A Circular Buffer whose internal storage is known at compile time
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct MyCicularBuffer<T, const N: usize> {
    /// The actual data storage
    data: [T; N],

    /// The current size of the buffer, since elements might be popped off
    size: usize,

    /// Equal to N. The total capaciy of the circular buffer
    capacity: usize,
}

impl<T, const N: usize> MyCicularBuffer<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self {
            data,
            size: N,
            capacity: N,
        }
    }

    pub fn new_empty() -> Self
    where
        T: Default + Clone + Copy,
    {
        Self {
            data: [T::default(); N],
            size: 0,
            capacity: N,
        }
    }

    pub fn append(&mut self, el: T) -> Result<usize, &'static str> {
        if self.size == self.capacity {
            return Err("Buffer is full");
        }

        self.data[self.size] = el;
        self.size += 1;
        Ok(self.size)
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T, const N: usize> Index<usize> for MyCicularBuffer<T, N> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        let actual_index = i % self.size;
        &self.data[actual_index]
    }
}

impl<T, const N: usize> IndexMut<usize> for MyCicularBuffer<T, N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let actual_index = i % self.size;
        &mut self.data[actual_index]
    }
}
