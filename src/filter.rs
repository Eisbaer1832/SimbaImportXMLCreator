use crate::ui::MatchedPattern;
use regex::Regex;
use slint::{SharedString};

pub fn easy_regex(pattern: &str) -> String {
    let mut pat = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '0' {
            let mut zeros = 1;
            while let Some(&'0') = chars.peek() {
                chars.next();
                zeros += 1;
            }
            pat.push_str(&format!("[0-9]{{{}}}", zeros));
        } else {
            pat.push(c);
        }
    }
    pat
}


pub fn filter(data_to_filter: Vec<String>, pattern: &str) -> Vec<MatchedPattern> {
    let filtered: Vec<MatchedPattern> = match Regex::new(&*easy_regex(pattern)) {
        Ok(re) => data_to_filter
            .iter()
            .filter(|name| re.is_match(name))
            .map(|name| {
                let m = re.find(name.as_str()).unwrap();
                MatchedPattern {
                    before: SharedString::from(&name[..m.start()]),
                    matched: SharedString::from(&name[m.start()..m.end()]),
                    after:  SharedString::from(&name[m.end()..]),
                    partner: false
                }
            })
            .collect(),
        Err(_) => data_to_filter  // invalid regex → show all
            .iter()
            .map(|m| MatchedPattern {
                before: SharedString::from(m),
                matched: SharedString::from(""),
                after:  SharedString::from(""),
                partner: false
            })
            .collect(),
    };

    filtered
}