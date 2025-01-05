use std::process::{self, Child, ChildStdin, ChildStdout, ChildStderr};
use std::io::{Read, Chain};


// Child that is killed when dropped. Plus, some other features.
pub struct SmartChild {
    child: Child,
}


impl SmartChild {
    pub fn from_child(child: Child) -> Self {
	Self {
	    child: child,
	}
    }

    pub fn from_command_str(command: &str) -> Result<Self, String> {
	let (program, args) = parse_command_str(&command)?;

	let child = process::Command::new(program)
	    .args(args)
	    .stdin(process::Stdio::piped())
	    .stdout(process::Stdio::piped())
	    .stderr(process::Stdio::piped())
	    .spawn()
	    .map_err(|_| format!("Failed to start child process: {:?}", command))?;

	Ok(Self::from_child(child))
    }

    pub fn take_stdin(&mut self) -> Result<ChildStdin, String> {
	self.child.stdin.take().ok_or("Failed to take stdin!".to_string())
    }

    pub fn take_stdout(&mut self) -> Result<ChildStdout, String> {
	self.child.stdout.take().ok_or("Failed to take stdout!".to_string())
    }

    pub fn take_stderr(&mut self) -> Result<ChildStderr, String> {
	self.child.stderr.take().ok_or("Failed to take stderr!".to_string())
    }

    pub fn take_stdout_and_stderr(&mut self) -> Result<Chain<ChildStdout, ChildStderr>, String> {
	Ok(self.take_stdout()?.chain(self.take_stderr()?))
    }

    pub fn wait(&mut self) -> Result<process::ExitStatus, String> {
	self.child.wait().map_err(|_| "Error waiting child!".to_string())
    }

    pub fn try_wait(&mut self) -> Result<Option<process::ExitStatus>, String> {
	self.child.try_wait().map_err(|_| "Error at try_wait child!".to_string())
    }
}


impl Drop for SmartChild {
    fn drop(&mut self) {
	match self.child.try_wait() {
	    Ok(Some(_status)) => {
		// child exited with status
		println!("Child process already exited. No need to kill.");
	    },
	    Ok(None) => {
		// child has not exited, kill it
		println!("Child process has not exited, killing!");
		let _ = self.child.kill();
	    }
	    Err(_) => (),
	}
    }
}


pub fn parse_command_str(command: &str) -> Result<(&str, Vec<&str>), String> {
    let mut args = command.split_whitespace();
    let program = args.next().ok_or("Empty command")?;
    let args: Vec<_> = args.collect();
    Ok((program, args))
}


