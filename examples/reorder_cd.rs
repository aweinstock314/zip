use std::io::{Read, Write, Cursor};
use zip::{spec::{CentralDirectoryEnd, CentralDirectoryHeader, LocalFileHeader, GeneralPurposeBitFlags}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "test.zip";
    let path = std::path::Path::new(filename);
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut cursor = Cursor::new(bytes.clone());

    let (mut eocd, _) = CentralDirectoryEnd::find_and_parse(&mut cursor)?;
    let cd_start = eocd.central_directory_offset as usize;
    let cd_end = cd_start + eocd.central_directory_size as usize;
    let cd = &bytes[cd_start..cd_end];

	let mut mask = LocalFileHeader {
		version_to_extract: 2,
		flags: GeneralPurposeBitFlags(0),
		compression_method: 0,
		last_mod_time: 0,
		last_mod_date: 0,
		crc32: 0,
		compressed_size: 0,
		uncompressed_size: 0,
		file_name_raw: b"mask".to_vec(),
		extra_field: Vec::new(),
	};
	let padding = mask.len() + eocd.len();
	let mut output = vec![0; padding + bytes.len()];
	let mut cdh_i = mask.len();
	let mut lfh_i = padding + eocd.central_directory_size as usize;

	let mut cd_cursor = Cursor::new(cd);
	while let Ok(mut cdh) = CentralDirectoryHeader::parse(&mut cd_cursor) {
		let lfh_start = cdh.offset as usize;
		let lfh_len = 30 + cdh.file_name_raw.len() + cdh.extra_field.len();
		let lfh_end = lfh_start + lfh_len;
		let data_end = lfh_end + cdh.compressed_size as usize;
		let file_data = &bytes[lfh_start..lfh_end];
		let mut file_cursor = Cursor::new(file_data);
		let lfh = LocalFileHeader::parse(&mut file_cursor)?;
		let file_bytes = &bytes[lfh_end..data_end];
		cdh.offset = lfh_i as u32;
		cdh.write(&mut &mut output[cdh_i..cdh_i + cdh.len()])?;
		cdh_i += cdh.len();
		lfh.write(&mut &mut output[lfh_i..lfh_i+lfh_len])?;
		lfh_i += lfh_len;
		(&mut output[lfh_i..lfh_i+file_bytes.len()]).copy_from_slice(&file_bytes);
		lfh_i += file_bytes.len();
	}
	eocd.central_directory_offset = mask.len() as u32;
	eocd.write(&mut &mut output[lfh_i..])?;
	eocd.write(&mut &mut output[cdh_i..cdh_i+eocd.len()])?;

	mask.compressed_size = (lfh_i - mask.len()) as u32;
	mask.uncompressed_size = (lfh_i - mask.len()) as u32;
	mask.write(&mut &mut output[0..mask.len()])?;

	if let Ok(mut outfile) = std::fs::File::create("test2.zip") {
		outfile.write_all(&output[..])?;
	}

    Ok(())
}
