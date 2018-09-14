use serde_derive::*;


pub enum FoundationVersion {
    Byzantium,
    Frontier,
    Homestead,
    SpuriousDragon,
    // more
}

impl From<FoundationVersion> for String {
    fn from(ver: &FoundationVersion) -> String {
        match *ver {
            FoundationVersion::Byzantium => "byzantium".to_string(),
            FoundationVersion::Homestead => "homestead".to_string(),
            FoundationVersion::Frontier => "frontier".to_string(),
        }
    }
}
/// Language Enum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Language {
    Solidity,
    Vyper,
    LLL,
    ASM,
}
