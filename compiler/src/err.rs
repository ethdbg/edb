use failure::Fail;

#[derive(Fail, Debug, Clone)]
pub enum LanguageError {
    #[fail(display = "Could not obtain a Source Map")]
    SourceMap(#[cause] SourceMapError)
}

#[derive(Fail, Debug, Clone)]
pub enum SourceMapError {
    #[fail(display = "Unknown Jump Variant: {}", _0)]
    UnknownJump(String),
    #[fail(display = "Decode Error")]
    Decode(#[cause] std::num::ParseIntError),
}
