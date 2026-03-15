#compdef chronicle

autoload -U is-at-least

_chronicle() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_chronicle_commands" \
"*::: :->chronicle" \
&& ret=0
    case $state in
    (chronicle)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-command-$line[1]:"
        case $line[1] in
            (init)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--git-init[Create the Git repository if missing (\`git init\`)]' \
'-h[Print help]' \
'--help[Print help]' \
'::path -- Initialize at this path (defaults to current directory / auto-detect):_files' \
&& ret=0
;;
(remember)
_arguments "${_arguments_options[@]}" : \
'-m+[Memory message/body (Markdown supported)]:MSG:_default' \
'--msg=[Memory message/body (Markdown supported)]:MSG:_default' \
'--id=[Optional explicit id / filename stem (e.g. \`Rust\` creates \`Rust.md\`)]:ID:_default' \
'--layer=[Store in short-term or long-term layer]:LAYER:(short long archive)' \
'*-t+[Extra tags (repeatable)]:TAGS:_default' \
'*--tags=[Extra tags (repeatable)]:TAGS:_default' \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--no-commit[Skip Git commit (still writes the file)]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(recall)
_arguments "${_arguments_options[@]}" : \
'-q+[Search query]:QUERY:_default' \
'--query=[Search query]:QUERY:_default' \
'-k+[Number of results to return]:TOP_K:_default' \
'--top-k=[Number of results to return]:TOP_K:_default' \
'--top=[Number of results to return]:TOP_K:_default' \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--include-archive[Include archive results]' \
'--no-touch[Do not update access metadata (hit_count/last_access)]' \
'--json[Output JSON for agent consumption]' \
'--no-assoc[Skip associative expansion via \`\[\[links\]\]\`]' \
'--no-commit[Skip Git commit for access metadata updates]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(consolidate)
_arguments "${_arguments_options[@]}" : \
'--min-hits=[Consolidate if hit_count >= this value]:MIN_HITS:_default' \
'--min-age-hours=[Consolidate if age (hours) >= this value (0 disables)]:MIN_AGE_HOURS:_default' \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--dry-run[Preview actions without writing]' \
'--no-commit[Skip Git commit]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(forget)
_arguments "${_arguments_options[@]}" : \
'(--threshold)--id=[Delete a specific memory by id]:ID:_default' \
'(--id)--threshold=[Archive long-term memories whose ACT-R heat is below this threshold (0..1 recommended)]:THRESHOLD:_default' \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--dry-run[Preview actions without writing]' \
'--no-commit[Skip Git commit]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(log)
_arguments "${_arguments_options[@]}" : \
'--limit=[Maximum number of commits to show]:LIMIT:_default' \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(branch)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
":: :_chronicle__branch_commands" \
"*::: :->branch" \
&& ret=0

    case $state in
    (branch)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-branch-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
':name:_default' \
&& ret=0
;;
(checkout)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
':name:_default' \
&& ret=0
;;
(merge)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
':name:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(current)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'--force[]' \
'-h[Print help]' \
'--help[Print help]' \
':name:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_chronicle__branch__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-branch-help-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(checkout)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(merge)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(current)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
':shell:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(wal)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
":: :_chronicle__wal_commands" \
"*::: :->wal" \
&& ret=0

    case $state in
    (wal)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-wal-command-$line[1]:"
        case $line[1] in
            (run)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_chronicle__wal__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-wal-help-command-$line[1]:"
        case $line[1] in
            (run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(status)
_arguments "${_arguments_options[@]}" : \
'--root=[Project root (defaults to auto-detect from current directory)]:ROOT:_files' \
'--commit=[Git commit mode after write/touch operations]:COMMIT:(sync async off)' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_chronicle__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-help-command-$line[1]:"
        case $line[1] in
            (init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remember)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(recall)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(consolidate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(forget)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(log)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(branch)
_arguments "${_arguments_options[@]}" : \
":: :_chronicle__help__branch_commands" \
"*::: :->branch" \
&& ret=0

    case $state in
    (branch)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-help-branch-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(checkout)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(merge)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(current)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(wal)
_arguments "${_arguments_options[@]}" : \
":: :_chronicle__help__wal_commands" \
"*::: :->wal" \
&& ret=0

    case $state in
    (wal)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:chronicle-help-wal-command-$line[1]:"
        case $line[1] in
            (run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_chronicle_commands] )) ||
_chronicle_commands() {
    local commands; commands=(
'init:Initialize \`.chronicle\` directory structure in the project' \
'remember:Store a new memory block (and commit it to Git)' \
'recall:Recall memory blocks using the MAMA matching algorithm' \
'consolidate:Move eligible short-term memories into long-term storage' \
'forget:Apply forgetting\: move low-strength memories to archive or delete' \
'log:Show memory change history (wraps \`git log\`)' \
'branch:Git branch helpers for sandboxed reasoning' \
'completions:Print shell completion script' \
'wal:Internal\: process WAL tasks' \
'status:Print project status summary' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle commands' commands "$@"
}
(( $+functions[_chronicle__branch_commands] )) ||
_chronicle__branch_commands() {
    local commands; commands=(
'create:Create and checkout a new branch' \
'checkout:Checkout an existing branch' \
'merge:Merge a branch into the current branch' \
'list:List local branches' \
'current:Print current branch name' \
'delete:Delete a local branch' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle branch commands' commands "$@"
}
(( $+functions[_chronicle__branch__checkout_commands] )) ||
_chronicle__branch__checkout_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch checkout commands' commands "$@"
}
(( $+functions[_chronicle__branch__create_commands] )) ||
_chronicle__branch__create_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch create commands' commands "$@"
}
(( $+functions[_chronicle__branch__current_commands] )) ||
_chronicle__branch__current_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch current commands' commands "$@"
}
(( $+functions[_chronicle__branch__delete_commands] )) ||
_chronicle__branch__delete_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch delete commands' commands "$@"
}
(( $+functions[_chronicle__branch__help_commands] )) ||
_chronicle__branch__help_commands() {
    local commands; commands=(
'create:Create and checkout a new branch' \
'checkout:Checkout an existing branch' \
'merge:Merge a branch into the current branch' \
'list:List local branches' \
'current:Print current branch name' \
'delete:Delete a local branch' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle branch help commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__checkout_commands] )) ||
_chronicle__branch__help__checkout_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help checkout commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__create_commands] )) ||
_chronicle__branch__help__create_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help create commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__current_commands] )) ||
_chronicle__branch__help__current_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help current commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__delete_commands] )) ||
_chronicle__branch__help__delete_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help delete commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__help_commands] )) ||
_chronicle__branch__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help help commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__list_commands] )) ||
_chronicle__branch__help__list_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help list commands' commands "$@"
}
(( $+functions[_chronicle__branch__help__merge_commands] )) ||
_chronicle__branch__help__merge_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch help merge commands' commands "$@"
}
(( $+functions[_chronicle__branch__list_commands] )) ||
_chronicle__branch__list_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch list commands' commands "$@"
}
(( $+functions[_chronicle__branch__merge_commands] )) ||
_chronicle__branch__merge_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle branch merge commands' commands "$@"
}
(( $+functions[_chronicle__completions_commands] )) ||
_chronicle__completions_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle completions commands' commands "$@"
}
(( $+functions[_chronicle__consolidate_commands] )) ||
_chronicle__consolidate_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle consolidate commands' commands "$@"
}
(( $+functions[_chronicle__forget_commands] )) ||
_chronicle__forget_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle forget commands' commands "$@"
}
(( $+functions[_chronicle__help_commands] )) ||
_chronicle__help_commands() {
    local commands; commands=(
'init:Initialize \`.chronicle\` directory structure in the project' \
'remember:Store a new memory block (and commit it to Git)' \
'recall:Recall memory blocks using the MAMA matching algorithm' \
'consolidate:Move eligible short-term memories into long-term storage' \
'forget:Apply forgetting\: move low-strength memories to archive or delete' \
'log:Show memory change history (wraps \`git log\`)' \
'branch:Git branch helpers for sandboxed reasoning' \
'completions:Print shell completion script' \
'wal:Internal\: process WAL tasks' \
'status:Print project status summary' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle help commands' commands "$@"
}
(( $+functions[_chronicle__help__branch_commands] )) ||
_chronicle__help__branch_commands() {
    local commands; commands=(
'create:Create and checkout a new branch' \
'checkout:Checkout an existing branch' \
'merge:Merge a branch into the current branch' \
'list:List local branches' \
'current:Print current branch name' \
'delete:Delete a local branch' \
    )
    _describe -t commands 'chronicle help branch commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__checkout_commands] )) ||
_chronicle__help__branch__checkout_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch checkout commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__create_commands] )) ||
_chronicle__help__branch__create_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch create commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__current_commands] )) ||
_chronicle__help__branch__current_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch current commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__delete_commands] )) ||
_chronicle__help__branch__delete_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch delete commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__list_commands] )) ||
_chronicle__help__branch__list_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch list commands' commands "$@"
}
(( $+functions[_chronicle__help__branch__merge_commands] )) ||
_chronicle__help__branch__merge_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help branch merge commands' commands "$@"
}
(( $+functions[_chronicle__help__completions_commands] )) ||
_chronicle__help__completions_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help completions commands' commands "$@"
}
(( $+functions[_chronicle__help__consolidate_commands] )) ||
_chronicle__help__consolidate_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help consolidate commands' commands "$@"
}
(( $+functions[_chronicle__help__forget_commands] )) ||
_chronicle__help__forget_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help forget commands' commands "$@"
}
(( $+functions[_chronicle__help__help_commands] )) ||
_chronicle__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help help commands' commands "$@"
}
(( $+functions[_chronicle__help__init_commands] )) ||
_chronicle__help__init_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help init commands' commands "$@"
}
(( $+functions[_chronicle__help__log_commands] )) ||
_chronicle__help__log_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help log commands' commands "$@"
}
(( $+functions[_chronicle__help__recall_commands] )) ||
_chronicle__help__recall_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help recall commands' commands "$@"
}
(( $+functions[_chronicle__help__remember_commands] )) ||
_chronicle__help__remember_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help remember commands' commands "$@"
}
(( $+functions[_chronicle__help__status_commands] )) ||
_chronicle__help__status_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help status commands' commands "$@"
}
(( $+functions[_chronicle__help__wal_commands] )) ||
_chronicle__help__wal_commands() {
    local commands; commands=(
'run:' \
    )
    _describe -t commands 'chronicle help wal commands' commands "$@"
}
(( $+functions[_chronicle__help__wal__run_commands] )) ||
_chronicle__help__wal__run_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle help wal run commands' commands "$@"
}
(( $+functions[_chronicle__init_commands] )) ||
_chronicle__init_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle init commands' commands "$@"
}
(( $+functions[_chronicle__log_commands] )) ||
_chronicle__log_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle log commands' commands "$@"
}
(( $+functions[_chronicle__recall_commands] )) ||
_chronicle__recall_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle recall commands' commands "$@"
}
(( $+functions[_chronicle__remember_commands] )) ||
_chronicle__remember_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle remember commands' commands "$@"
}
(( $+functions[_chronicle__status_commands] )) ||
_chronicle__status_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle status commands' commands "$@"
}
(( $+functions[_chronicle__wal_commands] )) ||
_chronicle__wal_commands() {
    local commands; commands=(
'run:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle wal commands' commands "$@"
}
(( $+functions[_chronicle__wal__help_commands] )) ||
_chronicle__wal__help_commands() {
    local commands; commands=(
'run:' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'chronicle wal help commands' commands "$@"
}
(( $+functions[_chronicle__wal__help__help_commands] )) ||
_chronicle__wal__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle wal help help commands' commands "$@"
}
(( $+functions[_chronicle__wal__help__run_commands] )) ||
_chronicle__wal__help__run_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle wal help run commands' commands "$@"
}
(( $+functions[_chronicle__wal__run_commands] )) ||
_chronicle__wal__run_commands() {
    local commands; commands=()
    _describe -t commands 'chronicle wal run commands' commands "$@"
}

if [ "$funcstack[1]" = "_chronicle" ]; then
    _chronicle "$@"
else
    compdef _chronicle chronicle
fi
