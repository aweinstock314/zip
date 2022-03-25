use std::{
    fs::File,
    io::{Cursor, Write},
};
use zip::{
    read::ZipArchive,
    result::ZipResult,
    write::{FileOptions, ZipWriter},
};

#[test]
fn test_eocd_searchlength() -> ZipResult<()> {
    let mut bytes = Vec::new();
    {
        let mut cursor = Cursor::new(&mut bytes);
        let mut writer = ZipWriter::new(&mut cursor);
        writer.start_file("test", FileOptions::default())?;
        writer.write(b"test\n")?;
        writer.set_raw_comment(vec![0; u16::max_value() as usize]);
        writer.finish()?;
        drop(writer);
        cursor.write(b"junk")?;
    }

    // This file is successfully extracted by unzip(1)
    let mut file = File::create("issue_183.zip")?;
    file.write(&bytes[..])?;

    {
        let cursor = Cursor::new(&bytes);
        let archive = ZipArchive::new(cursor);
        assert!(archive.is_ok());
    }

    Ok(())
}
