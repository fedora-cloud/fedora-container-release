mod bodhi;
mod koji;

use askama::Template;
use std::fs::{read_dir, remove_dir_all, remove_file, File};
use std::io::{Result, Write};
use std::path::Path;
use std::process::Command;
use std::thread;

#[derive(Template)]
#[template(path = "Containerfile")]
struct ContainerfileTemplate<'a> {
    tag: &'a str,
    result_tar: &'a str,
}

struct Archive {
    url: String,
    filename: String,
    arch: String,
    version: String,
    tarfile: String,
}

impl Archive {
    fn download(&self) {
        println!("Downloading {}", self.url);
        let mut file = File::create(format!("{}", self.filename)).unwrap();
        reqwest::blocking::get(&self.url)
            .unwrap()
            .copy_to(&mut file)
            .unwrap();
    }

    fn decompress(&self) {
        println!("Decompress the archive {}", self.filename);
        Command::new("tar")
            .arg(format!("--one-top-level={}", self.arch))
            .arg("-xf")
            .arg(&self.filename)
            .output()
            .expect("failed to decompress the archive");
    }

    fn create_rootfs(&self) {
        /* Search for the layer.tar rootfs from the uncompress
         koji archive. Once found rename it with the architecture and
         version.
         Remove all the other files and directories.
         Then compress the rootfs using xz.
        */
        for entry in read_dir(&self.arch).unwrap() {
            let file = entry.unwrap();
            if file.file_type().unwrap().is_dir() {
                let hash = file.file_name().into_string().unwrap();
                Command::new("mv")
                    .arg(format!("{}/{}/layer.tar", self.arch, hash))
                    .arg(format!("{}/{}", self.arch, self.tarfile))
                    .output()
                    .expect("failed to move the layer.tar archive");
                remove_dir_all(file.path()).unwrap();
            } else if !file.file_name().into_string().unwrap().contains(".tar") {
                remove_file(file.path()).unwrap();
            }
        }
        println!("Compress the rootfs");
        Command::new("xz")
            .arg("--best")
            .arg("-T")
            .arg("0")
            .arg(format!("{}/{}", self.arch, self.tarfile))
            .output()
            .expect("failed to compress the rootfs");
    }
}

pub fn prepare_containerfiles(release: String) -> Result<()> {
    let rawhide = bodhi::get_rawhide_version().unwrap();
    // get the archives' url from koji
    let urls = koji::get_koji_archive_url(&release, &rawhide, false);

    // prepare a vector to store the threads.
    let mut children = vec![];

    for url in urls {
        // for each url create a thread.
        children.push(thread::spawn(move || {
            let url_elements: Vec<&str> = url.split('/').collect();
            let filename = url_elements.last().unwrap().to_string();
            let filedata: Vec<&str> = filename.trim_end_matches(".tar.xz").split('.').collect();
            let arch = filedata.last().unwrap().to_string();
            let version = filedata[0]
                .trim_start_matches("Fedora-Container-Base-")
                .replace("-", ".");
            let tarfile = format!("fedora-{}-{}.tar", version, arch);

            let archive = Archive {
                url: url,
                filename: filename,
                arch: arch,
                version: version,
                tarfile: tarfile,
            };

            // if the archive is already on the disk, don't download it
            if !Path::new(&format!("./{}", archive.filename)).is_file() {
                archive.download();
            }
            archive.decompress();
            archive.create_rootfs();

            let dockerfile = ContainerfileTemplate {
                tag: &archive
                    .version
                    .split(".")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap(),
                result_tar: &format!("{}.xz", archive.tarfile),
            };
            let mut buffer = File::create(format!("{}/Dockerfile", archive.arch)).unwrap();
            buffer
                .write_all(dockerfile.render().unwrap().as_bytes())
                .unwrap();
        }))
    }

    // for each thread wait for it to finish
    for child in children {
        let _ = child.join();
    }
    Ok(())
}
