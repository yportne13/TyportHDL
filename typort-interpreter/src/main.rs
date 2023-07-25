mod jit;
use typort_parser::simple_example::*;
use core::mem;

fn main() {
    let ret = unsafe { run_code::<i64, i64>(r#"
fn fib(n : i64) -> i64 {
    if n == 0 {
        return 0;
    } else {
        if n == 1 {
            return 1;
        } else {
            return fib(n - 1) + fib(n - 2);
        };
    };
}

fn main() -> i64 {
    return fib(11);
}
    "#, 11) };
    println!("{:?}", ret);
    println!("Hello, world!");
}

unsafe fn run_code<I, O>(code: &str, input: I) -> Result<O, String> {
    let mut jit = jit::JIT::default();
    let code = file().run(code).ok_or("parse error")?;
    // Pass the string to the JIT, and it returns a raw pointer to machine code.
    let code_ptr = jit.compile(code)?;
    // Cast the raw pointer to a typed function pointer. This is unsafe, because
    // this is the critical point where you have to trust that the generated code
    // is safe to be called.
    let code_fn = mem::transmute::<_, fn(I) -> O>(code_ptr);
    // And now we can call it!
    Ok(code_fn(input))
}
