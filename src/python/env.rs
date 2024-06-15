pub fn posix_env() {
    let env = r###"
export PATH="$HOME/.vmp/python/$(cat $HOME/.vmp/python_version)/bin:$PATH"

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
			vmp --compile python install $(cat .python-version)
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
"###;

    println!("{}", env);
}
