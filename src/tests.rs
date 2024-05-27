use super::{TabulaVM, Rectangle, OutputFormat, ExtractionMethod, ABSOLUTE_AREA_CALCULATION_MODE};
use std::io::Read;

use lazy_static::lazy_static;

lazy_static! {
	static ref VM: anyhow::Result<TabulaVM> = {
		TabulaVM::new("../tabula-java/target/tabula-1.0.6-SNAPSHOT-jar-with-dependencies.jar", true)
	};
}

#[test]
fn parse_document() -> Result<(), anyhow::Error> {
	let vm = &VM.as_ref().unwrap();
	let env = vm.attach()?;
	let areas: Vec<(i32, Rectangle)> = vec![(ABSOLUTE_AREA_CALCULATION_MODE, Rectangle::from_coords(58.9, 150.56, 654.7, 596.12))];
	let tabula = env.configure_tabula(Some(&areas), Some(&[1]), OutputFormat::Csv, false, ExtractionMethod::Decide, false, None)?;
	let mut file = tabula.parse_document(std::path::Path::new("./test_data/spanning_cells.pdf"), "test_spanning_cells")?;
	let mut fin = String::new();
	file.read_to_string(&mut fin)?;
	
	let cmp = std::fs::read_to_string("./test_data/spanning_cells.csv")?;
	
	assert_eq!(fin, cmp);
	Ok(())
}

#[test]
fn parse_document_into() -> Result<(), anyhow::Error> {
	let vm = &VM.as_ref().unwrap();
	let env = vm.attach()?;
	let areas: Vec<(i32, Rectangle)> = vec![(ABSOLUTE_AREA_CALCULATION_MODE, Rectangle::from_coords(58.9, 150.56, 654.7, 596.12))];
	let tabula = env.configure_tabula(Some(&areas), Some(&vec![1]), OutputFormat::Csv, false, ExtractionMethod::Decide, false, None)?;
	let mut file = tempfile::NamedTempFile::new()?;
	tabula.parse_document_into(&std::path::Path::new("./test_data/spanning_cells.pdf"), file.path())?;
	let mut fin = String::new();
	file.read_to_string(&mut fin)?;

	let cmp = std::fs::read_to_string("./test_data/spanning_cells.csv")?;

	assert_eq!(fin, cmp);
    Ok(())
}
