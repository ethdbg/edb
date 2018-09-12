use pyo3::FromPyObject;

pub enum AstValue {
    Ann(Annotation),
    FunctionDef(FunctionDef),
}

pub struct Annotation {
    target: Name,
    annotation: Call,
    simple: i32,
    lineno: usize, // line number should never be negative
    col_offset: i32,
    source_code: String,
}

pub struct FunctionDef {
    name: String,
    args: Arguments,
    returns: Name,
    lineno: usize,
    col_offset: i32,
    source_code: String
}

pub struct Arguments {
    

}

pub struct Name {
    id: String,
    ctx: Context,
    linno: usize,
    col_offset: i32,
    source_code: String,
}

pub enum Context {
    Load,
    Store,
    Del,
}

/*
impl FromPyObject for Value {
        

}

*/
