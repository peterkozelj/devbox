use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Result;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};

//-- Cmd -------------------------------------------------------------------------------------------

/// Clone-able variant of [`std::process::Command`] with some build specific helper methods
///
/// [`std::process::Command`]: (https://doc.rust-lang.org/std/process/struct.Command.html)
///
/// Cloning allows configuring commands with some common arguments and then invoking it with
/// additional arguments/env vars at different places inside the script.
///
/// Method of this type are mirroring those of std::process::Command with addition of [`run`] and
/// [`run_result`] commands for easier use inside of build sript.
///
/// [`run`]: #method.run
/// [`run_result`]: #method.run_result
///
#[derive(Clone, Debug)]
pub struct Cmd {
    program: OsString,
    args: Vec<OsString>,
    envs: HashMap<OsString, OsString>,
    work: Option<PathBuf>,
}

impl Cmd {

    /// Constructs a new Cmd for launching the executable at path `program`
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            program: program.as_ref().to_owned(),
            args: vec![],
            envs: HashMap::new(),
            work: None,
        }
    }

    /// Adds an argument to the list of execution arguments
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Adds multiple arguments to the list of execution arguments
    pub fn args<I, S>(mut self, args: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>,
    {
        self.args.extend(args.into_iter().map(|e| e.as_ref().to_owned()));
        self
    }

    /// Sets an environment variable
    pub fn env<K: AsRef<OsStr>, V: AsRef<OsStr>>(mut self, env: K, val: V) -> Self {
        self.envs.insert(env.as_ref().to_owned(), val.as_ref().to_owned());
        self
    }

    /// Run the command and return it's output.
    ///
    /// This is convienece method for calling [`std::process::Command::output()`] method on command
    /// instance retrieved by [`command`] method
    ///
    /// [`command`]: #method.command
    /// [`std::process::Command::output()`]:
    /// https://doc.rust-lang.org/std/process/struct.Command.html#method.output
    pub fn output(&self) -> Output {
        println!("Executing: {:?} {:?} {:?}", self.program, self.args, self.envs);
        self.command().output().expect(format!("Command executon '{:?} {:?} {:?}' failed",
            self.program, self.args, self.envs).as_str()
        )
    }

    /// Run the command and exit the build with informative panic message if execution fails.
    ///
    /// This is convienece method for calling [`std::process::Command::status()`] method on command
    /// instance retrieved by [`command`] method
    ///
    /// [`command`]: #method.command
    /// [`std::process::Command::status()`]:
    /// https://doc.rust-lang.org/std/process/struct.Command.html#method.status
    pub fn run(&self) {
        self.run_result().expect(format!("Command executon '{:?} {:?} {:?}' failed",
            self.program, self.args, self.envs).as_str()
        );
    }

    /// Run the command and return it's status.
    ///
    /// This is convienece method for calling [`std::process::Command::status()`] method on command
    /// instance retrieved by [`command`] method
    ///
    /// [`command`]: #method.command
    /// [`std::process::Command::status()`]:
    /// https://doc.rust-lang.org/std/process/struct.Command.html#method.status
    pub fn run_result(&self) -> Result<ExitStatus> {
        self.command().status()
    }

    /// Build the `std::process::Command` with args and environment variables set up by methods on
    /// this Cmd instance.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command.envs(&self.envs);

        if let Some(work_dir) = &self.work {
            command.current_dir(work_dir);
        }

        command
    }
}