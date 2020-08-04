use crate::value::*;
use libloading::{Library, Symbol};
use std::env::current_dir;
use std::path::PathBuf;

/// Indicates a DLL using `dy`
pub struct Module {
    lib: Library,
}

/// Indicates an exported function using `dy`
pub struct Function<'lib> {
    sym: Symbol<'lib, unsafe extern "C" fn(args: *const ValuePtr, len: usize) -> ValuePtr>,
}

#[cfg(unix)]
fn get_dll_path(name: &str, dir: &str) -> PathBuf {
    let mut rtn = PathBuf::new();
    rtn.push(current_dir().unwrap());
    rtn.push(dir);
    rtn.push(format!("lib{}.so", name));
    rtn
}

#[cfg(windows)]
fn get_dll_path(name: &str, dir: &str) -> PathBuf {
    let mut rtn = PathBuf::new();
    rtn.push(current_dir().unwrap());
    rtn.push(dir);
    rtn.push(format!("{}.dll", name));
    rtn
}

impl Module {
    /// Creates a new `Module` instance from an existing DLL
    /// 
    /// # Arguments
    /// 
    /// * `name` - the name of the DLL
    /// * `search_paths` - the list of directories where the DLL may be located in
    pub fn new(name: &str, search_paths: &[&str]) -> Option<Module> {
        for search_path in search_paths {
            let dll_path = get_dll_path(name, search_path);
            if dll_path.exists() {
                return match Library::new(dll_path) {
                    Ok(lib) => Some(Module { lib: lib }),
                    Err(_) => None,
                };
            }
        }
        None
    }

    /// Retrieves an exported function from the DLL
    /// 
    /// # Arguments
    /// 
    /// * `name` - the name of the function
    pub fn get_fn<'lib>(&'lib self, name: &str) -> Option<Function<'lib>> {
        match unsafe { self.lib.get(name.as_bytes()) } {
            Ok(sym) => Some(Function { sym: sym }),
            Err(_) => None,
        }
    }
}

impl<'lib> Function<'lib> {
    /// Invokes the exported function
    /// 
    /// # Arguments
    /// 
    /// * `args` - the arguments
    pub fn call_with_borrowed(&self, args: &[Borrowed<'_>]) -> Owned {
        let list_ptr: Vec<ValuePtr> = args.iter().map(|arg| arg.get_ptr()).collect();
        let rtn = unsafe { (self.sym)(list_ptr.as_ptr(), list_ptr.len()) };
        unsafe { Owned::from_ptr(rtn) }
    }

    /// Calls the exported function and disposes arguments after the invocation
    /// 
    /// # Arguments
    /// 
    /// * `args` - the arguments
    pub fn call(&self, args: Vec<Owned>) -> Owned {
        let list_ptr: Vec<ValuePtr> = args.into_iter().map(|arg| arg.into_ptr()).collect();
        let rtn = unsafe { (self.sym)(list_ptr.as_ptr(), list_ptr.len()) };
        let rtn = unsafe { Owned::from_ptr(rtn) };
        for ptr in list_ptr {
            unsafe { Owned::from_ptr(ptr) };
        }
        rtn
    }
}
