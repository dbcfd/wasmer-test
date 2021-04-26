use common::*;

pub trait Checker {
    fn check(&self, s: &str) -> Option<String>;
}

#[derive(Default)]
pub struct TestChecker {

}

impl Checker for TestChecker {
    fn check(&self, s: &str) -> Option<String> {
        if s.contains("test") {
            Some("FOUND".to_string())
        } else {
            None
        }
    }
}

extern "C" {
    pub fn host_memory() -> *const u8;
}

#[no_mangle]
pub fn new() -> u32 {
    let checker = Box::new(TestChecker::default());
    let ptr = Box::into_raw(checker);
    let plugin = Plugin {
        address: ptr as u32,
        name: "Plugin".to_string(),
    };
    common::write(&plugin).unwrap()
}

#[no_mangle]
pub fn run(checker: u32, addr: u32, len: u32) -> u32 {
    let checker = checker as *mut std::ffi::c_void;
    if checker == std::ptr::null_mut() {
        panic!("Null pointer");
    }
    let checker = unsafe {
        Box::from_raw(checker as *mut TestChecker)
    };

    let mem = unsafe {
        let mem = host_memory();
        mem.offset(addr as _)
    };
    if mem == std::ptr::null() {
        panic!("Null pointer");
    }

    let s = unsafe {
        std::slice::from_raw_parts(MEMORY_START as *const u8, len as _)
    };

    let s = std::str::from_utf8(s).unwrap();
    let res = checker.check(s);
    std::mem::forget(checker);
    common::write(&res).unwrap()
}
