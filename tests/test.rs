use dy::*;
use std::ops::Deref;

#[test]
fn bool_array_test() {
    let cmp = vec![true, false, true, true, false];
    let dy = Value::new_bool_arr(&cmp);
    assert!(dy.is_bool_arr());

    let len = dy.get_bool_arr_len().unwrap();

    for i in 0..len {
        assert_eq!(cmp[i], dy.get_bool_by_idx(i).unwrap());
    }
}

#[test]
fn int_array_test() {
    let cmp = vec![2, 3, 4, 1];
    let dy = Value::new_int_arr(&cmp);
    assert!(dy.is_int_arr());
    assert_eq!(cmp, dy.get_int_arr().unwrap());
}

#[test]
fn float_array_test() {
    let cmp = vec![2.5, 3.6, 3.8, 1.2, 4.5, 5.8];
    let dy = Value::new_float_arr(&cmp);
    assert!(dy.is_float_arr());
    assert_eq!(cmp, dy.get_float_arr().unwrap());
}

fn get_doubled_array(val: Value) -> Value {
    assert!(val.is_arr());

    let data = val.get_arr().unwrap();
    let mut new_data: Vec<Value> = vec![];
    for d in &data {
        new_data.push(d.clone());
    }
    for d in &data {
        new_data.push(d.clone());
    }
    Value::new_arr(new_data)
}

#[test]
fn generic_array_test() {
    let dy = Value::new_arr(vec![
        Value::new_str("hello"),
        Value::new_int(15),
        Value::new_bool(true),
    ]);
    assert!(dy.is_arr());
    let dy = get_doubled_array(dy).decompose_arr().unwrap();
    assert_eq!(dy.len(), 6);

    for i in 0..6 {
        let val = &dy[i];
        match i {
            0 | 3 => assert_eq!(val.get_str().unwrap(), "hello"),
            1 | 4 => assert_eq!(val.get_int().unwrap(), 15),
            2 | 5 => assert_eq!(val.get_bool().unwrap(), true),
            _ => panic!("Invalid index"),
        }
    }
}

#[test]
fn generic_map_test() {
    let dy = Value::new_map(vec![
        ("foo", Value::new_int_arr(&[2, 5, 4, 8, 1])),
        ("bar", Value::new_str("hello")),
        ("baz", Value::new_int(15)),
    ]);
    assert!(dy.is_map());

    let map = dy.decompose_map().unwrap();

    for (key, val) in map.iter() {
        let key: &str = &key;
        match key {
            "foo" => assert_eq!(val.get_int_arr().unwrap(), &[2, 5, 4, 8, 1]),
            "bar" => assert_eq!(val.get_str().unwrap(), "hello"),
            "baz" => assert_eq!(val.get_int().unwrap(), 15),
            _ => panic!("Invalid key value"),
        }
    }
}
