use std::marker::PhantomData;

pub enum Eval<const COND: bool> {}

pub trait IsTrue {}
pub trait IsFalse {}

impl IsTrue for Eval<true> {}
impl IsFalse for Eval<false> {}

pub struct TheTypes<T, U> {
    __p1: PhantomData<T>,
    __p2: PhantomData<U>,
}
pub trait AreSame {}
pub trait AreNotSame {}

impl<T> AreSame for TheTypes<T, T> {}
impl<T, U> AreNotSame for TheTypes<T, U> {}
