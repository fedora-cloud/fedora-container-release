use xmlrpc::{Request, Value};

const PKG_NAME: &str = "Fedora-Container-Base";
const PKG_NAME_MINI: &str = "Fedora-Container-Minimal-Base";
const KOJI_HUB: &str = "https://koji.fedoraproject.org/kojihub/";

pub fn get_koji_archive_url(release: &str, minimal: bool) -> Vec<String> {
    let pkg_name: &str;
    match minimal {
        true => pkg_name = PKG_NAME_MINI,
        false => pkg_name = PKG_NAME,
    };
    let list_tagged_req = Request::new("listTagged")
        .arg(format!("f{}-updates-candidate", release))
        .arg(Value::Nil)
        .arg(false)
        .arg(Value::Nil)
        .arg(true)
        .arg(pkg_name)
        .arg(Value::Nil)
        .arg("image");

    let mut result = list_tagged_req.call_url(KOJI_HUB);
    let build_data = result.unwrap();
    let build_id = build_data[0]["build_id"].as_i32().unwrap();
    let build_release = build_data[0]["release"].as_str().unwrap();

    let list_archives_req = Request::new("listArchives")
        .arg(build_id)
        .arg(Value::Nil)
        .arg(Value::Nil)
        .arg(Value::Nil)
        .arg("image");
    result = list_archives_req.call_url(KOJI_HUB);
    let archive_data = result.unwrap();
    let images = archive_data.as_array().unwrap();

    let mut urls = Vec::new();
    for i in images {
        let filename = i["filename"].as_str().unwrap();
        if filename.contains(".tar.xz") {
            let url = format!(
                "https://kojipkgs.fedoraproject.org/packages/{}/{}/{}/images/{}",
                pkg_name, release, build_release, filename
            );
            urls.push(url)
        }
    }
    return urls;
}
