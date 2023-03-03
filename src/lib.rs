pub mod component;
pub mod gug;
mod macros;
pub mod rewrite;

pub use crate::component::debug::DebugData;
pub use crate::gug::Gug;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
