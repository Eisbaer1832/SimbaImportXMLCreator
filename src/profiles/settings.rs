use crate::APP_INFO;
use preferences::{Preferences, PreferencesMap};
use std::path::PathBuf;

pub fn fetch_profile_location() -> String {
    let load_result = PreferencesMap::<String>::load(&APP_INFO, "profile_location");
    if load_result.is_ok(){
        String::from(load_result.unwrap().get("profile_location").unwrap_or(&String::new()))
    }else {
        String::new()
    }
}

pub fn set_profile_location(path: PathBuf) {
    let mut prefs: PreferencesMap<String> = PreferencesMap::new();
    prefs.insert("profile_location".into(), path.to_str().unwrap().into());

    let save_result = prefs.save(&APP_INFO, "profile_location");
    assert!(save_result.is_ok());
}