use regex::Regex;

fn main() {
    let validator = Regex::new(r"(?ms)\A(z_)?[[:alpha:]]+ .*?(\nArguments:\n.*?)+(\nResult:\n.*?)+\nExamples:\n").unwrap();
    let example1 =
        std::fs::read_to_string("quizface_help/z_getoperationresult.txt")
            .unwrap();
    let example2 =
        std::fs::read_to_string("quizface_help/getaddressbalance.txt").unwrap();
    let example3 =
        std::fs::read_to_string("quizface_help/settxfee.txt").unwrap();
    dbg!(validator.is_match(&example1));
    dbg!(validator.is_match(&example2));
    dbg!(validator.is_match(&example3));
}
