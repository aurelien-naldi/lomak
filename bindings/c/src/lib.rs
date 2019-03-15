use std::marker::PhantomData;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use lomak::{
    func::expr::Expr,
    func::variables::VariableNamer,
    model::LQModel,
    model::io,
};


// Helper macro to import a C string into rust
// FIXME: does it leak?
macro_rules! import_string {
    ( $s: expr ) => {unsafe { CStr::from_ptr($s).to_str().unwrap() }};
}

/*
 * To share a rust struct to C, wrap it as a named void pointer
 * on top of which we can implement pseudo-safe callbacks and free
 */
#[repr(C)]
pub struct CPtr<T> {
    ptr: *mut libc::c_void,
    _p: PhantomData<T>,
}
pub type ExprPtr = CPtr<Expr>;
pub type ModelPtr = CPtr<LQModel>;


/* ***************************** EXPR API *********************************** */

#[no_mangle] pub extern fn new__expr_bool(b:bool) -> ExprPtr {
    ExprPtr::new( Expr::from_bool(b) )
}

#[no_mangle] pub extern fn str__expr(e: ExprPtr) -> *const c_char {
    share_rust_string( format!("{}", e.borrow()) )
}

#[no_mangle] pub extern fn drop__expr(e: ExprPtr) {
    ExprPtr::drop(e)
}

#[no_mangle] pub extern fn expr_not(e: ExprPtr) -> ExprPtr {
    ExprPtr::new(e.borrow().not())
}

#[no_mangle] pub extern fn expr_simplify(e: ExprPtr) -> ExprPtr {
    ExprPtr::new_or_null( e.borrow().simplify() )
}

#[no_mangle] pub extern fn expr_nnf(e: ExprPtr) -> ExprPtr {
    ExprPtr::new_or_null( e.borrow().nnf() )
}

#[no_mangle] pub extern fn expr_and(e: ExprPtr, e2: ExprPtr) -> ExprPtr {
    ExprPtr::new( e.borrow().and(e2.borrow()) )
}

#[no_mangle] pub extern fn expr_or(e: ExprPtr, e2: ExprPtr) -> ExprPtr {
    ExprPtr::new( e.borrow().or(e2.borrow()) )
}


/* ***************************** MODEL API *********************************** */

#[no_mangle] pub extern fn load_model(filename: *const c_char) -> ModelPtr {
    match io::load_model( import_string!(filename), None ) {
        Err(_) => CPtr::null_ptr(),
        Ok(m) => ModelPtr::new( m )
    }
}

#[no_mangle] pub extern fn new__model() -> ModelPtr {
    ModelPtr::new( LQModel::new() )
}

#[no_mangle] pub extern fn str__model(m: ModelPtr) -> *const c_char {
    share_rust_string( format!("{}", m.borrow()) )
}

#[no_mangle] pub extern fn drop__model(m: ModelPtr) {
    ModelPtr::drop(m)
}

/*
#[no_mangle] pub extern fn model_get_expr(mut model:ModelPtr, name: *const c_char) -> ExprPtr {
    ExprPtr::new( model.borrow_mut().get_var_from_name(import_string!(name)).as_expr() )
}

#[no_mangle] pub extern fn model_primes(model:ModelPtr) {
    model.borrow().primes();
}

#[no_mangle] pub extern fn model_fixpoints(model:ModelPtr) {
    model.borrow().stable();
}
*/

#[no_mangle] pub extern fn model_rename(model:ModelPtr, source: *const c_char, target: *const c_char) {
    model.borrow_mut().rename( import_string!(source), String::from(import_string!(target)));
}


/* ***************************** Handling strings *********************************** */

#[no_mangle] pub extern fn drop__string(e: *const c_char) {
    // FIXME: free this string
}

fn share_rust_string(s: String) -> *const c_char {
    let s = CString::new( s ).unwrap();
    let p = s.as_ptr();
    std::mem::forget(s);  // FIXME: does this leak?
    p
}



/* ***************************** Wrapping C Pointers ******************************* */

// Implementation of sharing blind structs to C and borrowing them back
// TODO: instead of boxing it, should we wrap it in a Rc<RefCell<T>> ?
impl<T> CPtr<T> {

    pub fn new(obj: T) -> CPtr<T> {
        let ptr = Box::into_raw( Box::new(obj) ) as *mut libc::c_void;
        CPtr { ptr: ptr, _p: PhantomData }
    }

    pub fn new_or_null(opt: Option<T>) -> CPtr<T> {
        match opt {
            Some(obj) => CPtr::new(obj),
            None      => CPtr::null_ptr(),
        }
    }

    pub fn null_ptr() -> CPtr<T> {
        CPtr { ptr: std::ptr::null_mut(), _p: PhantomData }
    }

    pub fn borrow<'a>(&'a self) -> &'a T {
        unsafe { (self.ptr as *mut T).as_ref().unwrap() }
    }

    pub fn borrow_mut<'a>(&'a self) -> &'a mut T {
        unsafe { (self.ptr as *mut T).as_mut().unwrap() }
    }

    pub fn drop(self) {
        unsafe { drop(Box::from_raw( self.ptr as *mut T )); }
    }
}

