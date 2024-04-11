use std::ops::{Index, IndexMut};

pub trait BufferElement: Clone + Copy + Default {}
impl<T> BufferElement for T where T: Clone + Copy + Default {}

/// A Circular Buffer whose internal storage is known at compile time
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct CircularBuffer<T: BufferElement, const N: usize> {
    /// The actual data storage
    data: [T; N],

    /// The current size of the buffer, since elements might be popped off
    size: usize,

    /// The current cursor position in the buffer, for indexing
    cursor: usize,

    /// Equal to N. The total capaciy of the circular buffer
    capacity: usize,

    /// Whether to overwrite the oldest elements when the buffer is full
    overwrites: bool,
}

impl<T: BufferElement, const N: usize> Default for CircularBuffer<T, N> {
    fn default() -> Self {
        Self {
            data: [T::default(); N],
            size: 0,
            cursor: 0,
            capacity: N,
            overwrites: true,
        }
    }
}

impl<T: BufferElement, const N: usize> CircularBuffer<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self {
            data,
            size: N,
            cursor: N - 1,
            capacity: N,
            overwrites: false
        }
    }

    pub fn new_overwriting(data: [T; N]) -> Self {
        Self {
            data,
            size: N,
            cursor: N - 1,
            capacity: N,
            overwrites: true
        }
    }

    pub fn new_empty() -> Self
    {
        Self {
            data: [T::default(); N],
            size: 0,
            cursor: 0,
            capacity: N,
            overwrites: false
        }
    }

    pub fn new_empty_overwriting() -> Self
    {
        Self {
            data: [T::default(); N],
            size: 0,
            cursor: 0,
            capacity: N,
            overwrites: true
        }
    }


    pub fn append(&mut self, el: T) -> Result<usize, &'static str> {
        if self.size < self.capacity {
            self.data[self.size] = el;
            self.size += 1;
            self.cursor += 1;
            return Ok(self.size);
        }

        if self.overwrites {
            self.data[self.cursor % self.capacity] = el;
            // We don't increment size here, because we're overwriting an existing element
            self.cursor = (self.cursor + 1) % self.capacity;
            return Ok(self.size);
        }

        Err("Buffer is full")
    
    }

    pub fn pop(&mut self) -> Result<T, &'static str> {
        if self.size == 0 {
            return Err("Buffer is empty");
        }

        self.size -= 1;
        let idx = self.cursor % self.capacity;
        self.cursor = (self.cursor + self.capacity - 1) % self.capacity;
        Ok(self.data[idx])
    }


    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: BufferElement, const N: usize> Index<usize> for CircularBuffer<T, N> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        let actual_index = i % self.size;
        &self.data[actual_index]
    }
}

impl<T: BufferElement, const N: usize> IndexMut<usize> for CircularBuffer<T, N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let actual_index = i % self.size;
        &mut self.data[actual_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_buffer_appends_until_full_when_non_overwriting() {
        let mut buffer = CircularBuffer::<u32, 3>::new_empty();
        assert_eq!(buffer.append(1), Ok(1));
        assert_eq!(buffer.append(2), Ok(2));
        assert_eq!(buffer.append(3), Ok(3));
        assert_eq!(buffer.append(4), Err("Buffer is full"));
        assert_eq!(buffer.data, [1, 2, 3]);
    }

    #[test]
    fn circular_buffer_appends_forever_when_overwriting() {
        let mut buffer = CircularBuffer::<u32, 3>::new_empty_overwriting();
        assert_eq!(buffer.append(1), Ok(1));
        assert_eq!(buffer.append(2), Ok(2));
        assert_eq!(buffer.append(3), Ok(3));
        assert_eq!(buffer.append(4), Ok(3));
        assert_eq!(buffer.append(5), Ok(3));
        // Cursor swept around to 0
        assert_eq!(buffer.data, [4, 5, 3]);
    }

    #[test]
    fn circular_buffer_indexes_repeatedly() {
        let buffer = CircularBuffer::new([1, 2, 3]);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);
        assert_eq!(buffer[3], 1);
        assert_eq!(buffer[4], 2);
        assert_eq!(buffer[5], 3);
    }

    #[test]
    fn circular_buffer_mutates_repeatedly() {
        let mut buffer = CircularBuffer::new([1, 2, 3]);
        buffer[0] = 4;
        buffer[1] = 5;
        buffer[2] = 6;
        assert_eq!(buffer[0], 4);
        assert_eq!(buffer[1], 5);
        assert_eq!(buffer[2], 6);
        assert_eq!(buffer[3], 4);
        assert_eq!(buffer[4], 5);
        assert_eq!(buffer[5], 6);
        assert_eq!(buffer.data, [4, 5, 6]);
    }

    #[test]
    fn circular_buffer_pop_repeatedly() {
        let mut buffer = CircularBuffer::new([1, 2, 3]);
        assert_eq!(buffer.pop(), Ok(3));
        assert_eq!(buffer.pop(), Ok(2));
        assert_eq!(buffer.pop(), Ok(1));
        assert_eq!(buffer.pop(), Err("Buffer is empty"));
        assert_eq!(buffer.size, 0);
    }


}