use crate::vm::{Value, HeapValue, Interpreter};


pub fn bi_print(vm: &mut Interpreter, args: Vec<Value>) -> Value {
    for value in args {
        match value {
            Value::HeapId(idx) => {
                match vm.heap.get(&idx).unwrap() {
                    HeapValue::Vec(x) => {println!("{x:?}")},
                    HeapValue::String(s) => {println!("{s}");},
                    //HeapValue::Class(_) => todo!(),
                }
            },
            _ => {println!("{value:?}");},
        }
    }
    
    Value::Unit
}

pub fn bi_array(vm: &mut Interpreter, args: Vec<Value>) -> Value {
    let idx = vm.heap.len();
    vm.heap.insert(idx, HeapValue::Vec(args));
    Value::HeapId(idx)
}
