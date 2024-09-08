use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;

pub fn compress_folder_to_tar(folder_path: &str, tar_path: &str) -> std::io::Result<()> {
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