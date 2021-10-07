use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::os::unix::io::FromRawFd;

pub struct TmpMemFile(isize);

impl TmpMemFile {
	pub unsafe fn new<T: AsRef<str>>(name: T) -> Result<Self> {
		let name = std::ffi::CString::new(name.as_ref())?;
		let fd = libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 0) as isize;

		Ok(Self(fd))
	}

	pub unsafe fn get_path(&self) -> PathBuf {
		let mut path = PathBuf::new();
		path.push("/proc");
		path.push(libc::getpid().to_string());
		path.push("fd");
		path.push(self.0.to_string());
		path
	}

	pub unsafe fn get_file(&self) -> File {
		File::from_raw_fd(self.0 as i32)
	}
}