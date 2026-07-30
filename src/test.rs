
#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::DirEntry;
    use std::path::PathBuf;
    use crate::generate::generate;
    use crate::profiles::{get_profiles, Profile};

    fn run_test(f: DirEntry, profiles: Vec<Profile>, buchungsstapel_name: &str) {
        let binding = f.file_name();
        let test_name = binding.to_str().unwrap();
        // get the appropriate profile
        for p in &profiles{
            if p.name == test_name {
                // run actual test
                let pdf_path = f.path().join("belege");
                let csv_file = f.path().join(buchungsstapel_name);
                let (lines, _, matched) =  generate(pdf_path, csv_file, p.pattern.clone(), p.column.clone());
                assert!((matched as usize / lines) >= 0.5 as usize);
                return;
            }
        }
        panic!("Profile {test_name} not found!")
    }

    #[test]
    fn test_scs() {
        let profiles = get_profiles().expect("Could not get profiles");
        let test_dir = PathBuf::from(std::env::var("TEST_DIR").unwrap()).join("scs");

        fs::read_dir(test_dir.clone()).unwrap().for_each(|file| {
            let f = file.unwrap();
            // check for misconfigured tests
            if !f.path().join("belege").exists() {panic!("belege directory is missing")}
            if !f.path().join("buchungsstapel.scs").exists() { panic!("buchungsstapel is missing")}

            run_test(f, profiles.clone(), "buchungsstapel.scs");
        });
    }

    #[test]
    fn test_csv() {
        let profiles = get_profiles().expect("Could not get profiles");
        let test_dir = PathBuf::from(std::env::var("TEST_DIR").unwrap()).join("csv");

        fs::read_dir(test_dir.clone()).unwrap().for_each(|file| {
            let f = file.unwrap();
            // check for misconfigured tests
            if !f.path().join("belege").exists() {panic!("belege directory is missing")}
            if !f.path().join("buchungsstapel.csv").exists() { panic!("buchungsstapel is missing")}

            run_test(f, profiles.clone(), "buchungsstapel.csv");
        });
    }
}
