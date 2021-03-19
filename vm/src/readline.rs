use std::io;
use std::path::Path;

type OtherError = Box<dyn std::error::Error>;
type OtherResult<T> = Result<T, OtherError>;

pub enum ReadlineResult {
    Line(String),
    EOF,
    Interrupt,
    IO(std::io::Error),
    EncodingError,
    Other(OtherError),
}

#[allow(unused)]
mod basic_readline {
    use super::*;

    pub trait Helper {}
    impl<T> Helper for T {}

    pub struct Readline<H: Helper> {
        helper: H,
    }

    impl<H: Helper> Readline<H> {
        pub fn new(helper: H) -> Self {
            Readline { helper }
        }

        pub fn load_history(&mut self, _path: &Path) -> OtherResult<()> {
            Ok(())
        }

        pub fn save_history(&mut self, _path: &Path) -> OtherResult<()> {
            Ok(())
        }

        pub fn add_history_entry(&mut self, _entry: &str) -> OtherResult<()> {
            Ok(())
        }

        pub fn readline(&mut self, prompt: &str) -> ReadlineResult {
            use std::io::prelude::*;
            print!("{}", prompt);
            if let Err(e) = io::stdout().flush() {
                return ReadlineResult::IO(e);
            }

            match io::stdin().lock().lines().next() {
                Some(Ok(line)) => ReadlineResult::Line(line),
                None => ReadlineResult::EOF,
                Some(Err(e)) => match e.kind() {
                    io::ErrorKind::Interrupted => ReadlineResult::Interrupt,
                    io::ErrorKind::InvalidData => ReadlineResult::EncodingError,
                    _ => ReadlineResult::IO(e),
                },
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod rustyline_readline {
    use super::*;

    pub trait Helper: rustyline::Helper {}
    impl<T: rustyline::Helper> Helper for T {}

    /// Readline: the REPL
    pub struct Readline<H: Helper> {
        repl: rustyline::Editor<H>,
    }

    impl<H: Helper> Readline<H> {
        pub fn new(helper: H) -> Self {
            use rustyline::*;
            let mut repl = Editor::with_config(
                Config::builder()
                    .completion_type(CompletionType::List)
                    .tab_stop(8)
                    .bracketed_paste(false) // multi-line paste
                    .build(),
            );
            repl.set_helper(Some(helper));
            Readline { repl }
        }

        pub fn load_history(&mut self, path: &Path) -> OtherResult<()> {
            self.repl.load_history(path)?;
            Ok(())
        }

        pub fn save_history(&mut self, path: &Path) -> OtherResult<()> {
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            self.repl.save_history(path)?;
            Ok(())
        }

        pub fn add_history_entry(&mut self, entry: &str) -> OtherResult<()> {
            self.repl.add_history_entry(entry);
            Ok(())
        }

        pub fn readline(&mut self, prompt: &str) -> ReadlineResult {
            use rustyline::error::ReadlineError;
            match self.repl.readline(prompt) {
                Ok(line) => ReadlineResult::Line(line),
                Err(ReadlineError::Interrupted) => ReadlineResult::Interrupt,
                Err(ReadlineError::Eof) => ReadlineResult::EOF,
                Err(ReadlineError::Io(e)) => ReadlineResult::IO(e),
                #[cfg(unix)]
                Err(ReadlineError::Utf8Error) => ReadlineResult::EncodingError,
                #[cfg(windows)]
                Err(ReadlineError::Decode(_)) => ReadlineResult::EncodingError,
                Err(e) => ReadlineResult::Other(e.into()),
            }
        }
    }

    #[cfg(test)]
    mod test_rusty_readline {
        use super::{Readline};
        use rustyline::{completion::Completer, highlight::Highlighter,
                        validate::Validator, Context, hint::Hinter};
        use rustyline::error::ReadlineError;
        use crate::readline::ReadlineResult::Line;
        struct HelperShim {}
        impl rustyline::Helper for HelperShim {}
        impl Highlighter for HelperShim {}
        impl Validator for HelperShim {}
        impl Completer for HelperShim {
            type Candidate = String;

            fn complete(
                &self,
                _line: &str,
                _pos: usize,
                _ctx: &Context,
            ) -> rustyline::Result<(usize, Vec<String>)> {
                Err(ReadlineError::Interrupted)
            }
        }
        impl Hinter for HelperShim {
            type Hint = String;
        }
        #[test] #[ignore]
        fn test_multi_read_line() {
            let mut repl = Readline::new(HelperShim { });
            if let Line(line) = repl.readline("print('hello')\nprint('hello2')\n") {
                assert_eq!(line, "print('hello')");
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
use basic_readline as readline_inner;
#[cfg(not(target_arch = "wasm32"))]
use rustyline_readline as readline_inner;

pub use readline_inner::Helper;

pub struct Readline<H: Helper>(readline_inner::Readline<H>);

impl<H: Helper> Readline<H> {
    pub fn new(helper: H) -> Self {
        Readline(readline_inner::Readline::new(helper))
    }
    pub fn load_history(&mut self, path: &Path) -> OtherResult<()> {
        self.0.load_history(path)
    }
    pub fn save_history(&mut self, path: &Path) -> OtherResult<()> {
        self.0.save_history(path)
    }
    pub fn add_history_entry(&mut self, entry: &str) -> OtherResult<()> {
        self.0.add_history_entry(entry)
    }
    pub fn readline(&mut self, prompt: &str) -> ReadlineResult {
        self.0.readline(prompt)
    }
}

