use itertools::Itertools;

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
