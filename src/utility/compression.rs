use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::{fs, io};
use tar::{Archive, Builder};

pub fn compress_folder_to_tar(folder_path: &str, tar_path: &str) -> io::Result<()> {
    let tar_gz = File::create(tar_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", folder_path)?;

    Ok(())
}

pub fn compress_files_to_tar(files_paths: &[String], combined_path: &str) -> Result<(), Box<dyn Error>> {
    let tar_gz = File::create(combined_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    for file_path in files_paths {
        let mut file = File::open(file_path)?;
        tar.append_file(Path::new(file_path).file_name().unwrap(), &mut file)?;
        fs::remove_file(file_path)?;
    }

    Ok(())
}

pub fn decompress_file_from_tar(tar_gz_path: &str, output_dir: &str) -> io::Result<()> {
    let tar_gz = File::open(tar_gz_path)?;
    let tar_gz_reader = BufReader::new(tar_gz);
    let tar = GzDecoder::new(tar_gz_reader);
    let mut archive = Archive::new(tar);

    archive.unpack(output_dir)?;

    Ok(())
}