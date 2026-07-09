pub mod settings;

use crate::profiles::settings::fetch_profile_location;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Profile {
    pub(crate) name: String,
    pub(crate) pattern: String,
    pub column: String,
    pub nr: String
}

pub fn get_profiles() -> Result<Vec<Profile>> {
    let profile_location = fetch_profile_location();
    let json_data = fs::read_to_string(&profile_location);

    match json_data {
        Ok(data) => {
            let ps: Vec<Profile> = match serde_json::from_str(&*data) {
                Ok(data) => data,
                Err(_e) => {
                    let mut file = File::create(&profile_location).unwrap();
                    let json_data = serde_json::to_string_pretty(&Vec::<Profile>::new())?;
                    file.write(json_data.as_bytes()).expect("can't save profiles");
                    Vec::new()
                }
            };
            Ok(ps)
        }
        Err(_) => {
            Ok(Vec::new())
        }
    }
}

pub fn set_profiles(n:String, p: String, c: String, num: String) -> Result<()> {
    let profile_location = fetch_profile_location();
    let mut json_data = fs::read_to_string(profile_location.clone()).unwrap();
    let mut ps: Vec<Profile> = serde_json::from_str(&*json_data).expect("JSON was mallformed");

    ps.push(Profile {name: n, pattern: p, column: c, nr : num});

    json_data = serde_json::to_string_pretty(&ps)?;

    let mut file = File::create(profile_location).unwrap();
    file.write(json_data.as_bytes()).expect("can't save profiles");

    Ok(())
}

pub fn delete_profile(name: String) -> Vec<Profile> {
    let profile_location = fetch_profile_location();
    let mut json_data = fs::read_to_string(profile_location.clone()).unwrap();
    let mut ps: Vec<Profile> = serde_json::from_str(&*json_data).expect("JSON was mallformed");

    for i in 0..ps.len() {
        if ps[i].name == name {
            ps.remove(i);
            break;
        }
    }

    json_data = serde_json::to_string_pretty(&ps).unwrap();
    let mut file = File::create(profile_location).unwrap();
    file.write(json_data.as_bytes()).expect("can't save profiles");

    get_profiles().expect("can't get profiles")
}