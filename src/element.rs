use num::{Num, One, Zero};
use std::ops::AddAssign;
use serde::Serialize;
pub trait Element: Num + Clone + Default + Copy + Zero + One + AddAssign + Serialize {
    type ElementType;
}
impl<T> Element for T where T: Num + Clone + Default + Copy + Zero + One + AddAssign + Serialize {
    type ElementType = T;
}