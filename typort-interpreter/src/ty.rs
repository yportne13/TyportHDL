
pub enum Type {
    Unit,
    Bool,
    I64,
    F64,
    Tuple2(Box<Type>, Box<Type>),
    Tuple3(Box<Type>, Box<Type>, Box<Type>),
    Tuple4(Box<Type>, Box<Type>, Box<Type>, Box<Type>),
    Func(Vec<Type>, Box<Type>),
    Own(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "Unit"),
            Type::Bool => write!(f, "Bool"),
            Type::I64 => write!(f, "Int"),
            Type::F64 => write!(f, "Float"),
            Type::Tuple2(a, b) => write!(f, "({a}, {b})"),
            Type::Tuple3(a, b, c) => write!(f, "({a}, {b}, {c})"),
            Type::Tuple4(a, b, c, d) => write!(f, "({a}, {b}, {c}, {d})"),
            Type::Func(i, o) => write!(
                f,
                "({}) -> {}",
                i.iter()
                    .map(|t| format!("{t}"))
                    .reduce(|a, b| format!("{a}, {b}"))
                    .unwrap_or("".to_owned()),
                o
            ),
            Type::Own(name) => write!(f, "{name}"),
        }
    }
}
