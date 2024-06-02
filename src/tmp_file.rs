use std::path::PathBuf;
use std::fs::File;

use anyhow::Result;

pub trait TempFile where Self: Sized {
	fn new<T: AsRef<str>>(name: T) -> Result<Self>;

	fn get_path(&self) -> PathBuf;

	fn into_file(self) -> File;
}

#[cfg(target_os = "linux")]
pub fn new<T: AsRef<str>>(name: T) -> Result<impl TempFile> {
	LinuxMemFd::new(name)
}

#[cfg(not(target_os = "linux"))]
pub fn new<T: AsRef<str>>(name: T) -> Result<impl TempFile> {
	GenericTempFile::new(name)
}

#[cfg(target_os = "linux")]
pub struct LinuxMemFd(i32);

#[cfg(target_os = "linux")]
impl TempFile for LinuxMemFd {
	fn new<T: AsRef<str>>(name: T) -> Result<Self> {
		let name = std::ffi::CString::new(name.as_ref())?;
		let fd = unsafe {
			libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 0) as i32
		};

		Ok(Self(fd))
	}

	fn get_path(&self) -> PathBuf {
		let mut path = PathBuf::new();
		path.push("/proc");
		path.push(std::process::id().to_string());
		path.push("fd");
		path.push(self.0.to_string());
		path
	}

	fn into_file(self) -> File {
		unsafe { <File as std::os::fd::FromRawFd>::from_raw_fd(self.0) }
	}
}

pub struct GenericTempFile(tempfile::NamedTempFile, File);

impl TempFile for GenericTempFile {
	fn new<T: AsRef<str>>(name: T) -> Result<Self> {
		let prefix = <std::ffi::OsString as std::str::FromStr>::from_str(name.as_ref())?;
		let tf = tempfile::NamedTempFile::with_prefix(&prefix)?;
		let f = tf.reopen()?;
		Ok(Self(tf, f))
	}

	fn get_path(&self) -> PathBuf {
		self.0.path().to_owned()
	}

	fn into_file(self) -> File {
		self.1
	}
}

#[cfg(test)]
mod tests {
	use std::{fs::File, io::{Read, Write}};

	use super::{TempFile, GenericTempFile};

	#[cfg(target_os = "linux")]
	#[test]
	fn test_linux_mem_fd() -> Result<(), anyhow::Error> {
    use super::LinuxMemFd;
		common_test::<LinuxMemFd>()
	}

	#[test]
	fn generic_temp_file() -> Result<(), anyhow::Error> {
		common_test::<GenericTempFile>()
	}

	fn common_test<T: TempFile>() -> Result<(), anyhow::Error> {
		const TEST_DATA: &[u8] = b"test data";

		let tf = GenericTempFile::new("name")?;

		let path = tf.get_path();
		let mut write_file = File::create(&path)?;
		write_file.write_all(TEST_DATA)?;
		drop(write_file);

		let mut read_data = Vec::new();
		let mut read_file = tf.into_file();
		read_file.read_to_end(&mut read_data)?;

		assert_eq!(TEST_DATA, &read_data, "file did not contain the written data");

		drop(read_file);

		assert!(!path.exists(), "temporary file not removed after all usage completed");

		Ok(())
	}
}
