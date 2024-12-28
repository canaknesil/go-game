use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time}; // for debugging, remove later
use reqwest;
use std::fs;
use std::error::Error;


#[derive(Clone)]
pub struct KataGoInstaller {
    path_info: Option<PathInfo>,
    mutex: Arc<Mutex<()>>, // TODO: Make sure to properly use this
}

#[derive(Clone)]
struct PathInfo {
    url: String,
    zip: PathBuf,
    dir: PathBuf,
    exe: PathBuf,    
}


impl KataGoInstaller {
    pub fn new(install_dir: &Path) -> Self {
	let url_and_dir = if cfg!(target_os = "windows") {
	    Some(("https://github.com/lightvector/KataGo/releases/download/v1.15.3/katago-v1.15.3-opencl-windows-x64.zip",
		  "katago-v1.15.3-opencl-windows-x64"))
	} else if cfg!(target_os = "linux") {
	    Some(("https://github.com/lightvector/KataGo/releases/download/v1.15.3/katago-v1.15.3-opencl-linux-x64.zip",
		  "katago-v1.15.3-opencl-linux-x64"))
	} else {
	    println!("KataGo doesn't support target_os '{}'!", std::env::consts::OS);
	    None
	};

	let path_info = url_and_dir.map(|(url, dir)| {
	    let zip = install_dir.join(format!("{dir}.zip"));
	    let dir = install_dir.join(dir);
	    
	    let mut exe = String::from("katago");
	    if cfg!(windows) {
		exe.push_str(".exe");
	    }
	    let exe = dir.join(exe);
	    PathInfo {
		url: url.to_string(),
		zip: zip,
		dir: dir,
		exe: exe,
	    }
	});

	Self {
	    path_info: path_info,
	    mutex: Arc::new(Mutex::new(())),
	}
    }

    fn get_path_info(&self) -> Result<&PathInfo, String> {
	match &self.path_info {
	    Some(path_info) => Ok(&path_info),
	    None => Err("KataGo has not been initialized with path information!".to_string()),
	}
    }

    fn try_lock(&self) -> Result<MutexGuard<'_, ()>, String> {
	self.mutex.try_lock().map_err(|_| "Cannot acquire lock! Another operation is ongoing.".to_string())
    }

    fn lock(&self) -> MutexGuard<'_, ()> {
	self.mutex.lock().unwrap()
    }

    pub fn is_installed_try(&self) -> Result<bool, String> {
	// TODO: Implement is_installed in a better way.
	let exe = &self.get_path_info()?.exe;
	let _guard = self.try_lock()?;
	Ok(exe.exists())
    }

    pub fn is_installed(&self) -> bool {
	// TODO: Implement is_installed in a better way.
	match self.get_path_info() {
	    Ok(path_info) => {
		let _guard = self.lock();
		path_info.exe.exists()
	    },
	    Err(s) => {
		println!("Returning false from is_installed: {s}");
		false
	    }
	}
    }

    pub fn install(&self) -> Result<(), String> {
	let dir = &self.get_path_info()?.dir;
	println!("Installing KataGo archieve...");
	println!("Installation directory: {:?}", dir);
	self.download().map_err(|e| e.to_string())?;
	// TODO
	// extract
	// check is_installed
	// test
	
	//thread::sleep(time::Duration::from_secs(3));

	Err("Not implemented.".to_string())
    }

    pub fn test(&self) -> Result<(), String> {
	// TODO
	Err("Not implemented.".to_string())
    }

    pub fn is_tuned(&self) -> bool {
	// TODO
	false
    }

    pub fn tune(&self) -> Result<(), String> {
	// TODO
	thread::sleep(time::Duration::from_secs(3));
	Err("Not implemented.".to_string())
    }

    fn download(&self) -> Result<(), Box<dyn Error>> {
	let path_info = self.get_path_info()?;
	let dir = &path_info.dir;
	let install_dir = path_info.dir.parent().ok_or(format!("{dir:?} does not have a parent! It appears to be the root directory. Not proceeding with download."))?;
	let zip = &path_info.zip;
	let url = &path_info.url;

	fs::create_dir_all(&install_dir)?;

	if zip.exists() {
	    println!("{zip:?} already exists. Not downloading again.");
	} else {
	    println!("Downloading '{url}' to '{zip:?}'...");
	    let content = reqwest::blocking::get(url)?.bytes()?;
	    fs::write(zip, content)?;
	}
	Ok(())
    }
}
