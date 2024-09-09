use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use tar::{Archive, Builder};

/// Compresses an entire folder into a .tar.gz archive.
///
/// This function takes a folder path and compresses its contents, including all subdirectories,
/// into a .tar.gz file at the specified tar_path. The resulting archive includes all files
/// and directories from the source folder, preserving the directory structure.
///
/// # Arguments
///
/// * `folder_path` - The path to the folder that should be compressed.
/// * `tar_path` - The path where the resulting .tar.gz file will be created.
///
/// # Returns
///
/// * `io::Result<()>` - An empty result if successful, or an I/O error if something goes wrong.
pub fn compress_folder_to_tar(folder_path: &str, tar_path: &str) -> io::Result<()> {
    let tar_gz = File::create(tar_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", folder_path)?;

    Ok(())
}

/// Compresses multiple files into a single .tar.gz archive.
///
/// This function takes a list of file paths and compresses them into a single .tar.gz
/// file at the specified combined_path. Each file is added to the archive under its
/// original file name, without any directory structure.
///
/// # Arguments
///
/// * `files_paths` - An array of strings representing the paths of the files to be compressed.
/// * `combined_path` - The path where the resulting .tar.gz file will be created.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - An empty result if successful, or an error if something goes wrong.
pub fn compress_files_to_tar(files_paths: &[String], combined_path: &str) -> Result<(), Box<dyn Error>> {
    let tar_gz = File::create(combined_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    for file_path in files_paths {
        let mut file = File::open(file_path)?;
        tar.append_file(Path::new(file_path).file_name().unwrap(), &mut file)?;
    }

    Ok(())
}

/// Decompresses a .tar.gz archive into a specified output directory.
///
/// This function takes the path of a .tar.gz file and decompresses its contents into
/// the specified output directory. The directory structure stored in the archive is
/// preserved during extraction.
///
/// # Arguments
///
/// * `tar_gz_path` - The path to the .tar.gz file that should be decompressed.
/// * `output_dir` - The directory where the archive's contents will be extracted.
///
/// # Returns
///
/// * `io::Result<()>` - An empty result if successful, or an I/O error if something goes wrong.
pub fn decompress_file_from_tar(tar_gz_path: &str, output_dir: &str) -> io::Result<()> {
    let tar_gz = File::open(tar_gz_path)?;
    let tar_gz_reader = BufReader::new(tar_gz);
    let tar = GzDecoder::new(tar_gz_reader);
    let mut archive = Archive::new(tar);

    archive.unpack(output_dir)?;

    Ok(())
}
