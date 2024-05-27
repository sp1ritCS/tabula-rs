//!
//! # Rust bindings for tabulapdf/tabula-java
//! 
//! ## Prerequisites
//! In order to use tabula-rs, you will need a tabula-java bytecode archive (jar).
//! You can build it yourself by cloning <ssh://git@github.com/tabulapdf/tabula-java.git> and then running invoking [maven](https://software.opensuse.org/package/maven) to build it.
//! ```sh
//! git clone git@github.com:tabulapdf/tabula-java.git && cd tabula-java
//! git apply path/to/tabula-rs/0001-add-ffi-constructor-to-CommandLineApp.patch
//! mvn compile assembly:single
//! ```
//! the built archive should then be target/tabula-$TABULA_VER-jar-with-dependencies.jar.
//!
//! Additionally, make sure `$JAVA_HOME/lib/server/libjvm.so` is reachable through `LD_LIBRARY_PATH` or explicitly set it as `LD_PRELOAD`.
//!
//! This can look like this:
//! ```sh
//! export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$JAVA_HOME/lib/server/
//! ```
//!
//! ## Using tabula-rs
//! ### Initalizing JVM & accessing JNI
//! In order to make use of tabula-java, you'll need to start [jni::JavaVM] with the built archive added to its classpath.
//! You could either do this manually, or call [TabulaVM::new()]` with the (space escaped) path to the archive as parameter.
//! 
//! Using [TabulaVM] you can now access the Java native interface by calling [TabulaVM::attach()].
//! ```
//! # use tabula::TabulaVM;
//! let vm = TabulaVM::new("../tabula-java/target/tabula-1.0.6-SNAPSHOT-jar-with-dependencies.jar", false).unwrap();
//! let env = vm.attach().unwrap();
//! ```
//!
//! ### Instantiating Tabula class
//! with access to the JNI you can instantia the [Tabula] class by calling [TabulaEnv::configure_tabula()].
//! ```
//! # use tabula::{ExtractionMethod, OutputFormat, TabulaVM};
//! # let vm = TabulaVM::new("../tabula-java/target/tabula-1.0.6-SNAPSHOT-jar-with-dependencies.jar", false).unwrap();
//! # let env = vm.attach().unwrap();
//! let t = env.configure_tabula(None, None, OutputFormat::Csv, true, ExtractionMethod::Basic, false, None).unwrap();
//! ```
//!
//! ### Parsing the document
//! [Tabula] provides [Tabula::parse_document()] that then parses a document located a its given path and returns a [std::fs::File] located in memory.
//! ```
//! # use tabula::{ExtractionMethod, OutputFormat, TabulaVM};
//! # let vm = TabulaVM::new("../tabula-java/target/tabula-1.0.6-SNAPSHOT-jar-with-dependencies.jar", false).unwrap();
//! # let env = vm.attach().unwrap();
//! # let t = env.configure_tabula(None, None, OutputFormat::Csv, true, ExtractionMethod::Basic, false, None).unwrap();
//! let file = t.parse_document(&std::path::Path::new("./test_data/spanning_cells.pdf"), "test_spanning_cells").unwrap();
//! ```  
//! 
//! ## Relavant links
//! - tabula-rs forge: <https://github.com/sp1ritCS/tabula-rs>
//! - tabula-java project: <https://github.com/tabulapdf/tabula-java/>


mod mem_file;
mod objects;
use objects::{IntoJObject, Pair};
pub use objects::{RELATIVE_AREA_CALCULATION_MODE, ABSOLUTE_AREA_CALCULATION_MODE, Rectangle, OutputFormat, ExtractionMethod};

use anyhow::Result;
use jni::{AttachGuard, InitArgsBuilder, JNIEnv, JNIVersion, JavaVM, objects::{JObject, JValue}, errors::Error as JError};
pub use jni; // reexport

use std::result::Result as StdResult;
use std::ops::Deref;
use std::path::Path;

/// Result returned from JNI
pub type JResult<T> = StdResult<T, JError>;

///
/// # Java VM capable of using Tabula
///
/// Can be created using [TabulaVM::new()] or by putting a [jni::JavaVM] as it's first inner parameter 
///
pub struct TabulaVM(JavaVM);
impl <'env> TabulaVM {
	/// 
	/// Create a new Java VM capable of using Tabula
	///
	/// - `libpath`: Escaped path to `tabula-java.jar`
	/// - `debug`: runs jvm with `-Xcheck:jni`
	///
	pub fn new(libpath: &str, debug: bool) -> Result<Self> {
		let opt = format!("-Djava.class.path={}", libpath);
		let mut jvm_args = InitArgsBuilder::new()
			.version(JNIVersion::V8)
			.option(&opt);

		if debug {
			jvm_args = jvm_args.option("-Xcheck:jni");
		}
		
		let jvm_args = jvm_args.build()?;

		Ok(Self(JavaVM::new(jvm_args)?))
	}
	
	/// Get Java native interface capable of instantiating Tabula
	pub fn attach(&'env self) -> Result<TabulaEnv<'env>> {
		Ok(TabulaEnv(self.0.attach_current_thread()?))
	}
}


///
/// # Java native interface capable of instantiating Tabula class
///
/// received by calling [TabulaVM::attach()]
///
pub struct TabulaEnv<'env>(AttachGuard<'env>);

impl <'env> TabulaEnv<'env> {
	fn get_pages_jarray(&self, pages: &[i32]) -> JResult<*mut jni::sys::_jobject> {
		let null = JObject::null();
		let array = self.new_object_array(pages.len() as i32, "java/lang/Integer", null)?;
		for (i, pg) in pages.iter().enumerate() {
			self.set_object_array_element(array, i as i32, pg.get_jobject(self)?)?;
		}
		Ok(array)
	}
	
	fn get_page_areas_jarray(&self, page_areas: &[(i32, Rectangle)]) -> JResult<*mut jni::sys::_jobject> {
		let null = JObject::null();
		let array = self.new_object_array(page_areas.len() as i32, "technology/tabula/Pair", null)?;
		for (i, (mode, rect)) in page_areas.iter().enumerate() {
			let pga = Pair::new(*mode, *rect);
			self.set_object_array_element(array, i as i32, pga.get_jobject(self)?)?;
		}
		Ok(array)
	}
	
	///
	/// # Instantiate Tabula class 
	///
	/// - `page_areas`: Portion of the page to analyze. If mode is [Relative](crate::RELATIVE_AREA_CALCULATION_MODE) the [Rectangle](crate::Rectangle) will be taken as % of actual height or width of the page.
	/// - `pages`: Nullable slice (if None then all pages) to be parsed
	/// - `output_format`: [crate::OutputFormat]
	/// - `guess`: Guess the portion of the page to analyze per page.
	/// - `method`: [crate::ExtractionMethod]
	/// - `use_returns`: Use embedded line returns in cells. (Only in spreadsheet mode.)
	/// - `password`: Password to decrypt document. None in case of no password.
	///
	#[allow(clippy::too_many_arguments)]
	pub fn configure_tabula(&self,
		page_areas: Option<&[(i32, Rectangle)]>,
		pages: Option<&[i32]>,
		output_format: OutputFormat,
		guess: bool,
		method: ExtractionMethod,
		use_returns: bool,
		password: Option<&str>
	) -> JResult<Tabula> {
		let areas = if let Some(page_areas) = page_areas {
			JValue::from(self.get_page_areas_jarray(page_areas)?)
		} else {
			JValue::from(JObject::null())
		};
		let pages = if let Some(pages) = pages {
			JValue::from(self.get_pages_jarray(pages)?)
		} else {
			JValue::from(JObject::null())
		};
		let password = password
			.and_then(|pw| self.new_string(pw).ok())
			.map(JValue::from)
			.unwrap_or(JValue::from(JObject::null()));
		let tabula = self.new_object("technology/tabula/CommandLineApp", "([Ltechnology/tabula/Pair;[Ljava/lang/Integer;Ltechnology/tabula/CommandLineApp$OutputFormat;ZLtechnology/tabula/CommandLineApp$ExtractionMethod;ZLjava/lang/String;)V", &[
			areas,
			pages,
			JValue::from(output_format.get_jobject(self)?),
			JValue::from(guess),
			JValue::from(method.get_jobject(self)?),
			JValue::from(use_returns),
			password
		])?;

		Ok(Tabula {
			env: self,
			inner: tabula
		})
	}
}

impl <'env> Deref for TabulaEnv<'env> {
	type Target = JNIEnv<'env>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

///
/// # Tabula class
///
/// received by calling [TabulaEnv::configure_tabula()]
///
pub struct Tabula<'env> {
	env: &'env TabulaEnv<'env>,
	inner: JObject<'env>
}

impl Tabula<'_> {
	///
	/// # Parse document located at `path`.
	///
	/// `descriptor_name` refers to the filename passed to [memfd_create()](https://git.kernel.org/pub/scm/docs/man-pages/man-pages.git/tree/man2/memfd_create.2)
	///
	pub fn parse_document(&self, path: &Path, descriptor_name: &str) -> Result<std::fs::File> {
		let output = unsafe { mem_file::TmpMemFile::new(descriptor_name) }?;

		let file = path.get_jobject(self.env)?;
		let outfile = unsafe { output.get_path() }.get_jobject(self.env)?;
		
		self.env.call_method(*self.deref(), "extractFileInto", "(Ljava/io/File;Ljava/io/File;)V", &[
			JValue::Object(file),
			JValue::Object(outfile)
		])?;

		let file = unsafe { output.get_file() };
		Ok(file)
	}
}

impl <'env> Deref for Tabula<'env> {
	type Target = JObject<'env>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[cfg(test)]
mod tests;
