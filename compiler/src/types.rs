use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all="lowercase")]
pub enum FoundationVersion {
    Byzantium,
    Frontier,
    Homestead,
    SpuriousDragon,
    // more
}

impl Default for FoundationVersion {
    fn default() -> FoundationVersion {
        FoundationVersion::Byzantium
    }
}

impl From<FoundationVersion> for String {
    fn from(ver: FoundationVersion) -> String {
        match ver {
            FoundationVersion::Byzantium => "byzantium".to_string(),
            FoundationVersion::Homestead => "homestead".to_string(),
            FoundationVersion::Frontier => "frontier".to_string(),
            FoundationVersion::SpuriousDragon => "spuriousdragon".to_string(),
        }
    }
}

//TODO: USE THIS ENUM!
/// Language Enum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LanguageType {
    Solidity,
    Vyper,
    LLL,
    ASM,
}

/// Default Language is Solidity
impl Default for LanguageType {
    fn default() -> LanguageType {
        LanguageType::Solidity
    }
}
