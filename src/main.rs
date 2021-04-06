use regex::Regex;

fn main() {
    let validator = Regex::new(r"(?ms)\A(z_)?[[:alpha:]]+\n.*?(Result:\n");
}
