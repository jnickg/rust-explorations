use num::{Num, One, Zero};
use serde::Serialize;
use std::{fmt::Display, ops::AddAssign};
pub trait Element:
    Num + Display + Clone + Default + Copy + Zero + One + AddAssign + Serialize
{
    type ElementType;
}
impl<T> Element for T
where
    T: Num + Display + Clone + Default + Copy + Zero + One + AddAssign + Serialize,
{
    type ElementType = T;
}
