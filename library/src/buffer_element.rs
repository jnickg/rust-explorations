pub trait BufferElement: Clone + Copy + Default {}
impl<T> BufferElement for T where T: Clone + Copy + Default {}
