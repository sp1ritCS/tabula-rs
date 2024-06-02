use std::path::PathBuf;
use std::fs::File;

use anyhow::Result;

#[cfg(target_os = "linux")]
pub struct TmpFile(i32);

#[cfg(target_os = "linux")]
impl TmpFile {
	pub fn new<T: AsRef<str>>(name: T) -> Result<Self> {
		let name = std::ffi::CString::new(name.as_ref())?;
		let fd = unsafe {
			libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 0) as i32
		};

		Ok(Self(fd))
	}

	pub fn get_path(&self) -> PathBuf {
		let mut path = PathBuf::new();
		path.push("/proc");
		path.push(std::process::id().to_string());
		path.push("fd");
		path.push(self.0.to_string());
		path
	}

	pub fn get_file(self) -> File {
		unsafe { <File as std::os::fd::FromRawFd>::from_raw_fd(self.0) }
	}
}

#[cfg(not(target_os = "linux"))]
pub struct TmpFile(tempfile::NamedTempFile, File);

#[cfg(not(target_os = "linux"))]
impl TmpFile {
	pub fn new<T: AsRef<str>>(name: T) -> Result<Self> {
		let prefix = <std::ffi::OsString as std::str::FromStr>::from_str(name.as_ref())?;
		let tf = tempfile::NamedTempFile::with_prefix(&prefix)?;
		let f = tf.reopen()?;
		Ok(Self(tf, f))
	}

	pub fn get_path(&self) -> PathBuf {
		self.0.path().to_owned()
	}

	pub fn get_file(self) -> File {
		self.1
	}
}
