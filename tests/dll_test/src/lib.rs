use dy::*;

#[export]
pub fn multiply_two_only_numbers(args: Vec<Borrowed<'_>>) -> Owned {
    Value::new_arr(
        args.iter()
            .map(|b| match b.as_type() {
                As::Int(i) => Value::new_int(i.get() * 2),
                As::Float(f) => Value::new_float(f.get() * 2.0),
                _ => b.copy(),
            })
            .collect(),
    )
}
