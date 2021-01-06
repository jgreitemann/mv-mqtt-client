use itertools::Itertools;
use regex::Regex;

pub fn ellipt(phrase: &str, min_length: usize) -> String {
    let mut shortened = phrase
        .split_whitespace()
        .scan((0, 0), |(total_up_to, this_length), w| {
            *total_up_to += *this_length + 1;
            *this_length = w.len();
            Some(((*total_up_to, *this_length), w))
        })
        .take_while(|((total_up_to, _), _)| *total_up_to < min_length)
        .map(|(_, w)| w)
        .join(" ");
    if shortened != phrase {
        shortened.push_str("...");
    }
    shortened
}

pub fn regex_from_mqtt_wildcard(wildcard: &str) -> Regex {
    let expanded = wildcard
        .split('/')
        .map(|w| {
            w.chars()
                .map(|c| match c {
                    x @ '['
                    | x @ ']'
                    | x @ '('
                    | x @ ')'
                    | x @ '.'
                    | x @ '*'
                    | x @ '?'
                    | x @ '\\'
                    | x @ '^'
                    | x @ '$'
                    | x @ '|' => format!("\\{}", x),
                    x => format!("{}", x),
                })
                .join("")
        })
        .map(|subtopic| match subtopic.as_str() {
            "#" => "[^+#]+".to_string(),
            "+" => "[^+#/]+".to_string(),
            _ => subtopic,
        })
        .join("/");
    println!("^{}$", expanded);
    Regex::new(&*format!("^{}$", expanded)).unwrap()
}
