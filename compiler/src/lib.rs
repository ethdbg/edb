



/// Functions 
pub trait Compiler {
    /// Get a PC from a line number in the Source Code
    fn pc_from_lineno(lineno: usize) -> usize;
    /// Other Functions ... (TBA)
}
