pub enum Eval<const COND: bool> {}

pub trait IsTrue {}
pub trait IsFalse {}

impl IsTrue for Eval<true> {}
impl IsFalse for Eval<false> {}

pub trait AreSameType {}
impl<T> AreSameType for (T, T) {}

/// free function for `SameType`
/// ```
/// use jnickg_imaging::my_traits::require_same_type;
/// require_same_type::<usize, usize>(); // no-op
/// ```
/// ```compile_fail
/// use jnickg_imaging::my_traits::require_same_type;
/// require_same_type::<usize, String>(); // fails
/// ```
pub const fn require_same_type<A, B>()
where
    (A, B): AreSameType,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_same_type_compiles_when_same() {
        require_same_type::<usize, usize>();
    }
}
