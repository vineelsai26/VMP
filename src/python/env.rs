use crate::python::list::list_python_versions;
use crate::utils::paths::vmp_dir;

pub async fn posix_env(env_type: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = "".to_string();

    if env_type == "all" {
        let versions = list_python_versions("installed".to_string()).await?;
        let python_root = vmp_dir()?.join("python");
        for version in versions {
            let bin = python_root.join(version).join("bin");
            path += &bin.to_string_lossy();
            path += ":";
        }
    }

    let mut env = r###"
export PATH="$HOME/.vmp/python/$(cat $HOME/.vmp/python_version)/bin:{PYTHON_PATHS}$PATH"

function vmp {
    $(whereis vmp | cut -d" " -f2) $@
    if [[ "$1" == "use" ]]
	then
        export PATH="$HOME/.vmp/python/$(cat $HOME/.vmp/python_version)/bin:$PATH"
	fi
}

function setPythonVersion {
	if [ -f .python-version ]
	then
		echo "Found .python-version file"
		if [ -d $HOME/.vmp/python/v$(ls "$HOME/.vmp/python" 2> /dev/null | grep "$(cat .python-version)" | tail -1 | cut -f2 -d"v")/bin ]
		then
			export PATH="$HOME/.vmp/python/v$(ls "$HOME/.vmp/python" 2> /dev/null | grep "$(cat .python-version)" | tail -1 | cut -f2 -d"v")/bin:$PATH"
		else
			vmp install $(cat .python-version)
			export PATH="$HOME/.vmp/python/v$(ls "$HOME/.vmp/python" 2> /dev/null | grep "$(cat .python-version)" | tail -1 | cut -f2 -d"v")/bin:$PATH"
		fi

		if [[ $(ls "$HOME/.vmp/python" 2> /dev/null | grep "$(cat .python-version)" | tail -1) != "" ]]; then
			echo "Using python version v$(ls "$HOME/.vmp/python" 2> /dev/null | grep "$(cat .python-version)" | tail -1 | cut -f2 -d"v")"
		fi
	fi
}

function cd {
	builtin cd "$@"
	setPythonVersion
}

setPythonVersion
"###.to_string();

    env = env.replace("{PYTHON_PATHS}", &path);

    println!("{}", env);

    Ok(())
}
