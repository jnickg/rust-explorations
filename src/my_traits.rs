pub enum Assert<const COND: bool> {}

pub trait IsTrue {}
pub trait IsFalse {}

impl IsTrue for Assert<true> {}
impl IsFalse for Assert<false> {}

pub enum TheTypes<T, U> {
    T1(T),
    T2(U)
}
pub trait AreSame {}
pub trait NotSame {}

impl<T> AreSame for TheTypes<T, T> {}
impl<T, U> NotSame for TheTypes<T, U> {}
