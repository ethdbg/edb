use std::path::PathBuf;
use pyo3::prelude::*;
use super::err::VyError;

pub fn parse(code: PathBuf) -> VyError<PyList> {

    let arb_list = vec![0, 1, 3];
    let gil = Python::acquire_gil();
    let py = gil.python();
    let vyper = py.import("vyper")?;
    PyList::new(arb_list.as_slice())
}


/*
get_contracts_and_defs_and_globals(parsed: PyList) -> PyResult<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
}

parse_tree_to_lll(parsed_code: ?, code: &str, runtime: bool) -> ? {



}

*/
