use std::{
    fs::{self, read_dir, File, OpenOptions},
    io::{self, BufReader, BufWriter, Cursor, Read, Write},
    path::Path,
};

/// 将文件存储到zip文件
#[allow(dead_code)]
fn app_to_zip(zip_filename: &str) {
    let zip_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(zip_filename)
        .unwrap();
    let mut zip = zip::ZipWriter::new(zip_file);

    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);

    for dir_entry in read_dir("/mnt/g/桌面/acd").unwrap() {
        let entry = dir_entry.unwrap().path();
        let filename = entry.file_name().unwrap().to_str().unwrap();
        if entry.extension().unwrap() == "csv" {
            zip.start_file(filename, options).unwrap();
            let data = std::fs::read(entry).unwrap();
            let _ = zip.write(&data).unwrap();
        }
    }
    zip.finish().unwrap();
}

fn main() {
    // let start = time::Instant::now();
    // // extract_from_zip("/mnt/g/桌面/1");
    // extract_from_zip1("/mnt/g/桌面/1");
    // println!("time: {}", start.elapsed().as_millis());

    app_to_zip("/mnt/g/桌面/ks.zip");
}

#[allow(dead_code)]
fn extract_from_zip(out_dir: &str) {
    let zip_file_path = "/mnt/g/桌面/1INCHUSDT-1m-2021-12-20.zip";
    let zip_file_reader = BufReader::new(File::open(zip_file_path).unwrap());
    let mut zip = zip::ZipArchive::new(zip_file_reader).unwrap();
    let out_dir = Path::new(out_dir);
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let out_file = out_dir.join(file.name());
        let mut zip_writer = BufWriter::new(File::create(out_file).unwrap());
        io::copy(&mut file, &mut zip_writer).unwrap();
    }
}

#[allow(dead_code)]
fn extract_from_zip1(out_dir: &str) {
    let mut zip_content = vec![];
    let mut f = fs::File::open("/mnt/g/桌面/1INCHUSDT-1m-2021-12-20.zip").unwrap();
    f.read_to_end(&mut zip_content).unwrap();

    let zip_reader = Cursor::new(zip_content);
    let mut zip = zip::ZipArchive::new(zip_reader).unwrap();

    let out_dir = Path::new(out_dir);
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let out_file = out_dir.join(file.name());
        let mut zip_writer = BufWriter::new(File::create(out_file).unwrap());
        io::copy(&mut file, &mut zip_writer).unwrap();
    }
}
