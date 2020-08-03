#![allow(non_upper_case_globals)]

mod bindings;

use bindings::*;
use std::borrow::Cow;
use std::clone::Clone;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::null;
use std::slice::from_raw_parts;

/// A pointer to a `dy` value.
pub type ValuePtr = dy_t;

/// The type indicating a `dy` value. Contains a `ValuePtr` instance.
#[derive(Debug)]
pub struct Value {
    ptr: ValuePtr,
}

/// The type indicating a borrowed `dy` value.
#[derive(Debug)]
pub struct Borrowed<'a> {
    val: Value,
    phantom: PhantomData<&'a Value>,
}

/// The type indicating an owned `dy` value. Automatically deallocates the memory.
#[derive(Debug)]
pub struct Owned {
    val: Value,
}

/// Indicates a key-value pair
#[derive(Debug)]
pub struct KeyValPair<'a> {
    key: &'a str,
    val: Value,
}

/// Indicates an iterator of an generic array
#[derive(Debug)]
pub struct ArrIter<'a> {
    /// the array
    val: &'a Value,
    idx: usize,
}

/// Indicates an iterator of a generic map
#[derive(Debug)]
pub struct MapIter<'a> {
    /// the generic map
    val: &'a Value,
    iter: dy_iter_t,
}

impl Value {
    /// Creates a new value instance from a pointer
    ///
    /// # Arguments
    ///
    /// * `ptr` - the pointer to wrap
    unsafe fn from_ptr(ptr: ValuePtr) -> Self {
        Value { ptr: ptr }
    }

    /// Returns the type of the value
    pub fn get_type(&self) -> Type {
        Type::from_dy_type_t(unsafe { dy_get_type(self.ptr) }).unwrap()
    }

    /// Clones the value
    pub fn copy(&self) -> Owned {
        unsafe { Owned::from_ptr(dy_copy(self.ptr)) }
    }

    /// Borrow a value
    pub fn borrow<'a>(&'a self) -> Borrowed<'a> {
        unsafe { Borrowed::from_ptr(self.ptr) }
    }

    /// Makes a new null value
    pub fn new_null() -> Owned {
        unsafe { Owned::from_ptr(dy_make_null()) }
    }

    /// Makes a new string
    ///
    /// # Arguments
    ///
    /// * `v` - the string to copy
    pub fn new_str(v: &str) -> Owned {
        let s = CString::new(v).unwrap();
        unsafe { Owned::from_ptr(dy_make_str(s.as_ptr())) }
    }

    /// Makes a new generic array
    ///
    /// # Arguments
    ///
    /// * `v` - the array to put
    pub fn new_arr(v: Vec<Owned>) -> Owned {
        let v: Vec<ValuePtr> = v.into_iter().map(|w| w.into_ptr()).collect();
        unsafe { Owned::from_ptr(dy_make_arr(v.as_ptr(), v.len() as u64)) }
    }

    /// Makes a new generic map
    ///
    /// # Arguments
    ///
    /// * `v` - the array of key-value pairs to copy
    pub fn new_map(v: Vec<(&str, Owned)>) -> Owned {
        let v: Vec<(CString, ValuePtr)> = v
            .into_iter()
            .map(|tup| {
                let (s, w) = tup;
                (CString::new(s).unwrap(), w.into_ptr())
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

        unsafe { Owned::from_ptr(dy_make_map(vv.as_ptr(), vv.len() as u64)) }
    }
}

impl Borrowed<'_> {
    /// Creates a new borrowed value instance from a pointer
    ///
    /// # Arguments
    ///
    /// * `ptr` - the pointer to wrap
    unsafe fn from_ptr(ptr: ValuePtr) -> Self {
        Borrowed {
            val: Value { ptr: ptr },
            phantom: PhantomData,
        }
    }
}

impl<'a> Deref for Borrowed<'a> {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.val
    }
}

impl Owned {
    /// Creates a new value instance from a pointer
    ///
    /// # Arguments
    ///
    /// * `ptr` - the pointer to wrap
    pub unsafe fn from_ptr(ptr: ValuePtr) -> Self {
        Owned {
            val: Value::from_ptr(ptr),
        }
    }

    /// Returns the internal pointer
    pub fn into_ptr(self) -> ValuePtr {
        let s = ManuallyDrop::new(self);
        s.val.ptr
    }
}

impl Deref for Owned {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.val
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { dy_dispose(self.val.ptr) }
    }
}

macro_rules! def_type {
    (
        $(
            $name:ident($doc:literal, $as_val:ident)
                = $internal:ident,
                  $as:ident, $is:ident
        );+ $(;)?
    ) => {
        /// Indicates the type of `dy` values
        #[derive(Debug, PartialEq, Clone, Copy)]
        #[allow(non_upper_case_global)]
        pub enum Type {
            $(
                #[doc = "Indicates a "]
                #[doc = $doc]
                #[doc = " type"]
                $name = $internal as isize,
            )+
        }

        impl Type {
            fn from_dy_type_t(i: _dy_type_t) -> Result<Self, ()> {
                match i {
                    $($internal => Ok(Type::$name),)+
                    _ => Err(())
                }
            }
        }

        $(
            #[doc = "Indicates a reference to a "]
            #[doc = $doc]
            #[doc = " type value"]
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy)]
            pub struct $as_val<'a> {
                val: &'a Value,
            }

            impl Value {
                #[doc = "If the value is a "]
                #[doc = $doc]
                #[doc = " type value, returns a reference to the given value"]
                pub fn $as<'a>(&'a self) -> Option<$as_val<'a>> {
                    if self.$is() {
                        Some($as_val {
                            val: self
                        })
                    } else {
                        None
                    }
                }

                #[doc = "Returns `true` if the value is a "]
                #[doc = $doc]
                #[doc = " type value"]
                pub fn $is(&self) -> bool {
                    self.get_type() == Type::$name
                }
            }
        )+
    };
}

def_type! {
    Null("null", AsNullValue)
        = _dy_type_t_dy_type_null,
          as_null, is_null;
    Bool("boolean", AsBoolValue)
        = _dy_type_t_dy_type_b,
          as_bool, is_bool;
    Int("8-byte integer", AsIntValue)
        = _dy_type_t_dy_type_i,
          as_int, is_int;
    Float("double-precision floating point", AsFloatValue)
        = _dy_type_t_dy_type_f,
          as_float, is_float;
    Str("string", AsStrValue)
        = _dy_type_t_dy_type_str,
          as_str, is_str;
    BoolArr("boolean array", AsBoolArrValue)
        = _dy_type_t_dy_type_barr,
          as_bool_arr, is_bool_arr;
    Bytes("byte array", AsBytesValue)
        = _dy_type_t_dy_type_bytes,
        as_bytes, is_bytes;
    IntArr("integer array", AsIntArrValue)
        = _dy_type_t_dy_type_iarr,
          as_int_arr, is_int_arr;
    FloatArr("floating point number array", AsFloatArrValue)
        = _dy_type_t_dy_type_farr,
          as_float_arr, is_float_arr;
    Arr("generic array", AsArrValue)
        = _dy_type_t_dy_type_arr,
          as_arr, is_arr;
    Map("generic map", AsMapValue)
        = _dy_type_t_dy_type_map,
          as_map, is_map;
}

macro_rules! impl_primitive_types {
    ($($as_val:ident($doc:literal, $ty:ty): $new:ident => $imake:ident, $iget:ident);+ $(;)?) => {
        $(
            impl Value {
                #[doc = "Makes a new "]
                #[doc = $doc]
                #[doc = " value"]
                ///
                /// # Arguments
                ///
                /// * `v` - the value to put
                pub fn $new(v: $ty) -> Owned {
                    unsafe { Owned::from_ptr($imake(v)) }
                }
            }

            /// Retrieves the internal data
            impl<'a> $as_val<'a> {
                pub fn get(&self) -> $ty {
                    unsafe { $iget(self.val.ptr) }
                }
            }
        )+
    };
}

impl_primitive_types! {
    AsBoolValue("boolean", bool)
        : new_bool  => dy_make_b, dy_get_b;
    AsIntValue("8-byte integer", i64)
        : new_int   => dy_make_i, dy_get_i;
    AsFloatValue("double-precision floating point", f64)
        : new_float => dy_make_f, dy_get_f;
}

impl<'a> AsStrValue<'a> {
    /// Returns the length of the string
    pub fn len(&self) -> usize {
        unsafe { dy_get_str_len(self.val.ptr) as usize }
    }

    /// Makes a string instance from this value
    pub fn get(&self) -> String {
        match unsafe { CStr::from_ptr(dy_get_str_data(self.val.ptr)) }.to_string_lossy() {
            Cow::Borrowed(s) => String::from(s),
            Cow::Owned(s) => s,
        }
    }
}

macro_rules! impl_array_types {
    (
        $($as_val:ident($doc:literal, $ty:ty)
            : $new:ident
            => $imake:ident, $ilen:ident, $iget:ident $(, $idata:ident)?);+ $(;)?
    ) => {
        $(
            impl Value {
                #[doc = "Makes a new "]
                #[doc = $doc]
                #[doc = " value"]
                ///
                /// # Arguments
                ///
                /// * `v` - the value to put
                pub fn $new(v: &[$ty]) -> Owned {
                    unsafe { Owned::from_ptr($imake(v.as_ptr(), v.len() as u64)) }
                }
            }

            impl<'a> $as_val<'a> {
                /// Returns the length of the array
                pub fn len(&self) -> usize {
                    unsafe { $ilen(self.val.ptr) as usize }
                }

                /// Returns the data of the entry at the given index
                ///
                /// * `idx` - the index of the entry
                pub fn at(&self, idx: usize) -> Option<$ty> {
                    if idx >= self.len() {
                        None
                    } else {
                        unsafe { Some($iget(self.val.ptr, idx as u64)) }
                    }
                }

                $(
                    /// Returns the internal data of the array
                    pub fn data(&self) -> &[$ty] {
                        unsafe { from_raw_parts($idata(self.val.ptr), self.len()) }
                    }
                )?
            }
        )+
    };
}

impl_array_types! {
    AsBoolArrValue("boolean array", bool)
        : new_bool_arr
        => dy_make_barr, dy_get_barr_len, dy_get_barr_idx;
    AsBytesValue("byte array", u8)
        : new_bytes
        => dy_make_bytes, dy_get_bytes_len, dy_get_bytes_idx, dy_get_bytes_data;
    AsIntArrValue("integer array", i64)
        : new_int_arr
        => dy_make_iarr, dy_get_iarr_len, dy_get_iarr_idx, dy_get_iarr_data;
    AsFloatArrValue("floating point number array", f64)
        : new_float_arr
        => dy_make_farr, dy_get_farr_len, dy_get_farr_idx, dy_get_farr_data;
}

impl<'a> AsArrValue<'a> {
    /// Returns the length of the array
    pub fn len(&self) -> usize {
        unsafe { dy_get_arr_len(self.val.ptr) as usize }
    }

    /// Returns the data of the entry at the given index
    ///
    /// * `idx` - the index of the entry
    pub fn at(&self, idx: usize) -> Option<Borrowed<'a>> {
        if idx >= self.len() {
            None
        } else {
            unsafe { Some(Borrowed::from_ptr(dy_get_arr_idx(self.val.ptr, idx as u64))) }
        }
    }

    /// Returns the iterator of this array
    pub fn iter(&self) -> ArrIter<'a> {
        ArrIter {
            val: self.val,
            idx: 0,
        }
    }
}

impl<'a> AsMapValue<'a> {
    /// Returns the size of the map
    pub fn size(&self) -> usize {
        unsafe { dy_get_map_len(self.val.ptr) as usize }
    }

    /// Returns the data with the given key
    ///
    /// # Arguments
    ///
    /// * `key` - the key of the data
    pub fn at(&self, key: &str) -> Option<KeyValPair<'a>> {
        let str = CString::new(key).unwrap();
        unsafe { KeyValPair::from_keyval_t(dy_get_map_key(self.val.ptr, str.as_ptr())) }
    }

    /// Returns the iterator of this map
    pub fn iter(&self) -> MapIter<'a> {
        MapIter {
            val: self.val,
            iter: unsafe { dy_make_map_iter(self.val.ptr) },
        }
    }
}

impl<'a> KeyValPair<'a> {
    pub fn get_key(&self) -> &'a str {
        self.key
    }

    pub fn get_val(&self) -> &Value {
        &self.val
    }

    unsafe fn from_keyval_t(pair: dy_keyval_t) -> Option<Self> {
        if pair.key == null() {
            None
        } else {
            Some(KeyValPair {
                key: CStr::from_ptr(pair.key).to_str().unwrap(),
                val: Value::from_ptr(pair.val),
            })
        }
    }
}

impl<'a> Iterator for ArrIter<'a> {
    type Item = Borrowed<'a>;
    fn next(&mut self) -> Option<Borrowed<'a>> {
        let len = unsafe { dy_get_arr_len(self.val.ptr) as usize };
        if self.idx == len {
            None
        } else {
            unsafe {
                let ptr = dy_get_arr_idx(self.val.ptr, self.idx as u64);
                self.idx += 1;
                Some(Borrowed::from_ptr(ptr))
            }
        }
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = KeyValPair<'a>;
    fn next(&mut self) -> Option<KeyValPair<'a>> {
        unsafe { KeyValPair::from_keyval_t(dy_get_map_iter(self.val.ptr, self.iter)) }
    }
}

impl<'a> Drop for MapIter<'a> {
    fn drop(&mut self) {
        unsafe {
            dy_dispose_map_iter(self.iter);
        }
    }
}
