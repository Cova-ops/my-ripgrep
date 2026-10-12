#[derive(PartialEq, Eq, Debug)]
pub struct ArgsInput {
    pub ignore_case: bool,
    pub exclude_dirs: Vec<String>,
    pub extensions: Vec<String>,
    pub amount_threads: usize,
}

impl Default for ArgsInput {
    fn default() -> Self {
        Self {
            ignore_case: false,
            exclude_dirs: vec![],
            extensions: vec![],
            amount_threads: 1,
        }
    }
}

pub fn validate_extra_args(args: &[String]) -> ArgsInput {
    // (--ignore-case, --exclude, --extension)
    let mut out = ArgsInput::default();

    if args.is_empty() {
        return ArgsInput::default();
    }

    let mut ignore_next = false;
    for i in 0..args.len() {
        if ignore_next {
            ignore_next = false;
            continue;
        }

        let current = args.get(i).unwrap();
        match current.as_str() {
            "--ignore-case" => out.ignore_case = true,
            "--exclude" => {
                let exclude_dir = match args.get(i + 1) {
                    Some(v) if !v.is_empty() => v,
                    _ => panic!("After --exclude is requeried a valid path"),
                };

                out.exclude_dirs.push(exclude_dir.clone());
                ignore_next = true;
            }
            "--extension" => {
                let extension = match args.get(i + 1) {
                    Some(v) if !v.is_empty() => v,
                    _ => panic!("After --extension is requeried a valid extension"),
                };

                out.extensions.push(extension.clone());
                ignore_next = true;
            }
            "--threads" => {
                let str_thread = match args.get(i + 1) {
                    Some(v) if !v.is_empty() => v,
                    _ => panic!("After --extension is requeried a valid extension"),
                };

                let Ok(threads) = str_thread.parse::<usize>() else {
                    panic!("After --threads is requeried a valid usize value")
                };

                assert!(
                    threads == 0,
                    "After --threads is requeried a valid usize value, more than 0"
                );

                out.amount_threads = threads;
                ignore_next = true;
            }
            _ => panic!(
                "{current} is not a valid parameter. Just --ignore-case, --exclude, --extension"
            ),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn strings(values: &[&str]) -> Vec<String> {
        values
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    #[test]
    fn defaults() {
        assert_eq!(validate_extra_args(&[]), ArgsInput::default());
    }

    #[test]
    fn parses_all_options_and_repeated_values() {
        assert_eq!(
            validate_extra_args(&strings(&[
                "--ignore-case",
                "--exclude",
                "target",
                "--exclude",
                "vendor",
                "--extension",
                "rs",
                "--extension",
                "txt",
                "--threads",
                "8"
            ])),
            ArgsInput {
                ignore_case: true,
                exclude_dirs: vec!["target".into(), "vendor".into()],
                extensions: vec!["rs".into(), "txt".into()],
                amount_threads: 8
            }
        );
    }

    #[test]
    #[should_panic(expected = "not a valid parameter")]
    fn rejects_unknown_option() {
        validate_extra_args(&strings(&["--unknown"]));
    }

    #[test]
    #[should_panic(expected = "valid path")]
    fn rejects_missing_exclusion() {
        validate_extra_args(&strings(&["--exclude"]));
    }

    #[test]
    #[should_panic(expected = "valid extension")]
    fn rejects_empty_extension() {
        validate_extra_args(&strings(&["--extension", ""]));
    }
}
