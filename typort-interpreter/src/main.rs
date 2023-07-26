mod hir;
mod mir;
mod jit;
use hir::parse_to_hir;
use mir::hir_to_mir;
use typort_parser::simple_example::*;
use core::mem;

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub data: T,
}

impl<T> From<typort_parser::simple_example::Span<T>> for Span<T> {
    fn from(value: typort_parser::simple_example::Span<T>) -> Self {
        Span { data: value.data }
    }
}

fn main() {
    let ret = unsafe { run_code::<i64, i64>(r#"
fn fib(n : i64) -> i64 {
    if n == 0 {
        return 0
    } else {
        if n == 1 {
            return 1
        } else {
            return fib(n - 1) + fib(n - 2)
        }
    }
}

fn main() -> i64 {
    return fib(10)
}
    "#, 11) };
    println!("{ret:?}");
    let ret = unsafe { run_code::<i64, i64>(r#"
fn testloop(n : i64) -> i64 {
    let x = 0
    let ret = 0
    while(x != n) {
        x = x + 1
        ret = ret + x
    }
    return ret
}

fn main() -> i64 {
    return testloop(100)
}
    "#, 11) };
    println!("{ret:?}");
}

unsafe fn run_code<I, O>(code: &str, input: I) -> Result<O, String> {
    let mut jit = jit::JIT::default();
    let ast = file().run(code).ok_or("parse error")?;
    let hir = parse_to_hir(ast);
    let mir = hir_to_mir(hir);
    // Pass the string to the JIT, and it returns a raw pointer to machine code.
    let code_ptr = jit.compile(mir)?;
    // Cast the raw pointer to a typed function pointer. This is unsafe, because
    // this is the critical point where you have to trust that the generated code
    // is safe to be called.
    let code_fn = mem::transmute::<_, fn(I) -> O>(code_ptr);
    // And now we can call it!
    Ok(code_fn(input))
}
