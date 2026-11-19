use core::slice::memchr;

use crate::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fmt,
    io,
    sync::{Mutex, OnceLock},
    sys::cap_table,
    vec,
};

pub struct Env {
    iter: vec::IntoIter<(OsString, OsString)>,
}

impl Env {
    pub(super) fn new(env: Vec<(OsString, OsString)>) -> Self {
        Env { iter: env.into_iter() }
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.as_slice()).finish()
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

fn env_data() -> Option<Vec<u8>> {
    let cap = cap_table::get_cap_table().get("env")?;
    let msg = cap.wait_message()?;
    msg.drain()
}

type EnvStore = Mutex<HashMap<OsString, OsString>>;

static ENV: OnceLock<EnvStore> = OnceLock::new();

fn get_env_store() -> &'static EnvStore {
    ENV.get_or_init(|| {
        let mut env_store = HashMap::new();

        // The raw environment block is only needed while building the map.
        if let Some(data) = env_data() {
            for entry in data.split(|b| *b == b'\0') {
                if let Some((key, value)) = parse(entry) {
                    env_store.insert(key, value);
                }
            }
        }

        Mutex::new(env_store)
    })
}

fn parse(input: &[u8]) -> Option<(OsString, OsString)> {
    // Strategy (copied from glibc): Variable name and value are separated
    // by an ASCII equals sign '='. Since a variable name must not be
    // empty, allow variable names starting with an equals sign. Skip all
    // malformed lines.
    if input.is_empty() {
        return None;
    }
    let pos = memchr::memchr(b'=', &input[1..]).map(|p| p + 1);
    pos.map(|p| {
        (
            os_string_from_vec(input[..p].to_vec()),
            os_string_from_vec(input[p + 1..].to_vec()),
        )
    })
}

fn os_string_from_vec(bytes: Vec<u8>) -> OsString {
    // On estros the OsStr encoding is raw bytes, so this is lossless.
    unsafe { OsString::from_encoded_bytes_unchecked(bytes) }
}

pub fn env() -> Env {
    let env = get_env_store().lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    Env::new(env)
}

pub fn getenv(k: &OsStr) -> Option<OsString> {
    get_env_store().lock().unwrap().get(k).cloned()
}

pub unsafe fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let (k, v) = (k.to_owned(), v.to_owned());
    get_env_store().lock().unwrap().insert(k, v);
    Ok(())
}

pub unsafe fn unsetenv(k: &OsStr) -> io::Result<()> {
    get_env_store().lock().unwrap().remove(k);
    Ok(())
}
