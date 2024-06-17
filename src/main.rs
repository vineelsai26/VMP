mod python;
mod utils;

use python::{
    env::posix_env, install::install_python, list::list_python_versions, usage::use_python,
};
use std::process::exit;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

fn help() {
    println!("Usage: vmp <command> [version]
Commands:
	install [version]        			Install a specific version of python or latest or lts version of python (default: lts)
	use [version]            			Use a specific version of python
	list [type]              			List all versions, installed versions or lts versions
	uninstall [version]      			Uninstall a specific version of python
	help                     			Print this help section
Examples:
	vmp install latest       			Install the latest version of python
	vmp use latest           			Use the latest version of python
	vmp install 3.11         			Install a specific version of python
	vmp use 3.11		          		Use a specific version of python
	vmp list all             			List all versions of python
	vmp list installed       			List installed versions of python
	vmp uninstall all        			Uninstall all versions of python
	vmp help                 			Print this help");
    exit(0);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = std::env::args().nth(1).expect("Not enough args");
    let version = std::env::args().nth(2);

    if cmd == "help" {
        help()
    } else if cmd == "version" {
        println!("{}", VERSION.unwrap_or("unknown"));
        exit(0);
    } else if cmd == "env" {
        posix_env();
        exit(0)
    } else if cmd == "list" && version.is_none() {
        list_python_versions("".to_string()).await?;
    }

    if version.is_none() {
        exit(0)
    }

    if cmd == "install" {
        install_python(version.unwrap()).await?;
    } else if cmd == "use" {
        use_python(version.unwrap()).await?;
    } else if cmd == "list" {
        list_python_versions(version.unwrap()).await?;
    }

    Ok(())
}
