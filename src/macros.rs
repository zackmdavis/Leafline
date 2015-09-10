#![macro_export]

#[macro_use]
macro_rules! moral_panic {
    ($y:expr) => (panic!("{}, which is contrary to the operation of the moral law!", $y))
}

