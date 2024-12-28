use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time}; // for debugging, remove later


#[derive(Clone)]
pub struct KataGoInstaller {
    install_dir: PathBuf,
    katago_exe: PathBuf,
    mutex: Arc<Mutex<()>>,
}


impl KataGoInstaller {
    pub fn new(install_dir: &Path) -> Self {
	let mut katago_exe = String::from("katago");
	if cfg!(windows) {
	    katago_exe.push_str(".exe");
	}
	
	Self {
	    install_dir: install_dir.to_path_buf(),
	    katago_exe: install_dir.join(katago_exe),
	    mutex: Arc::new(Mutex::new(())),
	}
    }

    pub fn is_installed_try(&self) -> Result<bool, String> {
	let _guard = self.try_lock()?;
	Ok(self.katago_exe.exists())
    }

    pub fn is_installed(&self) -> bool {
	let _guard = self.lock();
	self.katago_exe.exists()
    }

    pub fn install(&self) -> Result<(), String> {
	// TODO
	thread::sleep(time::Duration::from_secs(3));
	Err("Not implemented.".to_string())
    }

    pub fn is_tuned(&self) -> bool {
	// TODO
	false
    }

    pub fn test(&self) -> Result<(), String> {
	// TODO
	Err("Not implemented.".to_string())
    }

    pub fn tune(&self) -> Result<(), String> {
	// TODO
	thread::sleep(time::Duration::from_secs(3));
	Err("Not implemented.".to_string())
    }

    fn try_lock(&self) -> Result<MutexGuard<'_, ()>, String> {
	self.mutex.try_lock().map_err(|_| "Cannot acquire lock! Another operation is ongoing.".to_string())
    }

    fn lock(&self) -> MutexGuard<'_, ()> {
	self.mutex.lock().unwrap()
    }
}
