#![allow(non_upper_case_globals)]

mod bindings;

use bindings::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{Into, TryFrom};
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::ptr::null;
use std::slice::from_raw_parts;

macro_rules! def_type {
    ($($doc:literal $name:ident = $internal:ident),+ $(,)?) => {
        /// Indicates the type of `dy` values
        #[derive(Debug, PartialEq, Clone, Copy)]
        #[allow(non_upper_case_global)]
        pub enum Type {
            $(
                #[doc = $doc]
                #[doc = " type"]
                $name = $internal as isize,
            )+
        }

        impl TryFrom<_dy_type_t> for Type {
            type Error = ();
            fn try_from(i: _dy_type_t) -> Result<Self, ()> {
                match i {
                    $($internal => Ok(Type::$name),)+
                    _ => Err(())
                }
            }
        }

        impl Into<_dy_type_t> for Type {
            fn into(self) -> _dy_type_t {
                match self {
                    $(Type::$name => $internal,)+
                }
            }
        }
    };
}

def_type! {
    "Null type"
    Null = _dy_type_t_dy_type_null,
    "Boolean type"
    Bool = _dy_type_t_dy_type_b,
    "8-byte integer type"
    Int = _dy_type_t_dy_type_i,
    "Double-precision floating point type"
    Float = _dy_type_t_dy_type_f,
    "String type"
    Str = _dy_type_t_dy_type_str,
    "Boolean array type"
    BoolArr = _dy_type_t_dy_type_barr,
    "Integer array type"
    IntArr = _dy_type_t_dy_type_iarr,
    "Floating point number type"
    FloatArr = _dy_type_t_dy_type_farr,
    "Generic array type"
    Arr = _dy_type_t_dy_type_arr,
    "Generic map type"
    Map = _dy_type_t_dy_type_map,
}

/// Indicates a value
pub type ValuePtr = dy_t;

/// Wraps a value.
#[derive(Debug)]
pub struct Value {
    val: ValuePtr,
    owned: bool,
}

/// Indicates an iterator of a generic map
#[derive(Debug)]
pub struct MapIter<'a> {
    wrap: &'a Value,
    iter: dy_iter_t,
}

/// Indicates a key-value pair
#[derive(Debug)]
pub struct KeyValPair<'a> {
    key: &'a str,
    wrap: Value,
}

impl Value {
    /// Returns the type of the value
    pub fn get_type(&self) -> Type {
        unsafe { Type::try_from(dy_get_type(self.val)).unwrap() }
    }

    /// Copies the instance if the instance is borrowed
    pub fn ensure_ownership(self) -> Self {
        if self.owned {
            self
        } else {
            self.clone()
        }
    }

    /// Creates a new wrapper instance from a value
    ///
    /// # Arguments
    ///
    /// * `val` - the value instance to wrap
    pub unsafe fn from_ptr(val: ValuePtr) -> Self {
        Value {
            val: val,
            owned: true,
        }
    }

    /// Creates a new wrapper instance from a value but without the ownership.
    ///
    /// # Arguments
    ///
    /// * `val` - the value instance to wrap
    pub unsafe fn from_ptr_borrowed(val: ValuePtr) -> Self {
        Value {
            val: val,
            owned: false,
        }
    }

    /// Returns the internal value
    pub unsafe fn into_val(mut self) -> ValuePtr {
        self.owned = false;
        self.val
    }

    /// Makes a new null value
    pub fn new_null() -> Self {
        unsafe { Value::from_ptr(dy_make_null()) }
    }

    /// Makes a new string
    ///
    /// # Arguments
    ///
    /// * `v` - the string to copy
    pub fn new_str(v: &str) -> Self {
        let s = CString::new(v).unwrap();
        unsafe { Value::from_ptr(dy_make_str(s.as_ptr())) }
    }

    /// Returns the length of the string if the value is an string
    pub fn get_str_len(&self) -> Option<usize> {
        if !self.is_str() {
            None
        } else {
            unsafe { Some(self.get_str_len_unchecked()) }
        }
    }

    /// Returns the length of the string without checking the type
    pub unsafe fn get_str_len_unchecked(&self) -> usize {
        dy_get_str_len(self.val) as usize
    }

    pub fn get_str(&self) -> Option<String> {
        if !self.is_str() {
            None
        } else {
            unsafe { Some(self.get_str_unchecked()) }
        }
    }

    /// Retrieves the internal data without checking the type.
    pub unsafe fn get_str_unchecked(&self) -> String {
        match CStr::from_ptr(dy_get_str_data(self.val)).to_string_lossy() {
            Cow::Borrowed(s) => String::from(s),
            Cow::Owned(s) => s,
        }
    }

    /// Makes a new generic array
    ///
    /// # Arguments
    ///
    /// * `v` - the array to put
    pub fn new_arr(v: Vec<Value>) -> Self {
        let v: Vec<ValuePtr> = v
            .into_iter()
            .map(|w: Value| unsafe { w.ensure_ownership().into_val() })
            .collect();
        unsafe { Value::from_ptr(dy_make_arr(v.as_ptr(), v.len() as u64)) }
    }

    /// Returns the length of the array if the value is an array
    pub fn get_arr_len(&self) -> Option<usize> {
        if !self.is_arr() {
            None
        } else {
            unsafe { Some(self.get_arr_len_unchecked()) }
        }
    }

    /// Returns the length of the array without checking the type
    pub unsafe fn get_arr_len_unchecked(&self) -> usize {
        dy_get_arr_len(self.val) as usize
    }

    /// Returns the data of the entry at the given index in the internal data if the value
    /// is an array.
    ///
    /// # Arguments
    ///
    /// * `idx` - the index of the entry
    pub fn get_arr_idx(&self, idx: usize) -> Option<Value> {
        if !self.is_arr() {
            None
        } else if idx >= self.get_arr_len().unwrap() {
            None
        } else {
            Some(unsafe { self.get_arr_idx_unchecked(idx) })
        }
    }

    /// Returns the data of the entry at the given index without checking the type
    ///
    /// * `idx` - the index of the entry
    pub unsafe fn get_arr_idx_unchecked(&self, idx: usize) -> Value {
        Value::from_ptr_borrowed(dy_get_arr_idx(self.val, idx as u64))
    }

    /// Retrieves the internal data without checking the type
    pub unsafe fn get_arr_unchecked(&self) -> Vec<Value> {
        let arr: &[ValuePtr] =
            from_raw_parts(dy_get_arr_data(self.val), self.get_arr_len_unchecked());
        arr.iter().map(|v| Value::from_ptr_borrowed(*v)).collect()
    }

    /// decomposes the value into a vector if the value is an array
    pub fn decompose_arr(self) -> Result<Vec<Value>, Self> {
        if self.is_arr() && self.owned {
            unsafe { Ok(self.decompose_arr_unchecked()) }
        } else {
            Err(self)
        }
    }

    /// decomposes the value (possibly an array) into a vector without checking the type of the valueFF
    pub unsafe fn decompose_arr_unchecked(self) -> Vec<Value> {
        let arr: &[ValuePtr] =
            from_raw_parts(dy_get_arr_data(self.val), self.get_arr_len_unchecked());
        let rtn = arr.iter().map(|v| Value::from_ptr(*v)).collect();
        let s = ManuallyDrop::new(self);
        dy_dispose_self(s.val);
        rtn
    }

    /// Makes a new generic map
    ///
    /// # Arguments
    ///
    /// * `v` - the array of key-value pairs to copy
    pub fn new_map(v: Vec<(&str, Value)>) -> Self {
        let v: Vec<(CString, ValuePtr)> = v
            .into_iter()
            .map(|tup| {
                let (s, w) = tup;
                (CString::new(s).unwrap(), unsafe {
                    w.ensure_ownership().into_val()
                })
            })
            .collect();

        let vv: Vec<dy_keyval_t> = v
            .iter()
            .map(|tup| {
                let (s, v) = tup;
                dy_keyval_t {
                    key: s.as_ptr(),
                    val: *v,
                }
            })
            .collect();

        unsafe { Value::from_ptr(dy_make_map(vv.as_ptr(), vv.len() as u64)) }
    }

    /// Returns the length of the map if the value is an map
    pub fn get_map_len(&self) -> Option<usize> {
        if !self.is_map() {
            None
        } else {
            unsafe { Some(self.get_map_len_unchecked()) }
        }
    }

    /// Returns the length of the array without checking the type
    pub unsafe fn get_map_len_unchecked(&self) -> usize {
        dy_get_map_len(self.val) as usize
    }

    /// Returns the data with the given key
    ///
    /// # Arguments
    ///
    /// * `key` - the key of the data
    pub fn get_map_key(&self, key: &str) -> Option<KeyValPair> {
        if !self.is_map() {
            None
        } else {
            unsafe { self.get_map_key_unchecked(key) }
        }
    }

    /// Returns the data with the given key without checking the type of the value
    ///
    /// # Arguments
    ///
    /// * `key` - the key of the data
    pub unsafe fn get_map_key_unchecked(&self, key: &str) -> Option<KeyValPair> {
        let str = CString::new(key).unwrap();
        KeyValPair::from_keyval_t(dy_get_map_key(self.val, str.as_ptr()))
    }

    /// decomposes the value into a hashmap if the value is a map
    pub fn decompose_map(self) -> Result<HashMap<String, Value>, Self> {
        if self.is_map() && self.owned {
            unsafe { Ok(self.decompose_map_unchecked()) }
        } else {
            Err(self)
        }
    }

    /// Decomposes the value (possibly a map) into a hashmap without checking the type of the value
    pub unsafe fn decompose_map_unchecked(self) -> HashMap<String, Value> {
        let mut rtn = HashMap::with_capacity(self.get_map_len_unchecked());
        let iter = dy_make_map_iter(self.val);
        let mut pair = dy_get_map_iter(self.val, iter);
        while pair.key != null() {
            let key = match CStr::from_ptr(pair.key).to_string_lossy() {
                Cow::Borrowed(s) => String::from(s),
                Cow::Owned(s) => s,
            };
            rtn.insert(key, Value::from_ptr(pair.val));
            pair = dy_get_map_iter(self.val, iter);
        }
        dy_dispose_map_iter(iter);
        let s = ManuallyDrop::new(self);
        dy_dispose_self(s.val);
        rtn
    }
}

macro_rules! impl_is_type {
    (
        $e:ident, $name:literal
            : $is:ident
    ) => {
        impl Value {
            #[doc = "Checks if the value is "]
            #[doc = $name]
            pub fn $is(&self) -> bool {
                self.get_type() == Type::$e
            }
        }
    };
}

macro_rules! impl_type_common {
    (
        $ty:ty, $e:ident, $name:literal
            : $is:ident, $get:ident, $uget:ident
    ) => {
        impl Value {
            /// Retrieves the internal data if the type is correct. Returns `None` otherwise.
            pub fn $get(&self) -> Option<$ty> {
                unsafe {
                    if self.$is() {
                        Some(self.$uget())
                    } else {
                        None
                    }
                }
            }
        }

        impl_is_type! { $e, $name: $is }
    };
}

macro_rules! impl_arr_type_common {
    (
        $ty:ty, $e:ident, $name:literal
            : $new:ident, $is:ident, $len:ident, $ulen:ident, $get_idx:ident, $uget_idx:ident
            => $imake:ident, $ilen:ident, $iget_idx:ident
    ) => {
        impl Value {
            #[doc = "Makes a new "]
            #[doc = $name]
            #[doc = " value"]
            ///
            /// # Arguments
            ///
            /// * `v` - the value to put
            pub fn $new(v: &[$ty]) -> Self {
                unsafe { Value::from_ptr($imake(v.as_ptr(), v.len() as u64)) }
            }

            /// Returns the length of the array if the value is an array
            pub fn $len(&self) -> Option<usize> {
                if !self.$is() {
                    None
                } else {
                    unsafe { Some(self.$ulen()) }
                }
            }

            /// Returns the length of the array without checking the type
            pub unsafe fn $ulen(&self) -> usize {
                $ilen(self.val) as usize
            }

            /// Returns the data of the entry at the given index in the internal data if the value
            /// is an array.
            ///
            /// # Arguments
            ///
            /// * `idx` - the index of the entry
            pub fn $get_idx(&self, idx: usize) -> Option<$ty> {
                if !self.$is() {
                    None
                } else if idx >= self.$len().unwrap() {
                    None
                } else {
                    Some(unsafe { self.$uget_idx(idx) })
                }
            }

            /// Returns the data of the entry at the given index without checking the type
            ///
            /// * `idx` - the index of the entry
            pub unsafe fn $uget_idx(&self, idx: usize) -> $ty {
                $iget_idx(self.val, idx as u64)
            }
        }

        impl_is_type! { $e, $name: $is }
    };
}

macro_rules! impl_type {
    (
        $ty:ty, $e:ident, $name:literal
            : $new:ident, $is:ident, $get:ident, $uget:ident
            => $imake:ident, $iget:ident
    ) => {
        impl Value {
            #[doc = "Makes a new "]
            #[doc = $name]
            #[doc = " value"]
            ///
            /// # Arguments
            ///
            /// * `v` - the value to put
            pub fn $new(v: $ty) -> Self {
                unsafe { Value::from_ptr($imake(v)) }
            }

            /// Retrieves the internal data without checking the type.
            pub unsafe fn $uget(&self) -> $ty {
                $iget(self.val)
            }
        }

        impl_type_common! { $ty, $e, $name: $is, $get, $uget }
    };
}

macro_rules! impl_arr_type {
    (
        $ty:ty, $e:ident, $name:literal
            : $new:ident, $is:ident, $len:ident,
              $ulen:ident, $get_idx:ident, $uget_idx:ident,
              $get:ident, $uget:ident
            => $imake:ident, $ilen:ident, $iget_idx:ident, $iget_data:ident
    ) => {
        impl Value {
            /// Retrieves the internal data if the type is correct. Returns `None` otherwise.
            pub fn $get(&self) -> Option<&[$ty]> {
                if !self.$is() {
                    None
                } else {
                    Some(unsafe { self.$uget() })
                }
            }

            /// Retrieves the internal data without checking the type
            pub unsafe fn $uget(&self) -> &[$ty] {
                from_raw_parts($iget_data(self.val), self.$ulen())
            }
        }

        impl_arr_type_common! {
            $ty, $e, $name
                : $new, $is, $len, $ulen, $get_idx, $uget_idx
                => $imake, $ilen, $iget_idx
        }
    };
}

impl_is_type! { Null, "null": is_null }

impl_type! {
    bool, Bool, "boolean"
        : new_bool, is_bool, get_bool, get_bool_unchecked
        => dy_make_b, dy_get_b
}

impl_type! {
    i64, Int, "integer"
        : new_int, is_int, get_int, get_int_unchecked
        => dy_make_i, dy_get_i
}

impl_type! {
    f64, Float, "float"
        : new_float, is_float, get_float, get_float_unchecked
        => dy_make_f, dy_get_f
}

impl_is_type! { Str, "string": is_str }

impl_arr_type_common! {
    bool, BoolArr, "boolean array"
        : new_bool_arr,
          is_bool_arr,
          get_bool_arr_len,
          get_bool_arr_len_unchecked,
          get_bool_by_idx,
          get_bool_by_idx_unchecked
        => dy_make_barr, dy_get_barr_len, dy_get_barr_idx
}

impl_arr_type! {
    i64, IntArr, "integer array"
        : new_int_arr,
          is_int_arr,
          get_int_arr_len,
          get_int_arr_len_unchecked,
          get_int_by_idx,
          get_int_by_idx_unchecked,
          get_int_arr,
          get_int_arr_unchecked
        => dy_make_iarr, dy_get_iarr_len, dy_get_iarr_idx, dy_get_iarr_data
}

impl_arr_type! {
    f64, FloatArr, "float array"
        : new_float_arr,
          is_float_arr,
          get_float_arr_len,
          get_float_arr_len_unchecked,
          get_float_by_idx,
          get_float_by_idx_unchecked,
          get_float_arr,
          get_float_arr_unchecked
        => dy_make_farr, dy_get_farr_len, dy_get_farr_idx, dy_get_farr_data
}

impl_type_common! {
    Vec<Value>, Arr, "generic array"
        : is_arr, get_arr, get_arr_unchecked
}

impl_is_type! { Map, "generic map": is_map }

impl Clone for Value {
    fn clone(&self) -> Self {
        unsafe {
            Value {
                val: dy_copy(self.val),
                owned: true,
            }
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                dy_dispose(self.val);
            }
        }
    }
}

impl<'a> KeyValPair<'a> {
    pub fn get_key(&self) -> &'a str {
        self.key
    }

    pub fn get_val(&self) -> &Value {
        &self.wrap
    }

    unsafe fn from_keyval_t(pair: dy_keyval_t) -> Option<Self> {
        if pair.key == null() {
            None
        } else {
            Some(KeyValPair {
                key: CStr::from_ptr(pair.key).to_str().unwrap(),
                wrap: Value::from_ptr_borrowed(pair.val),
            })
        }
    }
}

impl<'a> MapIter<'a> {
    /// makes an iterator of the generic map in the internal data
    ///
    /// # Arguments
    ///
    /// * `wrap` - the generic map
    pub fn new(wrap: &'a Value) -> Option<Self> {
        if !wrap.is_map() {
            None
        } else {
            unsafe { Some(MapIter::new_unchecked(wrap)) }
        }
    }

    /// makes an iterator of the value (which possibly is a generic map) in the internal data
    /// without checking the type of the value
    ///
    /// # Arguments
    ///
    /// * `wrap` - the value wrapper
    pub unsafe fn new_unchecked(wrap: &'a Value) -> Self {
        MapIter {
            wrap: wrap,
            iter: dy_make_map_iter(wrap.val),
        }
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = KeyValPair<'a>;
    fn next(&mut self) -> Option<KeyValPair<'a>> {
        unsafe { KeyValPair::from_keyval_t(dy_get_map_iter(self.wrap.val, self.iter)) }
    }
}

impl<'a> Drop for MapIter<'a> {
    fn drop(&mut self) {
        unsafe {
            dy_dispose_map_iter(self.iter);
        }
    }
}
