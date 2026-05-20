use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub(crate) name: String,
    pub(crate) pattern: String
}

pub fn get_profiles() -> Result<Vec<Profile>> {
    let data = r#"
        [
            {
                "name" : "LüPa",
                "pattern" : "R0000"
            }
        ]
        "#;

    let ps: Vec<Profile> = serde_json::from_str(data).expect("JSON was not mallformed");

    Ok(ps)
}