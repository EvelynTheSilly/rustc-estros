use crate::collections::BTreeMap;
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::num::NonZeroI32;
use crate::path::Path;
use crate::sys::cap::Capability;
use crate::sys::cap_table;
use crate::sys::syscalls;
use crate::sys::unsupported_err;

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub const ENV: &str = "env";

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub const STDIN: &str = "stdin";

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub const STDOUT: &str = "stdout";

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub const STDERR: &str = "stderr";

/// TODO: doc
pub fn spawn(elf: &[u8], caps: &[Capability]) -> io::Result<u64> {
    let ids: Vec<u64> = caps.iter().map(Capability::id).collect();
    let pid = unsafe {
        syscalls::spawn_process(elf.as_ptr(), elf.len(), ids.as_ptr() as *const u8, ids.len())
    };
    if (pid as i64) < 0 {
        return Err(io::const_error!(io::ErrorKind::Other, "failed to spawn process"));
    }
    Ok(pid)
}

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
        spawn(elf, &caps)
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

#[derive(Default)]
struct Env {
    clear: bool,
    vars: Vec<(OsString, Option<OsString>)>,
}

impl Env {
    fn capture(&self) -> Vec<(OsString, OsString)> {
        let mut env = BTreeMap::new();
        if !self.clear {
            for (key, value) in crate::env::vars_os() {
                env.insert(key, value);
            }
        }
        for (key, value) in &self.vars {
            match value {
                Some(value) => {
                    env.insert(key.clone(), value.clone());
                }
                None => {
                    env.remove(key);
                }
            }
        }
        env.into_iter().collect()
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
#[derive(Clone)]
pub enum Stdio {
    /// TODO: doc
    Inherit,
    /// TODO: doc
    Cap(Capability),
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pid(u64);

impl Pid {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub struct Process {
    pid: u64,
}

impl Process {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn pid(&self) -> Pid {
        Pid(self.pid)
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn id(&self) -> u32 {
        self.pid as u32
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn kill(&mut self) -> io::Result<()> {
        Err(unsupported_err())
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        Err(unsupported_err())
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Err(unsupported_err())
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct ExitStatus(i32);

impl ExitStatus {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        if self.0 == 0 { Ok(()) } else { Err(ExitStatusError(*self)) }
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }
}

#[stable(feature = "estros", since = "1.0.0")]
impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code: {}", self.0)
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(ExitStatus);

#[stable(feature = "estros", since = "1.0.0")]
impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        self.0
    }
}

impl ExitStatusError {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn code(self) -> Option<NonZeroI32> {
        NonZeroI32::new(self.0.0)
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(i32);

impl ExitCode {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub const SUCCESS: ExitCode = ExitCode(0);
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub const FAILURE: ExitCode = ExitCode(1);

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

#[stable(feature = "estros", since = "1.0.0")]
impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        Self(code as i32)
    }
}

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub struct Spawner<'a> {
    elf: &'a [u8],
    #[allow(dead_code)]
    args: Vec<OsString>,
    env: Env,
    #[allow(dead_code)]
    cwd: Option<OsString>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    caps: Vec<(String, Capability)>,
}

impl<'a> Spawner<'a> {
    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn new(elf: &'a [u8]) -> Spawner<'a> {
        Spawner {
            elf,
            args: Vec::new(),
            env: Env::default(),
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
            caps: Vec::new(),
        }
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Spawner<'a> {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn args<I, S>(mut self, args: I) -> Spawner<'a>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args.extend(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Spawner<'a> {
        self.env.vars.push((key.as_ref().to_owned(), Some(value.as_ref().to_owned())));
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn envs<I, K, V>(mut self, vars: I) -> Spawner<'a>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env.vars.extend(
            vars.into_iter()
                .map(|(key, value)| (key.as_ref().to_owned(), Some(value.as_ref().to_owned()))),
        );
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Spawner<'a> {
        self.env.vars.push((key.as_ref().to_owned(), None));
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn env_clear(mut self) -> Spawner<'a> {
        self.env.clear = true;
        self.env.vars.clear();
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Spawner<'a> {
        self.cwd = Some(dir.as_ref().as_os_str().to_owned());
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn stdin(mut self, cfg: Stdio) -> Spawner<'a> {
        self.stdin = Some(cfg);
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn stdout(mut self, cfg: Stdio) -> Spawner<'a> {
        self.stdout = Some(cfg);
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn stderr(mut self, cfg: Stdio) -> Spawner<'a> {
        self.stderr = Some(cfg);
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn cap(mut self, name: &str, cap: Capability) -> Spawner<'a> {
        self.caps.push((name.to_owned(), cap));
        self
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn spawn(self) -> io::Result<Process> {
        let mut caps = CapSet::new();
        caps.push(ENV, env_cap(&self.env.capture())?)?;

        for (name, stdio) in [(STDIN, self.stdin), (STDOUT, self.stdout), (STDERR, self.stderr)] {
            match stdio {
                Some(Stdio::Cap(cap)) => caps.push(name, cap)?,
                Some(Stdio::Inherit) | None => {
                    if let Some(cap) = cap_table::get_cap(name) {
                        caps.push(name, cap)?;
                    }
                }
            }
        }

        for (name, cap) in self.caps {
            caps.push(&name, cap)?;
        }

        let pid = caps.spawn(self.elf)?;
        Ok(Process { pid })
    }

    /// TODO: doc
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn status(self) -> io::Result<ExitStatus> {
        let mut process = self.spawn()?;
        process.wait()
    }
}
