use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Read, Write, Cursor};
use zip::{spec::{CentralDirectoryEnd, CentralDirectoryHeader, LocalFileHeader, GeneralPurposeBitFlags}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let argv = ::std::env::args().collect::<Vec<_>>();
	if argv.len() < 3 {
		println!("Usage: reorder_cd input.zip output.zip");
		return Ok(());
	}
    let mut file = std::fs::File::open(&argv[1])?;
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
		compressed_size: 8,
		uncompressed_size: 8,
		file_name_raw: b"mask".to_vec(),
		extra_field: Vec::new(),
	};
	let padding = mask.len() + 8;
	//let mut output = vec![0; mask.len() + (1 << 20) + bytes.len()];
	let mut cdh_i = 0;
	let mut lfh_i = padding;
	let mut cdhs = Vec::new();
	let mut lfhs = Vec::new();

	let mut cd_cursor = Cursor::new(cd);
	while let Ok(mut cdh) = CentralDirectoryHeader::parse(&mut cd_cursor) {
		let lfh_start = cdh.offset as usize;
		let file_data = &bytes[lfh_start..];
		let mut file_cursor = Cursor::new(file_data);
		let lfh = LocalFileHeader::parse(&mut file_cursor)?;
		let lfh_end = lfh_start + lfh.len();
		let data_end = lfh_end + cdh.compressed_size as usize;
		let file_bytes = &bytes[lfh_end..data_end];
		cdh.offset = lfh_i as u32;
		//cdh.write(&mut &mut output[cdh_i..cdh_i + cdh.len()])?;
		cdh_i += cdh.len();
		cdhs.push(cdh);
		//lfh.write(&mut &mut output[lfh_i..lfh_i+lfh.len()])?;
		lfh_i += lfh.len();
		//(&mut output[lfh_i..lfh_i+file_bytes.len()]).copy_from_slice(&file_bytes);
		lfh_i += file_bytes.len();
		lfhs.push((lfh, file_bytes.to_vec()));
	}

	let mut output = vec![0; cdh_i + lfh_i + eocd.len()];
	let mut lfh_i = padding;
	for (lfh, file_bytes) in lfhs.iter() {
		println!("{:06x}: {:?}", lfh_i, lfh);
		lfh.write(&mut &mut output[lfh_i..lfh_i+lfh.len()])?;
		lfh_i += lfh.len();
		(&mut output[lfh_i..lfh_i+file_bytes.len()]).copy_from_slice(&file_bytes);
		lfh_i += file_bytes.len();
	}
	let mut cdh2_i = lfh_i;
	for cdh in cdhs.iter() {
		println!("{:06x}: {:?}", cdh2_i, cdh);
		cdh.write(&mut &mut output[cdh2_i..cdh2_i + cdh.len()])?;
		cdh2_i += cdh.len();
	}

	assert_eq!(eocd.central_directory_size as usize, cdh2_i - lfh_i);
	//eocd.central_directory_offset = mask.len() as u32;
	//eocd.write(&mut &mut output[cdh_i..cdh_i+eocd.len()])?;
	eocd.central_directory_offset = lfh_i as u32;
	eocd.write(&mut &mut output[cdh2_i..cdh2_i+eocd.len()])?;

	//mask.compressed_size = (lfh_i - mask.len()) as u32;
	//mask.uncompressed_size = (lfh_i - mask.len()) as u32;
	mask.write(&mut &mut output[0..mask.len()])?;
	(&mut &mut output[mask.len()..mask.len()+4]).write_u32::<LittleEndian>(lfh_i as u32)?;
	(&mut &mut output[mask.len()+4..mask.len()+8]).write_u32::<LittleEndian>((cdh2_i - lfh_i) as u32)?;

	if let Ok(mut outfile) = std::fs::File::create(&argv[2]) {
		outfile.write_all(&output[..])?;
	}

    Ok(())
}
