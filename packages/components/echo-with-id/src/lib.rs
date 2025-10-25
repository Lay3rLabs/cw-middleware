#[allow(clippy::all)]
mod bindings;
mod entry;
mod error;

#[cfg(test)]
mod test {
    use crate::entry::handle_raw;

    #[test]
    fn test_echo_with_id() {
        let res = handle_raw(b"Hello, world!".to_vec()).unwrap();
        assert_eq!(res.unwrap().payload, b"Hello, world!".to_vec());
    }
}
