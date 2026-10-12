pub fn validate_extra_args(args: &[String]) -> (bool, Vec<String>, Vec<String>) {
    // (--ignore-case, --exclude, --extension)
    let mut out = (false, vec![], vec![]);

    if args.is_empty() {
        return out;
    }

    let mut ignore_next = false;
    for i in 0..args.len() {
        if ignore_next {
            ignore_next = false;
            continue;
        }

        let current = args.get(i).unwrap();
        if current == "--ignore-case" {
            out.0 = true;
        } else if current == "--exclude" {
            let exclude_path = match args.get(i + 1) {
                Some(v) if !v.is_empty() => v,
                _ => panic!("After --exclude is requeried a valid path"),
            };

            out.1.push(exclude_path.clone());
            ignore_next = true;
        } else if current == "--extension" {
            let extension = match args.get(i + 1) {
                Some(v) if !v.is_empty() => v,
                _ => panic!("After --extension is requeried a valid extension"),
            };

            out.2.push(extension.clone());
            ignore_next = true;
        } else {
            panic!(
                "{current} is not a valid parameter. Just --ignore-case, --exclude, --extension"
            );
        }
    }

    out
}
