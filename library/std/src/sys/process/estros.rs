use super::env::{CommandEnv, CommandEnvs, CommandResolvedEnvs};
pub use crate::ffi::OsString as EnvKey;
use crate::ffi::{OsStr, OsString};
use crate::io;
use crate::num::NonZeroI32;
use crate::path::Path;
use crate::process::StdioPipes;
use crate::sys::cap::Capability;
use crate::sys::cap_table;
use crate::sys::fs::File;
use crate::sys::unsupported_err;
use crate::fmt;

/// TODO: doc
pub const ENV: &str = "env";

/// TODO: doc
pub const STDOUT: &str = "stdout";

/// TODO: doc
pub const STDERR: &str = "stderr";

struct CapSet {
    caps: Vec<(String, Capability)>,
}

impl CapSet {
    fn new() -> CapSet {
        CapSet { caps: Vec::new() }
    }

    fn push(&mut self, name: &str, cap: Capability) -> io::Result<()> {
        if name.is_empty() || !name.is_ascii() {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "capability names must be non-empty ASCII"
            ));
        }
        self.caps.push((name.to_owned(), cap));
        Ok(())
    }

    fn spawn(self, elf: &[u8]) -> io::Result<u64> {
        let descriptor = Capability::spawn_message();
        let stream = encode_descriptor(&self.caps);
        if descriptor.send_message(&stream, Vec::new()).is_none() {
            return Err(io::const_error!(
                io::ErrorKind::Other,
                "failed to send the capability descriptor"
            ));
        }

        let mut caps = Vec::with_capacity(self.caps.len() + 1);
        caps.push(descriptor);
        caps.extend(self.caps.into_iter().map(|(_, cap)| cap));
        crate::sys::pal::process::spawn(elf, &caps)
    }
}

fn encode_descriptor(caps: &[(String, Capability)]) -> Vec<u8> {
    let mut stream = Vec::new();
    for (index, (name, _)) in caps.iter().enumerate() {
        let child_id = index as u64 + 1;
        stream.extend_from_slice(&child_id.to_ne_bytes());
        stream.extend_from_slice(&(name.len() as u64).to_ne_bytes());
        stream.extend_from_slice(name.as_bytes());
    }
    stream
}

fn env_cap(env: &[(OsString, OsString)]) -> io::Result<Capability> {
    let mut block = Vec::new();
    for (key, value) in env {
        let key = key.as_encoded_bytes();
        if key.is_empty() || key.contains(&b'=') {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "environment variable names must be non-empty and contain no '='"
            ));
        }
        block.extend_from_slice(key);
        block.push(b'=');
        block.extend_from_slice(value.as_encoded_bytes());
        block.push(0);
    }

    let cap = Capability::spawn_message();
    if cap.send_message(&block, Vec::new()).is_none() {
        return Err(io::const_error!(io::ErrorKind::Other, "failed to send the environment block"));
    }
    Ok(cap)
}

pub enum Stdio {
    Inherit,
    #[allow(dead_code)]
    Null,
    MakePipe,
    #[allow(dead_code)]
    Cap(Capability),
    #[allow(dead_code)]
    Unsupported,
}

pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    env: CommandEnv,
    cwd: Option<OsString>,
    #[allow(dead_code)]
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    elf: Option<Vec<u8>>,
    caps: Vec<(String, Capability)>,
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command {
            program: program.to_owned(),
            args: vec![program.to_owned()],
            env: Default::default(),
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
            elf: None,
            caps: Vec::new(),
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.args.push(arg.to_owned());
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.cwd = Some(dir.to_owned());
    }

    pub fn stdin(&mut self, stdin: Stdio) {
        self.stdin = Some(stdin);
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.stdout = Some(stdout);
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.stderr = Some(stderr);
    }

    pub fn get_program(&self) -> &OsStr {
        &self.program
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let mut iter = self.args.iter();
        iter.next();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_env_clear(&self) -> bool {
        self.env.does_clear()
    }

    pub fn get_resolved_envs(&self) -> CommandResolvedEnvs {
        CommandResolvedEnvs::new(self.env.capture())
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.cwd.as_ref().map(|cwd| Path::new(cwd))
    }

    /// TODO: doc
    #[allow(dead_code)]
    pub(crate) fn elf(&mut self, elf: &[u8]) {
        self.elf = Some(elf.to_vec());
    }

    /// TODO: doc
    #[allow(dead_code)]
    pub(crate) fn cap(&mut self, name: &str, cap: Capability) {
        self.caps.push((name.to_owned(), cap));
    }

    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        let elf = self.elf.as_deref().ok_or_else(unsupported_err)?;

        let mut caps = CapSet::new();
        let env: Vec<(OsString, OsString)> = self.env.capture().into_iter().collect();
        caps.push(ENV, env_cap(&env)?)?;

        if let Some(cap) = stream_cap(&self.stdout, STDOUT)? {
            caps.push(STDOUT, cap)?;
        }
        if let Some(cap) = stream_cap(&self.stderr, STDERR)? {
            caps.push(STDERR, cap)?;
        }
        for (name, cap) in &self.caps {
            caps.push(name, cap.clone())?;
        }

        let pid = caps.spawn(elf)?;
        Ok((Process { pid }, StdioPipes { stdin: None, stdout: None, stderr: None }))
    }
}

fn stream_cap(stdio: &Option<Stdio>, name: &str) -> io::Result<Option<Capability>> {
    match stdio {
        None | Some(Stdio::Inherit) => Ok(cap_table::get_cap(name)),
        Some(Stdio::Cap(cap)) => Ok(Some(cap.clone())),
        Some(Stdio::Null) | Some(Stdio::MakePipe) | Some(Stdio::Unsupported) => {
            Err(unsupported_err())
        }
    }
}

pub fn output(_cmd: &mut Command) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    Err(unsupported_err())
}

impl From<ChildPipe> for Stdio {
    fn from(pipe: ChildPipe) -> Stdio {
        pipe.diverge()
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        Stdio::Inherit
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        Stdio::Inherit
    }
}

impl From<File> for Stdio {
    fn from(_: File) -> Stdio {
        Stdio::Unsupported
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("program", &self.program)
            .field("args", &self.args)
            .field("cwd", &self.cwd)
            .field("env", &self.env)
            .finish_non_exhaustive()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        if self.0 == 0 { Ok(()) } else { Err(ExitStatusError(*self)) }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code: {}", self.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(ExitStatus);

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        self.0
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZeroI32> {
        NonZeroI32::new(self.0.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(i32);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const FAILURE: ExitCode = ExitCode(1);

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        Self(code as i32)
    }
}

pub struct Process {
    pid: u64,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.pid as u32
    }

    pub fn kill(&mut self) -> io::Result<()> {
        Err(unsupported_err())
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        Err(unsupported_err())
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Err(unsupported_err())
    }
}

pub struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, OsString>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next().map(|os| &**os)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}

pub type ChildPipe = crate::sys::pipe::Pipe;

pub fn read_output(
    _out: ChildPipe,
    _stdout: &mut Vec<u8>,
    _err: ChildPipe,
    _stderr: &mut Vec<u8>,
) -> io::Result<()> {
    Err(unsupported_err())
}

pub fn getpid() -> u32 {
    unsafe { crate::sys::syscalls::get_pid() as u32 }
}
