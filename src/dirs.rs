use std::fmt::Write;
use std::{
    fs::{self},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
    vec,
};

use crate::thread;

#[derive(Debug)]
struct File {
    path: PathBuf,
    extension: Option<Arc<Vec<String>>>,
    ignore_case: bool,
}

impl File {
    const fn new(path: PathBuf, extension: Option<Arc<Vec<String>>>, ignore_case: bool) -> Self {
        Self {
            path,
            extension,
            ignore_case,
        }
    }

    /// Check if this file should be included
    fn check_valid_extension(&self) -> bool {
        if self.extension.is_none() {
            return true;
        }

        let extension = self.extension.as_ref().unwrap();
        let Some(file_ext) = self.path.extension() else {
            return false; // The file does not have extension
        };

        match file_ext.to_str() {
            Some(v) if extension.iter().any(|ext| ext == v) => true,
            None => true, // The extension is not in UTF-8, should be included
            _ => false,
        }
    }

    fn search_match(&self, to_search: &str) -> Option<(Vec<String>, u32)> {
        let file = fs::File::open(&self.path).ok()?;
        let mut reader = BufReader::new(file);

        let normalized_search = self.ignore_case.then(|| to_search.to_ascii_lowercase());

        let mut out: Vec<String> = vec![];
        let mut cont = 0;
        let mut line = String::new();

        loop {
            line.clear();

            let bytes_read = reader.read_line(&mut line).ok()?;

            if bytes_read == 0 {
                break;
            }

            cont += 1;

            let matched = normalized_search.as_ref().map_or_else(
                || line.contains(to_search),
                |v| line.to_ascii_lowercase().contains(v),
            );

            if matched {
                out.push(format!("{}:{}: {}", self.path.display(), cont, line.trim()));
            }
        }

        Some((out, cont))
    }
}

#[derive(Debug)]
struct Directory<'a> {
    path: &'a Path,
    ignore_case: bool,
    exclude_dirs: Option<Arc<Vec<String>>>,
    extension: Option<Arc<Vec<String>>>,
}

impl<'a> Directory<'a> {
    fn new_from_self(&self, path: &'a Path) -> Self {
        Self {
            path,
            ignore_case: self.ignore_case,
            exclude_dirs: self.exclude_dirs.clone(),
            extension: self.extension.clone(),
        }
    }

    /// Check if this dir should be excluded
    fn check_excluded_dir(&self) -> bool {
        if self.exclude_dirs.is_none() {
            return false;
        }

        let exclude_dirs = self.exclude_dirs.as_ref().unwrap();

        let Some(dir_name) = self.path.file_name() else {
            return false; // Included this, maybe is root path
        };

        match dir_name.to_str() {
            Some(name) if exclude_dirs.iter().any(|exclude| exclude == name) => true,
            _ => false, // If dir's name is not UTF-8, should be included
        }
    }

    fn get_all_files(&self) -> Vec<File> {
        if self.check_excluded_dir() {
            return vec![];
        }

        let dir = fs::read_dir(self.path).unwrap();

        let mut out: Vec<File> = vec![];
        for entry in dir {
            let Ok(entry) = entry else { continue };

            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            let entry_path = entry.path();

            if file_type.is_dir() {
                let new_dir = self.new_from_self(entry_path.as_path());
                let mut new_vec = new_dir.get_all_files();
                out.append(&mut new_vec);
            } else if file_type.is_file() {
                let ext_ref = self.extension.as_ref().map(std::clone::Clone::clone);
                let new_file = File::new(entry_path, ext_ref, self.ignore_case);

                let valid_ext = new_file.check_valid_extension();
                if !valid_ext {
                    continue;
                }

                out.push(new_file);
            }
        }

        out
    }
}

#[derive(Debug)]
pub struct WorkDir {
    to_search: String,
    root_path: PathBuf,
    ignore_case: bool,
    extension: Option<Arc<Vec<String>>>,
    exclude_dirs: Option<Arc<Vec<String>>>,
    max_threads: usize,
}

impl WorkDir {
    pub fn new(
        to_search: &str,
        raw_path: &str,
        ignore_case: bool,
        extension: Vec<String>,
        exclude_dirs: Vec<String>,
        max_threads: usize,
    ) -> Result<Self, String> {
        let root_path = PathBuf::from(raw_path);

        if !root_path.exists() {
            return Err("Route doesn't exists. Review that.".into());
        }

        let extension = if extension.is_empty() {
            None
        } else {
            Some(Arc::new(extension))
        };

        let exclude_dirs = if exclude_dirs.is_empty() {
            None
        } else {
            Some(Arc::new(exclude_dirs))
        };

        Ok(Self {
            to_search: to_search.to_owned(),
            root_path,
            ignore_case,
            extension,
            exclude_dirs,
            max_threads,
        })
    }

    pub fn find_matches(&self) -> String {
        let start = Instant::now();

        if self.root_path.is_file() {
            let ext_ref = self.extension.as_ref().map(std::clone::Clone::clone);

            let (matches, amount_lines) =
                File::new(self.root_path.clone(), ext_ref, self.ignore_case)
                    .search_match(&self.to_search)
                    .unwrap_or_default();
            let elapsed_time: Duration = start.elapsed();

            let result = ResultSearch {
                amount_files: 1,
                amount_lines,
                elapsed_time,
                matches,
            };

            return result.show_results();
        }

        let dir = Directory {
            path: self.root_path.as_path(),
            exclude_dirs: self.exclude_dirs.clone(),
            ignore_case: self.ignore_case,
            extension: self.extension.clone(),
        };

        let files = dir.get_all_files();
        let pool = thread::ThreadPool::new(self.max_threads);

        let to_search_arc = Arc::new(self.to_search.clone());
        for file in files {
            let lock = Arc::clone(&to_search_arc);
            pool.execute(move || file.search_match(lock.as_ref()));
        }

        let vec_threads = pool.join();

        let mut vec_matches = vec![];
        let mut amount_files = 0;
        let mut amount_lines = 0;

        for mat in vec_threads {
            amount_files += 1;

            if let Some((mut matches, lines)) = mat {
                vec_matches.append(&mut matches);
                amount_lines += lines;
            }
        }

        let elapsed_time: Duration = start.elapsed();

        let result = ResultSearch {
            amount_lines,
            amount_files,
            matches: vec_matches,
            elapsed_time,
        };

        result.show_results()
    }
}

struct ResultSearch {
    amount_files: u32,
    amount_lines: u32,
    elapsed_time: Duration,
    matches: Vec<String>,
}

impl ResultSearch {
    pub fn show_results(mut self) -> String {
        let amount_matches = self.matches.len();
        let mut out = String::new();

        if cfg!(test) {
            self.matches.sort();
        }

        for line in self.matches {
            let _ = writeln!(out, "{}", line.trim_end());
        }

        let _ = write!(
            out,
            "\nAmount files: {} -- Amount lines: {} \
         -- Amount matches: {} -- Time: {} microseconds.",
            self.amount_files,
            self.amount_lines,
            amount_matches,
            self.elapsed_time.as_micros()
        );

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extension_filter() {
        let allowed = Some(Arc::new(vec!["rs".to_string()]));
        assert!(File::new("main.rs".into(), allowed.clone(), false).check_valid_extension());
        assert!(!File::new("main.txt".into(), allowed.clone(), false).check_valid_extension());
        assert!(!File::new("README".into(), allowed, false).check_valid_extension());
        assert!(File::new("README".into(), None, false).check_valid_extension());
    }

    #[test]
    fn matches_report_line_numbers_and_case() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("app.log");
        fs::write(&path, "INFO start\nERROR first\nerror second\n").unwrap();
        let (matches, lines) = File::new(path.clone(), None, false)
            .search_match("ERROR")
            .unwrap();
        assert_eq!(lines, 3);
        assert_eq!(matches, vec![format!("{}:2: ERROR first", path.display())]);
        let (matches, _) = File::new(path, None, true).search_match("error").unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn unreadable_or_missing_file_returns_none() {
        assert!(
            File::new("does-not-exist.log".into(), None, false)
                .search_match("x")
                .is_none()
        );
    }

    #[test]
    fn recursively_filters_directories_and_extensions() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("archive")).unwrap();
        fs::create_dir_all(tmp.path().join("target")).unwrap();
        fs::write(tmp.path().join("app.log"), "ERROR").unwrap();
        fs::write(tmp.path().join("notes.txt"), "ERROR").unwrap();
        fs::write(tmp.path().join("archive/old.log"), "ERROR").unwrap();
        fs::write(tmp.path().join("target/ignored.log"), "ERROR").unwrap();
        let dir = Directory {
            path: tmp.path(),
            ignore_case: false,
            exclude_dirs: Some(Arc::new(vec!["target".into()])),
            extension: Some(Arc::new(vec!["log".into()])),
        };
        let files = dir.get_all_files();
        let names: std::collections::HashSet<_> = files
            .iter()
            .map(|f| f.path.strip_prefix(tmp.path()).unwrap().to_path_buf())
            .collect();
        assert_eq!(
            names,
            [PathBuf::from("app.log"), PathBuf::from("archive/old.log")]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn workdir_rejects_nonexistent_path() {
        let tmp = tempdir().unwrap();
        assert!(
            WorkDir::new(
                "x",
                &tmp.path().join("missing").to_string_lossy(),
                false,
                vec![],
                vec![],
                1
            )
            .is_err()
        );
    }

    #[test]
    fn workdir_searches_single_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("app.log");
        fs::write(&path, "ERROR here\nINFO there\n").unwrap();
        let result = WorkDir::new("ERROR", &path.to_string_lossy(), false, vec![], vec![], 1)
            .unwrap()
            .find_matches();
        assert!(result.contains("app.log:1: ERROR here"));
        assert!(result.contains("Amount files: 1 -- Amount lines: 2 -- Amount matches: 1"));
    }

    #[test]
    fn result_format_includes_counts() {
        let result = ResultSearch {
            amount_files: 2,
            amount_lines: 3,
            elapsed_time: Duration::from_micros(5),
            matches: vec!["a:1: hit".into()],
        }
        .show_results();
        assert!(result.contains("a:1: hit"));
        assert!(result.contains(
            "Amount files: 2 -- Amount lines: 3 -- Amount matches: 1 -- Time: 5 microseconds."
        ));
    }
}
