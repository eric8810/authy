use crate::error::{AuthyError, Result};

pub fn run(shell: &str) -> Result<()> {
    let output = match shell {
        "bash" => generate_bash(),
        "zsh" => generate_zsh(),
        "fish" => generate_fish(),
        other => {
            return Err(AuthyError::Other(format!(
                "Unsupported shell '{}'. Use bash, zsh, or fish.",
                other
            )));
        }
    };
    print!("{}", output);
    Ok(())
}

fn generate_bash() -> String {
    r#"# authy shell hook — eval "$(authy hook bash)"

_authy_find_config() {
  local dir="$1"
  while [ "$dir" != "/" ]; do
    if [ -f "$dir/.authy.toml" ]; then
      echo "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  if [ -f "/.authy.toml" ]; then
    echo "/"
    return 0
  fi
  return 1
}

_authy_hook() {
  local project_dir
  project_dir="$(_authy_find_config "$PWD")"

  if [ -n "$project_dir" ]; then
    # Entering or staying in a project
    if [ "$project_dir" != "${AUTHY_PROJECT_DIR:-}" ]; then
      # New project — clean up old one first
      if [ -n "${AUTHY_PROJECT_DIR:-}" ]; then
        eval "$(authy alias --cleanup --shell bash)"
        unset AUTHY_PROJECT_DIR AUTHY_KEYFILE
        echo "authy: unloading ${AUTHY_PROJECT_DIR##*/}" >&2
      fi

      export AUTHY_PROJECT_DIR="$project_dir"

      # Set keyfile if configured
      local keyfile
      keyfile="$(authy project-info --field keyfile --dir "$project_dir" 2>/dev/null)"
      if [ -n "$keyfile" ]; then
        export AUTHY_KEYFILE="$keyfile"
      fi

      # Load aliases
      eval "$(authy alias --from-project --shell bash)"

      echo "authy: loading ${project_dir##*/}/.authy.toml" >&2
    fi
  else
    # Left all projects
    if [ -n "${AUTHY_PROJECT_DIR:-}" ]; then
      eval "$(authy alias --cleanup --shell bash)"
      echo "authy: unloading ${AUTHY_PROJECT_DIR##*/}" >&2
      unset AUTHY_PROJECT_DIR AUTHY_KEYFILE
    fi
  fi
}

cd() { builtin cd "$@" && _authy_hook; }
pushd() { builtin pushd "$@" && _authy_hook; }
popd() { builtin popd "$@" && _authy_hook; }

# Trigger on shell start
_authy_hook
"#
    .to_string()
}

fn generate_zsh() -> String {
    r#"# authy shell hook — eval "$(authy hook zsh)"

_authy_find_config() {
  local dir="$1"
  while [ "$dir" != "/" ]; do
    if [ -f "$dir/.authy.toml" ]; then
      echo "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  if [ -f "/.authy.toml" ]; then
    echo "/"
    return 0
  fi
  return 1
}

_authy_hook() {
  local project_dir
  project_dir="$(_authy_find_config "$PWD")"

  if [ -n "$project_dir" ]; then
    if [ "$project_dir" != "${AUTHY_PROJECT_DIR:-}" ]; then
      if [ -n "${AUTHY_PROJECT_DIR:-}" ]; then
        eval "$(authy alias --cleanup --shell zsh)"
        unset AUTHY_PROJECT_DIR AUTHY_KEYFILE
        echo "authy: unloading ${AUTHY_PROJECT_DIR##*/}" >&2
      fi

      export AUTHY_PROJECT_DIR="$project_dir"

      local keyfile
      keyfile="$(authy project-info --field keyfile --dir "$project_dir" 2>/dev/null)"
      if [ -n "$keyfile" ]; then
        export AUTHY_KEYFILE="$keyfile"
      fi

      eval "$(authy alias --from-project --shell zsh)"

      echo "authy: loading ${project_dir##*/}/.authy.toml" >&2
    fi
  else
    if [ -n "${AUTHY_PROJECT_DIR:-}" ]; then
      eval "$(authy alias --cleanup --shell zsh)"
      echo "authy: unloading ${AUTHY_PROJECT_DIR##*/}" >&2
      unset AUTHY_PROJECT_DIR AUTHY_KEYFILE
    fi
  fi
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd _authy_hook

# Trigger on shell start
_authy_hook
"#
    .to_string()
}

fn generate_fish() -> String {
    r#"# authy shell hook — authy hook fish | source

function _authy_find_config
    set -l dir $argv[1]
    while test "$dir" != "/"
        if test -f "$dir/.authy.toml"
            echo $dir
            return 0
        end
        set dir (dirname $dir)
    end
    if test -f "/.authy.toml"
        echo "/"
        return 0
    end
    return 1
end

function _authy_hook --on-variable PWD
    set -l project_dir (_authy_find_config $PWD)

    if test -n "$project_dir"
        if test "$project_dir" != "$AUTHY_PROJECT_DIR"
            if set -q AUTHY_PROJECT_DIR; and test -n "$AUTHY_PROJECT_DIR"
                eval (authy alias --cleanup --shell fish)
                echo "authy: unloading "(basename $AUTHY_PROJECT_DIR) >&2
                set -e AUTHY_PROJECT_DIR
                set -e AUTHY_KEYFILE
            end

            set -gx AUTHY_PROJECT_DIR $project_dir

            set -l keyfile (authy project-info --field keyfile --dir $project_dir 2>/dev/null)
            if test -n "$keyfile"
                set -gx AUTHY_KEYFILE $keyfile
            end

            eval (authy alias --from-project --shell fish)

            echo "authy: loading "(basename $project_dir)"/.authy.toml" >&2
        end
    else
        if set -q AUTHY_PROJECT_DIR; and test -n "$AUTHY_PROJECT_DIR"
            eval (authy alias --cleanup --shell fish)
            echo "authy: unloading "(basename $AUTHY_PROJECT_DIR) >&2
            set -e AUTHY_PROJECT_DIR
            set -e AUTHY_KEYFILE
        end
    end
end

# Trigger on shell start
_authy_hook
"#
    .to_string()
}
