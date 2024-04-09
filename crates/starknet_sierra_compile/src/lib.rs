//! An experimental attempt to compile Sierra into Casm by using cairo-lang as a lib .

pub mod compile;

#[cfg(any(feature = "testing", test))]
pub mod test_utils;
