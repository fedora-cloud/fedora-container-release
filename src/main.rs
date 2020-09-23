mod koji;

use askama::Template;
use std::fs::{read_dir, remove_dir_all, remove_file, File};
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(Template)]
#[template(path = "Containerfile")]
struct ContainerfileTemplate<'a> {
    tag: &'a str,
    result_tar: &'a str,
}

#[derive(Debug, StructOpt)]
struct CliOptions {
    #[structopt(short, long)]
    release: String,
}

fn main() -> std::io::Result<()> {
    let cliopt = CliOptions::from_args();
    let urls = koji::get_koji_archive_url(&cliopt.release, false);

    for url in urls {
        let url_elements: Vec<&str> = url.split('/').collect();
        let filename = url_elements.last().unwrap();
        let filedata: Vec<&str> = filename.trim_end_matches(".tar.xz").split('.').collect();
        let arch = filedata[2];
        let version = filedata[0]
            .trim_start_matches("Fedora-Container-Base-")
            .replace("-", ".");

        let mut file = File::create(format!("{}", filename))?;
        println!("Downloading {}", filename);
        reqwest::blocking::get(&url)
            .unwrap()
            .copy_to(&mut file)
            .unwrap();

        println!("Decompress the archive");
        Command::new("tar")
            .arg(format!("--one-top-level={}", arch))
            .arg("-xf")
            .arg(format!("{}", filename))
            .output()
            .expect("failed to decompress the archive");

        let result_tar = format!("fedora-{}-{}.tar", version, arch);
        for entry in read_dir(format!("{}", arch))? {
            let file = entry.unwrap();
            if file.file_type().unwrap().is_dir() {
                let hash = file.file_name().into_string().unwrap();
                println!("Rename the rootfs : {}", result_tar);
                Command::new("mv")
                    .arg(format!("{}/{}/layer.tar", arch, hash))
                    .arg(format!("{}/{}", arch, result_tar))
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
            .arg(format!("{}/{}", arch, result_tar))
            .output()
            .expect("failed to compress the rootfs");

        let dockerfile = ContainerfileTemplate {
            tag: &cliopt.release,
            result_tar: &format!("{}.xz", result_tar),
        };
        let mut buffer = File::create(format!("{}/Dockerfile", arch))?;
        buffer.write_all(dockerfile.render().unwrap().as_bytes())?;
    }
    Ok(())
}
