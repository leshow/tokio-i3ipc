#[macro_use]
extern crate serde_derive;

pub mod event;
pub mod reply;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
