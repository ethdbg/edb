use log::*;
use std::{
    path::PathBuf,
    fs::File,
    io::Read,
};
use pyo3::prelude::*;
use super::err::VyError;
use super::types::parse::AstValue;

// TODO: use from_code instead of global vyper mod
pub fn parse(code_file: PathBuf) -> VyError<()> {
    let mut code = String::new();
    let mut file = File::open(code_file.as_path())?;
    info!("Read {} bytes from vyper code", file.read_to_string(&mut code)?);
    let arb_list = vec![0, 1, 3];
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = py.import("sys");
    let parser = py.import("vyper.parser.parser")?;
    let res = parser.call1("parse", code.as_str());
    let res = match res {
        Err(err) => {
            err.print(py);
            panic!("Failed due to error");
        }
        Ok(res) => res,
    };
    info!("RES: {}", res);
    let test: PyResult<Vec<String>> = res.extract();

    let test = match test {
        Err(err) => { 
            err.print(py);
            panic!("Error");
        },
        Ok(res) => res,
    };
    info!("TEST: {:?}", test);
    let res: PyDict = res.try_into()?;
    // let res: PyDict = res.extract()?;
    Ok(())
}

pub fn get_contracts_and_defs_and_globals(parsed: PyList) -> PyResult<()> {
    unimplemented!(); 
}
/*
pub fn parse_tree_to_lll(parsed_code: ?, code: &str, runtime: bool) -> ? {
    unimplemented!();
}

*/
