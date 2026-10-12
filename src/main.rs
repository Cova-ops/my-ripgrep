/*
MyRipGrep

Goal:
- Search for a pattern recursively inside a directory.
- Read files line-by-line.
- Process files concurrently.

Functional Requirements:
- Accept: <pattern> <directory>
- Search recursively.
- Print: <file>:<line>:<content>
- Support:
    - --ignore-case
    - --exclude <dir>
    - --extension <ext>
- Continue if a file cannot be read.
- Print statistics (files, lines, matches, elapsed time).
- Sort results by file and line number.
- Return exit code 0 on success, 1 on fatal error.

*/

mod args;
mod dirs;
mod thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().collect();
    args.remove(0); // Remove the first one, because it is the exec path

    let search_string = args.first().ok_or("Missing searching string")?;
    let path = args.get(1).ok_or("Missing path to make the search")?;

    let args_input = args::validate_extra_args(args.get(2..).unwrap_or_default());

    let work_dir = dirs::WorkDir::new(
        search_string,
        path,
        args_input.ignore_case,
        args_input.extensions,
        args_input.exclude_dirs,
        args_input.amount_threads,
    )?;
    let out = work_dir.find_matches();

    println!("{out}");

    Ok(())
}
