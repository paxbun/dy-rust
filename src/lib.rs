mod bindings;
use bindings::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        unsafe {
            let dy = dy_make_i(15);
            assert_eq!(dy_get_type(dy), _dy_type_t_dy_type_i);
            assert_eq!(dy_get_i(dy), 15);
        }
    }
}
