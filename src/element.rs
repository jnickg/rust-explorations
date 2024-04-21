use num::{Num, One, Zero};
use std::ops::AddAssign;
pub trait Element: Num + Clone + Default + Copy + Zero + One + AddAssign {
    type ElementType;
}
impl<T> Element for T where T: Num + Clone + Default + Copy + Zero + One + AddAssign {
    type ElementType = T;
}