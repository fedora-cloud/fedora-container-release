use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BodhiReleases {
    releases: Vec<BodhiRelease>,
}

#[derive(Debug, Deserialize)]
struct BodhiRelease {
    version: String,
}

pub fn get_rawhide_version() -> Result<i32, Box<dyn std::error::Error>> {
    let resp: BodhiReleases =
        reqwest::blocking::get("https://bodhi.fedoraproject.org/releases/?state=pending")?
            .json()?;
    let mut versions = Vec::new();
    for rel in resp.releases {
        match rel.version.parse::<i32>() {
            Ok(v) => versions.push(v),
            Err(_e) => (),
        }
    }
    let rawhide = versions.iter().max_by(|x, y| x.cmp(y)).unwrap();
    Ok(*rawhide)
}
