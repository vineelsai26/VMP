use std::fs;

use super::versions::fetch_available_versions;
use homedir::get_my_home;

pub async fn list_python_versions(list_type: String) -> Result<(), Box<dyn std::error::Error>> {
    if list_type == "all" {
        let versions = fetch_available_versions().await?;
        for version in versions {
            println!("{}", version);
        }
    } else if list_type == "installed" {
        let python_install_path = get_my_home()
            .unwrap()
            .unwrap()
            .as_path()
            .join(".vmp")
            .join("python");

        let versions = fs::read_dir(python_install_path.as_path()).unwrap();
        for version in versions {
            println!(
                "{}",
                version.unwrap().file_name().to_str().unwrap().to_string()
            );
        }
    } else {
        println!("unknown option {}, please use `all` or `installed` instead", list_type)
    }

    Ok(())
}
