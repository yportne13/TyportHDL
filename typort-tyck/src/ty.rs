
#[derive(Clone, Debug)]
pub enum Type {
    Bool,
    Num,
    String,
    Class(String, Vec<Type>),
    Fn(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
}
