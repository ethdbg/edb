use pyo3::FromPyObject;

pub enum AstValue {
    Ann(Annotation),
    FunctionDef(FunctionDef),
}

pub struct Annotation {
    simple: i32,
    lineno: usize, // line number should never be negative
    col_offset: i32,
    source_code: String,
}

pub struct FunctionDef {
    name: String,
    lineno: usize,
    col_offset: i32,
    source_code: String
}
/*
impl FromPyObject for Value {
        

}

*/
