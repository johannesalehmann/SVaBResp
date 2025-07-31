mod input;

#[cfg(test)]
mod tests;

fn main() {
    let exit_code = checker();
    std::process::exit(exit_code);
}

fn checker() -> i32 {
    let source = include_str!("tests/files/bluetooth.v1.prism");
    let parse = input::prism::parse_prism(Some("bluetooth.v1.prism"), source);
    let _parse = match parse {
        None => {
            return 1;
        }
        Some(parse) => parse,
    };

    0
}
