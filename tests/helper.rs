use std::{fmt, path::PathBuf, rc::Rc, sync::Mutex};

use logix::{env::Env, LocalFile, Logix, ManagedFile};

static GLOBAL_LOCK: Mutex<()> = Mutex::new(());

struct Inner {
    home: PathBuf,
    _logix: PathBuf,
    local_config: PathBuf,
    logix_config: PathBuf,
    logix_dotfile: PathBuf,
    _dir: tempfile::TempDir,
}

pub struct TestFs {
    inner: Rc<Inner>,
}

impl TestFs {
    pub fn new(root_logix: &str) -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let fs = TestFs {
            inner: Rc::new(Inner {
                home: dir.path().join("home/zeldor"),
                _logix: dir.path().join("home/zeldor/.config/logix"),
                local_config: dir.path().join("home/zeldor/.config"),
                logix_config: dir.path().join("home/zeldor/.config/logix/config"),
                logix_dotfile: dir.path().join("home/zeldor/.config/logix/dotfiles"),
                _dir: dir,
            }),
        };
        fs.write_config_file("logix/root.logix", root_logix);
        fs
    }

    pub fn write_config_file(&self, path: &str, data: &str) {
        self.write_home_file(&format!(".config/{path}"), data);
    }

    pub fn write_home_file(&self, path: &str, data: &str) {
        let path = self.inner.home.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, data).unwrap();
    }

    pub fn load_logix(&self) -> Loaded<Logix> {
        let env = self.init_env();
        Loaded {
            inner: env.inner,
            value: Logix::load(env.value).unwrap(),
        }
    }

    pub fn init_env(&self) -> Loaded<Env> {
        let _lock = GLOBAL_LOCK.lock().unwrap();
        let org_home = std::env::var_os("HOME").unwrap();
        std::env::set_var("HOME", &self.inner.home); // TODO: This is not enough to trick directories on all platforms
        let env = Env::init();
        std::env::set_var("HOME", org_home);
        Loaded {
            inner: self.inner.clone(),
            value: env.unwrap(),
        }
    }

    pub fn managed_logix_config(&self, name: &str) -> ManagedFile {
        ManagedFile::Local(LocalFile {
            local: self.inner.local_config.join(name),
            logix: self.inner.logix_config.join(name),
        })
    }

    pub fn managed_logix_dotfile(&self, name: &str) -> ManagedFile {
        ManagedFile::Local(LocalFile {
            local: self.inner.home.join(name),
            logix: self.inner.logix_dotfile.join(name),
        })
    }
}

pub struct Loaded<T> {
    inner: Rc<Inner>,
    value: T,
}

impl<T> std::ops::Deref for Loaded<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

pub fn compare_iters<T: PartialEq + fmt::Debug>(
    name: &str,
    mut want_it: impl Iterator<Item = T>,
    mut got_it: impl Iterator<Item = T>,
) -> bool {
    for i in 0.. {
        let want = want_it.next();
        let got = got_it.next();
        if want != got {
            eprintln!("*** ERROR: {name} at index {i} differs");
            eprintln!("Want: {want:#?}");
            eprintln!("Got:  {got:#?}");
            return false;
        } else if want.is_none() {
            return true;
        }
    }
    false
}
