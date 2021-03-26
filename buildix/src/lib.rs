pub mod error;
pub mod execute;
pub mod filter;
pub mod limit;
pub mod offset;
pub mod prelude;
pub mod select;
pub mod sort;

pub use crate::select::{Select, SelectBuilder};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
