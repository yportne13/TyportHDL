mod hir;
mod mir;
mod built_in;
mod ty;
//mod jit;
mod vm;
use hir::parse_to_hir;
use mir::hir_to_mir;
//use typort_parser::simple_example::*;
use core::mem;

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub data: T,
}

impl<T> Span<T> {
    pub fn map<F, O>(self, f: F) -> Span<O>
    where
        F: Fn(T) -> O
    {
        Span {
            data: f(self.data)
        }
    }
}

impl<T> From<typort_parser::simple_example::Span<T>> for Span<T> {
    fn from(value: typort_parser::simple_example::Span<T>) -> Self {
        Span { data: value.data }
    }
}

type Line = usize;
type Col = usize;

#[derive(Debug)]
pub struct Diagnostic {
    pub msg: String,
    pub range: ((Line, Col), (Line, Col)),
}

fn main() {
    /*let ret = unsafe { run_code::<i64, i64>(FIB, 11) };
    println!("{ret:?}");
    let ret = unsafe { run_code::<i64, i64>(WHILE, 11) };
    println!("{ret:?}");
    let ret = unsafe { run_code::<i64, i64>(FOR, 11) };
    println!("{ret:?}");*/
    println!("##### fib #####\n");
    let ret = run_code_vm(FIB);
    println!("{ret:?}");
    println!("\n##### while #####\n");
    let ret = run_code_vm(WHILE);
    println!("{ret:?}");
    println!("\n##### for #####\n");
    let ret = run_code_vm(FOR);
    println!("{ret:?}");
    println!("\n##### string #####\n");
    let ret = run_code_vm(STRING);
    println!("{ret:?}");
    println!("\n##### array #####\n");
    let _ = run_code_vm(ARRAY);
}

/*unsafe fn run_code<I, O>(code: &str, input: I) -> Result<O, String> {
    let mut jit = jit::JIT::default();
    let ast = typort_parser::simple_example::file().run(code).ok_or("parse error")?;
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
}*/

fn run_code_vm(code: &str) -> Result<vm::Value, String> {
    let (_, parse_fail, _) = typort_parser::simple_example::file().run_with_out(code, Default::default());
    if !parse_fail.is_empty() {
        println!("parse fail at {:?}", parse_fail);
    }
    let ast = typort_parser::simple_example::file().run(code).ok_or("parse error")?;
    let hir = parse_to_hir(ast);
    //println!("hir: {:#?}", hir);
    let mir = hir_to_mir(hir);
    //println!("mir: {:#?}", mir);
    let mut vm = vm::Interpreter::new(mir);
    let main = vm.classes.get("main").unwrap().clone();
    let ret = vm.translate_block(&main.block);
    //let ret = vm.run();
    Ok(ret)
}

const FIB: &str = include_str!("../../examples/fib.typort");
const FOR: &str = include_str!("../../examples/for.typort");
const WHILE: &str = include_str!("../../examples/while.typort");
const STRING: &str = include_str!("../../examples/string.typort");
const ARRAY: &str = include_str!("../../examples/array.typort");

/*#[test]
fn test_jit() {
    let ret = unsafe { run_code::<i64, i64>(FIB, 11) };
    println!("{ret:?}");
    let ret = unsafe { run_code::<i64, i64>(WHILE, 11) };
    println!("{ret:?}");
    let ret = unsafe { run_code::<i64, i64>(FOR, 11) };
    println!("{ret:?}");
}*/

#[test]
fn test_vm() {
    let ret = run_code_vm(FIB);
    println!("{ret:?}");
    let ret = run_code_vm(WHILE);
    println!("{ret:?}");
    let ret = run_code_vm(FOR);
    println!("{ret:?}");
}
