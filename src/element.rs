use num::{Num, One, Zero};
use serde::Serialize;
use std::{fmt::Display, ops::{AddAssign, SubAssign}};
pub trait Element:
    Num + Display + Clone + Default + Copy + Zero + One + AddAssign + SubAssign + Serialize
{
    type ElementType;
}
impl<T> Element for T
where
    T: Num + Display + Clone + Default + Copy + Zero + One + AddAssign + SubAssign + Serialize,
{
    type ElementType = T;
}
