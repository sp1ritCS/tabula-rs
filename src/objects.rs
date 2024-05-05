use jni::objects::{JValue, JObject};
use super::{JResult, TabulaEnv};

// jobject trait impl
pub trait IntoJObject<'env> {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>>;
}

// constants
pub const RELATIVE_AREA_CALCULATION_MODE: i32 = 0;
pub const ABSOLUTE_AREA_CALCULATION_MODE: i32 = 1;

// jobject impls
impl <'env> IntoJObject<'env> for JObject<'env> {
	fn get_jobject(&self, _env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		Ok(self.clone())
	}
}

impl <'env> IntoJObject<'env> for i32 {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		env.new_object("java/lang/Integer", "(I)V", &[JValue::from(*self)])
	}
}

impl <'env> IntoJObject<'env> for std::path::Path {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		let path = env.new_string(self.to_string_lossy())?;
		env.new_object("java/io/File", "(Ljava/lang/String;)V", &[JValue::from(path)])
	}
}

// structs
#[derive(Debug, Clone, Copy)]
///
/// # Oxidized `technology.tabula.Rectangle`
///
/// defines an area on a Page using either relative (%) or a absolute (xy) values
///
pub struct Rectangle {
	left: f32,
	top: f32,
	width: f32,
	height: f32
}
impl Rectangle {
	pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
		Self {
			left,
			top,
			width,
			height
		}
	}
	
	pub fn from_coords(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
		Self {
			left: x1,
			top: y1,
			width: x2-x1,
			height: y2-y1
		}
	}
}
impl <'env> IntoJObject<'env> for Rectangle {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		env.new_object("technology/tabula/Rectangle", "(FFFF)V", &[
			JValue::from(self.top),
			JValue::from(self.left),
			JValue::from(self.width),
			JValue::from(self.height)
		])
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Pair<'env, K: IntoJObject<'env>, V: IntoJObject<'env>>(
	K,
	V,
	#[allow(dead_code)] &'env std::marker::PhantomData<()>,
);
impl <'env, K: IntoJObject<'env>, V: IntoJObject<'env>> Pair<'env, K, V> {
	pub fn new(left: K, right: V) -> Self {
		Self(left, right, &std::marker::PhantomData)
	}
	pub fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		env.new_object("technology/tabula/Pair", "(Ljava/lang/Object;Ljava/lang/Object;)V", &[
			JValue::from(self.0.get_jobject(env)?),
			JValue::from(self.1.get_jobject(env)?)
		])
	}
}


// ENUMS
#[derive(Debug, Clone, Copy)]
/// Oxidized `technology.tabula.CommandLineApp$OutputFormat`
pub enum OutputFormat {
	Csv,
	Json,
	Tsv
}
impl <'env> IntoJObject<'env> for OutputFormat {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		let field = match self {
			OutputFormat::Csv => "CSV",
			OutputFormat::Json => "JSON",
			OutputFormat::Tsv => "TSV"
		};
		env.get_static_field("technology/tabula/CommandLineApp$OutputFormat", field, "Ltechnology/tabula/CommandLineApp$OutputFormat;")?.l()
	}
}

/// Oxidized `technology.tabula.CommandLineApp$ExtractionMethod`
#[derive(Debug, Clone, Copy)]
pub enum ExtractionMethod {
	Basic,
	Spreadsheet,
	Decide
}
impl <'env> IntoJObject<'env> for ExtractionMethod {
	fn get_jobject(&self, env: &'env TabulaEnv) -> JResult<JObject<'env>> {
		let field = match self {
			ExtractionMethod::Basic => "BASIC",
			ExtractionMethod::Spreadsheet => "SPREADSHEET",
			ExtractionMethod::Decide => "DECIDE"
		};
		env.get_static_field("technology/tabula/CommandLineApp$ExtractionMethod", field, "Ltechnology/tabula/CommandLineApp$ExtractionMethod;")?.l()
	}
}
