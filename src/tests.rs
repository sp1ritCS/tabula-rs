use super::{TabulaVM, Rectangle, OutputFormat, ExtractionMethod, ABSOLUTE_AREA_CALCULATION_MODE};

use std::io::Read;

use lazy_static::lazy_static;

lazy_static! {
	static ref VM: anyhow::Result<TabulaVM> = {
		TabulaVM::new("../tabula-java/target/tabula-1.0.6-SNAPSHOT-jar-with-dependencies.jar", true)
	};
}

#[test]
fn parse_document() {
	let vm = &VM.as_ref().unwrap();
	let env = vm.attach().unwrap();
	let areas: Vec<(i32, Rectangle)> = vec![(ABSOLUTE_AREA_CALCULATION_MODE, Rectangle::from_coords(58.9, 150.56, 654.7, 596.12))];
	let tabula = env.configure_tabula(Some(&areas), Some(&vec![1]), OutputFormat::Csv, false, ExtractionMethod::Decide, false, None).unwrap();
	let mut file = tabula.parse_document(&std::path::Path::new("./test_data/spanning_cells.pdf"), "test_spanning_cells").unwrap();
	let mut fin = String::new();
	file.read_to_string(&mut fin).unwrap();
	
	let cmp = std::fs::read_to_string("./test_data/spanning_cells.csv").unwrap();
	
	assert_eq!(fin, cmp);
}

#[test]
fn parse_document_into() {
	let vm = &VM.as_ref().unwrap();
	let env = vm.attach().unwrap();
	let areas: Vec<(i32, Rectangle)> = vec![(ABSOLUTE_AREA_CALCULATION_MODE, Rectangle::from_coords(58.9, 150.56, 654.7, 596.12))];
	let tabula = env.configure_tabula(Some(&areas), Some(&vec![1]), OutputFormat::Csv, false, ExtractionMethod::Decide, false, None).unwrap();
	let mut file = tempfile::NamedTempFile::new().unwrap();
	tabula.parse_document_into(&std::path::Path::new("./test_data/spanning_cells.pdf"), file.path()).unwrap();
	let mut fin = String::new();
	file.read_to_string(&mut fin).unwrap();

	let cmp = std::fs::read_to_string("./test_data/spanning_cells.csv").unwrap();

	assert_eq!(fin, cmp);
}
