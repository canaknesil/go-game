use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use reqwest;
use std::fs;
use std::io::{self, BufRead, Read};
use std::error::Error;
use zip::ZipArchive;
use crate::child_process_engine::ChildProcessEngine;
use crate::gtp::GTPEngineMinimal;
use std::process;


#[derive(Clone)]
pub struct KataGoInstaller {
    path_info: Option<PathInfo>,
    mutex: Arc<Mutex<()>>,
}

#[derive(Clone)]
struct PathInfo {
    katago_url: String,
    katago_zip: PathBuf,
    katago_dir: PathBuf,
    katago_exe: PathBuf,
    analysis_model_url: String,
    analysis_model: PathBuf,
    human_model_url: String,
    human_model: PathBuf,
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

	let analysis_model_url = "https://github.com/lightvector/KataGo/releases/download/v1.4.5/g170e-b20c256x2-s5303129600-d1228401921.bin.gz";
	let analysis_model = "g170e-b20c256x2-s5303129600-d1228401921.bin.gz";
	let human_model_url = "https://github.com/lightvector/KataGo/releases/download/v1.15.0/b18c384nbt-humanv0.bin.gz";
	let human_model = "b18c384nbt-humanv0.bin.gz";

	let path_info = url_and_dir.map(|(url, dir)| {
	    let zip = install_dir.join(format!("{dir}.zip"));
	    let dir = install_dir.join(dir);
	    
	    let mut exe = String::from("katago");
	    if cfg!(windows) {
		exe.push_str(".exe");
	    }
	    let exe = dir.join(exe);
	    
	    PathInfo {
		katago_url: url.to_string(),
		katago_zip: zip,
		katago_dir: dir,
		katago_exe: exe,
		analysis_model_url: analysis_model_url.to_string(),
		analysis_model: install_dir.join(analysis_model),
		human_model_url: human_model_url.to_string(),
		human_model: install_dir.join(human_model),
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

    // fn try_lock(&self) -> Result<MutexGuard<'_, ()>, String> {
    // 	self.mutex.try_lock().map_err(|_| "Cannot acquire lock! Another operation is ongoing.".to_string())
    // }

    fn lock(&self) -> MutexGuard<'_, ()> {
	self.mutex.lock().unwrap()
    }

    // pub fn is_installed_try(&self) -> Result<bool, String> {
    // 	let exe = &self.get_path_info()?.exe;
    // 	let _guard = self.try_lock()?;
    // 	Ok(exe.exists())
    // }

    pub fn is_installed(&self) -> bool {
	let _guard = self.lock();
	self.is_installed_without_lock()
    }

    fn is_installed_without_lock(&self) -> bool {
	match self.get_path_info() {
	    Ok(path_info) => {
		path_info.katago_exe.exists()
	    },
	    Err(s) => {
		println!("Returning false from is_installed, path_info not setup. {s}");
		false
	    }
	}
    }

    pub fn install(&self) -> Result<(), String> {
	let _guard = self.lock();

	println!("Installing KataGo archieve...");

	let dir = &self.get_path_info()?.katago_dir;
	println!("Installation directory: {:?}", dir);
	
	self.download_all().map_err(|e| e.to_string())?;
	self.extract().map_err(|e| e.to_string())?;
	if !self.is_installed_without_lock() {
	    return Err("Extraction unsuccessful!".to_string());
	}
	self.tune()?;
	if !self.is_tuned_without_lock() {
	    println!("Tuning unsuccessful!");
	}

	Ok(())
    }

    fn tune(&self) -> Result<(), String> {
	let pi = &self.get_path_info()?;
	let exe = &pi.katago_exe;
	let exe = exe.to_str().ok_or(format!("Cannot convert path to string: {exe:?}"))?;
	let model = &pi.analysis_model;
	let model = model.to_str().ok_or(format!("Cannot convert path to string: {model:?}"))?;

	// TODO: Consider refactoring code below to a global function in child_process_engine.
	// TODO: Consider creating a child wrapper implementing drop.
	let command = vec![format!("{exe}"), "benchmark".to_string(), "-model".to_string(), format!("{model}")];
	println!("Running command: {command:?}");

	let mut child = process::Command::new(&command[0])
	    .args(&command[1..])
	    .stdout(process::Stdio::piped())
	    .stderr(process::Stdio::piped())
	    .spawn()
	    .map_err(|_| format!("Failed to start child process: {:?}", command))?;

	let stdout = child.stdout.take().ok_or("Failed to get stdout!".to_string())?;
	let stderr = child.stderr.take().ok_or("Failed to get stderr!".to_string())?;
	let reader = io::BufReader::new(stdout.chain(stderr));
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(err) => eprintln!("Error reading line: {}", err),
            }
        }

	let status = child.wait().map_err(|_| "Error waiting child process!".to_string())?;
	println!("Benchmark process exited with status: {}", status);
	
	Ok(())
    }

    pub fn is_tuned(&self) -> bool {
	let _guard = self.lock();
	self.is_tuned_without_lock()
    }

    fn is_tuned_without_lock(&self) -> bool {
	match self.get_path_info() {
	    Ok(path_info) => {
		path_info.katago_dir.join("KataGoData").join("opencltuning").exists()
	    },
	    Err(s) => {
		println!("Returning false from is_installed, path_info not setup. {s}");
		false
	    }
	}
    }

    pub fn test(&self) -> Result<String, String> {
	let _guard = self.lock();
	self.test_without_lock()
    }

    fn test_without_lock(&self) -> Result<String, String> {
	let pi = &self.get_path_info()?;
	let exe = &pi.katago_exe;
	let exe = exe.to_str().ok_or(format!("Cannot convert path to string: {exe:?}"))?;
	let model = &pi.analysis_model;
	let model = model.to_str().ok_or(format!("Cannot convert path to string: {model:?}"))?;

	let command = format!("{exe} gtp -model {model}");
	let mut engine = ChildProcessEngine::new(&command)?;
	let version = engine.version();
	engine.quit()?;
	version
    }

    fn download_all(&self) -> Result<(), Box<dyn Error>> {
	let pi = self.get_path_info()?;
	let dir = &pi.katago_dir;
	let install_dir = pi.katago_dir.parent().ok_or(format!("{dir:?} does not have a parent! It appears to be the root directory. Not proceeding with download."))?;

	// Create installation directory
	fs::create_dir_all(&install_dir)?;

	// Download KataGo
	download(&pi.katago_url, &pi.katago_zip)?;

	// Download models
	download(&pi.analysis_model_url, &pi.analysis_model)?;
	download(&pi.human_model_url, &pi.human_model)?;
	
	Ok(())
    }

    fn extract(&self) -> Result<(), Box<dyn Error>> {
	let path_info = self.get_path_info()?;
	let zip = &path_info.katago_zip;
	let dir = &path_info.katago_dir;

	println!("Extracting '{zip:?}' to '{dir:?}'...");
	
	if !zip.exists() {
	    return Err(format!("Zip file does not exist: {zip:?}").into());
	}

	let file = fs::File::open(zip)?;
	let mut archive = ZipArchive::new(file)?;

	if dir.exists() {
	    println!("{dir:?} exists. Deleting...");
	    fs::remove_dir_all(dir)?;
	}

	for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = Path::new(dir).join(file.name());

	    println!("{:?}", out_path);
            if file.is_dir() {
		fs::create_dir_all(&out_path)?;
            } else {
		if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
		}
		let mut out_file = fs::File::create(&out_path)?;
		io::copy(&mut file, &mut out_file)?;
            }
	}

	Ok(())
    }
}


fn download(url: &str, file_path: &Path) -> Result<bool, Box<dyn Error>> {
    // Return true if file doesn't exist and download is successful.
    // Return false if file exists.
    if file_path.exists() {
	println!("{file_path:?} already exists. Not downloading again.");
	Ok(false)
    } else {
	println!("Downloading from '{url}' to '{file_path:?}'...");
	let content = reqwest::blocking::get(url)?.bytes()?;
	fs::write(file_path, content)?;
	Ok(true)
    }
}
