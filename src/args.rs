//! The options of one command: `--name value`, `--name=value` and `--flag`.

/// The options that a command has not read yet.
#[derive(Debug, Default)]
pub(crate) struct Args {
    options: Vec<(String, Option<String>)>,
}

impl Args {
    pub(crate) fn parse(argv: &[String]) -> Result<Args, String> {
        let mut out = Args::default();
        let mut i = 0;
        while i < argv.len() {
            let word = &argv[i];
            let Some(name) = word.strip_prefix("--") else {
                return Err(format!("usage: {word:?} is not an option"));
            };
            if let Some((name, value)) = name.split_once('=') {
                out.options.push((name.to_owned(), Some(value.to_owned())));
            } else if argv.get(i + 1).is_some_and(|next| !next.starts_with("--")) {
                out.options.push((name.to_owned(), Some(argv[i + 1].clone())));
                i += 1;
            } else {
                out.options.push((name.to_owned(), None));
            }
            i += 1;
        }
        Ok(out)
    }

    /// Takes the value of `--name`.
    pub(crate) fn value(&mut self, name: &str) -> Option<String> {
        let i = self.options.iter().position(|(n, v)| n == name && v.is_some())?;
        self.options.remove(i).1
    }

    /// Takes `--name` with no value. A flag that has a value is an error in `finish`.
    pub(crate) fn flag(&mut self, name: &str) -> bool {
        match self.options.iter().position(|(n, v)| n == name && v.is_none()) {
            Some(i) => {
                self.options.remove(i);
                true
            }
            None => false,
        }
    }

    /// Fails if an option was not read, so a typo is not ignored.
    pub(crate) fn finish(self) -> Result<(), String> {
        match self.options.first() {
            None => Ok(()),
            Some((name, _)) => Err(format!("usage: unknown option --{name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn values_and_flags() {
        let mut a = Args::parse(&words("--file p.toml --shell --n=3")).unwrap();
        assert_eq!(a.value("file").as_deref(), Some("p.toml"));
        assert!(a.flag("shell"));
        assert_eq!(a.value("n").as_deref(), Some("3"));
        a.finish().unwrap();
    }

    #[test]
    fn an_unknown_option_fails() {
        let mut a = Args::parse(&words("--file x --sheel")).unwrap();
        assert!(a.value("file").is_some());
        assert!(!a.flag("shell"));
        assert_eq!(a.finish().unwrap_err(), "usage: unknown option --sheel");
        assert!(Args::parse(&words("stray")).is_err());
    }
}
