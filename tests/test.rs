use dy::*;

#[test]
fn bool_array_test() {
    let cmp = vec![true, false, true, true, false];
    let dy = Value::new_bool_arr(&cmp);
    let arr = dy.as_bool_arr().unwrap();
    for i in 0..arr.len() {
        assert_eq!(cmp[i], arr.at(i).unwrap());
    }
}

#[test]
fn bytes_test() {
    let cmp = vec![2, 3, 4, 1];
    let dy = Value::new_bytes(&cmp);
    let arr = dy.as_bytes().unwrap();
    assert_eq!(cmp, arr.data());
}

#[test]
fn int_array_test() {
    let cmp = vec![2, 3, 4, 1];
    let dy = Value::new_int_arr(&cmp);
    let arr = dy.as_int_arr().unwrap();
    assert_eq!(cmp, arr.data());
}

#[test]
fn float_array_test() {
    let cmp = vec![2.5, 3.6, 3.8, 1.2, 4.5, 5.8];
    let dy = Value::new_float_arr(&cmp);
    let arr = dy.as_float_arr().unwrap();
    assert_eq!(cmp, arr.data());
}

fn get_doubled_array(arr: AsArrValue<'_>) -> Owned {
    let mut new_data = vec![];
    for d in arr.iter() {
        new_data.push(d.copy());
    }
    for d in arr.iter() {
        new_data.push(d.copy());
    }
    Value::new_arr(new_data)
}

#[test]
fn generic_array_test() {
    let arr = Value::new_arr(vec![
        Value::new_str("hello"),
        Value::new_int(15),
        Value::new_bool(true),
    ]);
    let arr = arr.as_arr().unwrap();
    let arr2 = get_doubled_array(arr);
    let arr2 = arr2.as_arr().unwrap();
    assert_eq!(arr2.len(), 6);

    for i in 0..6 {
        let val = arr2.at(i).unwrap();
        match i {
            0 | 3 => assert_eq!(val.as_str().unwrap().get(), "hello"),
            1 | 4 => assert_eq!(val.as_int().unwrap().get(), 15),
            2 | 5 => assert_eq!(val.as_bool().unwrap().get(), true),
            _ => panic!("Invalid index"),
        }
    }
}

#[test]
fn generic_map_test() {
    let map = Value::new_map(vec![
        ("foo", Value::new_int_arr(&[2, 5, 4, 8, 1])),
        ("bar", Value::new_str("hello")),
        ("baz", Value::new_int(15)),
    ]);
    let map = map.as_map().unwrap();
    for pair in map.iter() {
        let val = pair.get_val();
        match pair.get_key() {
            "foo" => assert_eq!(val.as_int_arr().unwrap().data(), &[2, 5, 4, 8, 1]),
            "bar" => assert_eq!(val.as_str().unwrap().get(), "hello"),
            "baz" => assert_eq!(val.as_int().unwrap().get(), 15),
            _ => panic!("Invalid key value"),
        }
    }
}

#[test]
fn as_test() {
    let arr = Value::new_arr(vec![
        Value::new_str("hello"),
        Value::new_int(15),
        Value::new_bool(true),
    ]);
    let arr = arr.as_arr().unwrap();

    for e in arr.iter() {
        match e.as_type() {
            As::Str(s) => assert_eq!(s.get(), "hello"),
            As::Int(i) => assert_eq!(i.get(), 15),
            As::Bool(b) => assert_eq!(b.get(), true),
            _ => panic!("Invalid type"),
        }
    }
}