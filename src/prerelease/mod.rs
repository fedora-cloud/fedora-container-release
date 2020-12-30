mod bodhi;
mod koji;

use askama::Template;
use std::fs::{read_dir, remove_dir_all, remove_file, File};
use std::io::{Write, Result};
use std::process::Command;

#[derive(Template)]
#[template(path = "Containerfile")]
struct ContainerfileTemplate<'a> {
    tag: &'a str,
    result_tar: &'a str,
}

struct Archive<'a> {
    filename: &'a str,
    arch: &'a str,
    version: String
}


fn download_archive<'a>(url: &'a str) -> Archive<'a> {
    let url_elements: Vec<&str> = url.split('/').collect();
    let filename = url_elements.last().unwrap();
    let filedata: Vec<&str> = filename.trim_end_matches(".tar.xz").split('.').collect();
    let arch = filedata.last().unwrap();
    let version = filedata[0]
        .trim_start_matches("Fedora-Container-Base-")
        .replace("-", ".");

    let mut file = File::create(format!("{}", filename)).unwrap();
    println!("Downloading {}", url);
    reqwest::blocking::get(url)
        .unwrap()
        .copy_to(&mut file)
        .unwrap();

    return Archive{
        filename: filename,
        arch: arch,
        version: version
    };
}

pub fn prepare_containerfiles(release: String) -> Result<()> {
    let rawhide = bodhi::get_rawhide_version().unwrap();
    let urls = koji::get_koji_archive_url(&release, &rawhide, false);
    for url in urls {
        let archive = download_archive(&url);
        println!("Decompress the archive {}", archive.filename);
        Command::new("tar")
            .arg(format!("--one-top-level={}", archive.arch))
            .arg("-xf")
            .arg(format!("{}", archive.filename))
            .output()
            .expect("failed to decompress the archive");

        let result_tar = format!("fedora-{}-{}.tar", archive.version, archive.arch);
        for entry in read_dir(format!("{}", archive.arch))? {
            let file = entry.unwrap();
            if file.file_type().unwrap().is_dir() {
                let hash = file.file_name().into_string().unwrap();
                println!("Rename the rootfs : {}", result_tar);
                Command::new("mv")
                    .arg(format!("{}/{}/layer.tar", archive.arch, hash))
                    .arg(format!("{}/{}", archive.arch, result_tar))
                    .output()
                    .expect("failed to move the layer.tar archive");
                remove_dir_all(file.path())?;
            } else if !file.file_name().into_string().unwrap().contains(".tar") {
                remove_file(file.path())?;
            }
        }

        println!("Compress the rootfs");
        Command::new("xz")
            .arg("--best")
            .arg("-T")
            .arg("0")
            .arg(format!("{}/{}", archive.arch, result_tar))
            .output()
            .expect("failed to compress the rootfs");

        let dockerfile = ContainerfileTemplate {
            tag: &release,
            result_tar: &format!("{}.xz", result_tar),
        };
        let mut buffer = File::create(format!("{}/Dockerfile", archive.arch))?;
        buffer.write_all(dockerfile.render().unwrap().as_bytes())?;
    }
    Ok(())
}
