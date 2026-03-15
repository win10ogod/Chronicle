
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'chronicle' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'chronicle'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'chronicle' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize `.chronicle` directory structure in the project')
            [CompletionResult]::new('remember', 'remember', [CompletionResultType]::ParameterValue, 'Store a new memory block (and commit it to Git)')
            [CompletionResult]::new('recall', 'recall', [CompletionResultType]::ParameterValue, 'Recall memory blocks using the MAMA matching algorithm')
            [CompletionResult]::new('consolidate', 'consolidate', [CompletionResultType]::ParameterValue, 'Move eligible short-term memories into long-term storage')
            [CompletionResult]::new('forget', 'forget', [CompletionResultType]::ParameterValue, 'Apply forgetting: move low-strength memories to archive or delete')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show memory change history (wraps `git log`)')
            [CompletionResult]::new('branch', 'branch', [CompletionResultType]::ParameterValue, 'Git branch helpers for sandboxed reasoning')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Print shell completion script')
            [CompletionResult]::new('wal', 'wal', [CompletionResultType]::ParameterValue, 'Internal: process WAL tasks')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Print project status summary')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;init' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--git-init', '--git-init', [CompletionResultType]::ParameterName, 'Create the Git repository if missing (`git init`)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;remember' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Memory message/body (Markdown supported)')
            [CompletionResult]::new('--msg', '--msg', [CompletionResultType]::ParameterName, 'Memory message/body (Markdown supported)')
            [CompletionResult]::new('--id', '--id', [CompletionResultType]::ParameterName, 'Optional explicit id / filename stem (e.g. `Rust` creates `Rust.md`)')
            [CompletionResult]::new('--layer', '--layer', [CompletionResultType]::ParameterName, 'Store in short-term or long-term layer')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Extra tags (repeatable)')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Extra tags (repeatable)')
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--no-commit', '--no-commit', [CompletionResultType]::ParameterName, 'Skip Git commit (still writes the file)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;recall' {
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Search query')
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Search query')
            [CompletionResult]::new('-k', '-k', [CompletionResultType]::ParameterName, 'Number of results to return')
            [CompletionResult]::new('--top-k', '--top-k', [CompletionResultType]::ParameterName, 'Number of results to return')
            [CompletionResult]::new('--top', '--top', [CompletionResultType]::ParameterName, 'Number of results to return')
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--include-archive', '--include-archive', [CompletionResultType]::ParameterName, 'Include archive results')
            [CompletionResult]::new('--no-touch', '--no-touch', [CompletionResultType]::ParameterName, 'Do not update access metadata (hit_count/last_access)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON for agent consumption')
            [CompletionResult]::new('--no-assoc', '--no-assoc', [CompletionResultType]::ParameterName, 'Skip associative expansion via `[[links]]`')
            [CompletionResult]::new('--no-commit', '--no-commit', [CompletionResultType]::ParameterName, 'Skip Git commit for access metadata updates')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;consolidate' {
            [CompletionResult]::new('--min-hits', '--min-hits', [CompletionResultType]::ParameterName, 'Consolidate if hit_count >= this value')
            [CompletionResult]::new('--min-age-hours', '--min-age-hours', [CompletionResultType]::ParameterName, 'Consolidate if age (hours) >= this value (0 disables)')
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Preview actions without writing')
            [CompletionResult]::new('--no-commit', '--no-commit', [CompletionResultType]::ParameterName, 'Skip Git commit')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;forget' {
            [CompletionResult]::new('--id', '--id', [CompletionResultType]::ParameterName, 'Delete a specific memory by id')
            [CompletionResult]::new('--threshold', '--threshold', [CompletionResultType]::ParameterName, 'Archive long-term memories whose ACT-R heat is below this threshold (0..1 recommended)')
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Preview actions without writing')
            [CompletionResult]::new('--no-commit', '--no-commit', [CompletionResultType]::ParameterName, 'Skip Git commit')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;log' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum number of commits to show')
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create and checkout a new branch')
            [CompletionResult]::new('checkout', 'checkout', [CompletionResultType]::ParameterValue, 'Checkout an existing branch')
            [CompletionResult]::new('merge', 'merge', [CompletionResultType]::ParameterValue, 'Merge a branch into the current branch')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List local branches')
            [CompletionResult]::new('current', 'current', [CompletionResultType]::ParameterValue, 'Print current branch name')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a local branch')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;branch;create' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;checkout' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;merge' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;list' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;current' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;delete' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;branch;help' {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create and checkout a new branch')
            [CompletionResult]::new('checkout', 'checkout', [CompletionResultType]::ParameterValue, 'Checkout an existing branch')
            [CompletionResult]::new('merge', 'merge', [CompletionResultType]::ParameterValue, 'Merge a branch into the current branch')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List local branches')
            [CompletionResult]::new('current', 'current', [CompletionResultType]::ParameterValue, 'Print current branch name')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a local branch')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;branch;help;create' {
            break
        }
        'chronicle;branch;help;checkout' {
            break
        }
        'chronicle;branch;help;merge' {
            break
        }
        'chronicle;branch;help;list' {
            break
        }
        'chronicle;branch;help;current' {
            break
        }
        'chronicle;branch;help;delete' {
            break
        }
        'chronicle;branch;help;help' {
            break
        }
        'chronicle;completions' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;wal' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'run')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;wal;run' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;wal;help' {
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'run')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;wal;help;run' {
            break
        }
        'chronicle;wal;help;help' {
            break
        }
        'chronicle;status' {
            [CompletionResult]::new('--root', '--root', [CompletionResultType]::ParameterName, 'Project root (defaults to auto-detect from current directory)')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Git commit mode after write/touch operations')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'chronicle;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize `.chronicle` directory structure in the project')
            [CompletionResult]::new('remember', 'remember', [CompletionResultType]::ParameterValue, 'Store a new memory block (and commit it to Git)')
            [CompletionResult]::new('recall', 'recall', [CompletionResultType]::ParameterValue, 'Recall memory blocks using the MAMA matching algorithm')
            [CompletionResult]::new('consolidate', 'consolidate', [CompletionResultType]::ParameterValue, 'Move eligible short-term memories into long-term storage')
            [CompletionResult]::new('forget', 'forget', [CompletionResultType]::ParameterValue, 'Apply forgetting: move low-strength memories to archive or delete')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show memory change history (wraps `git log`)')
            [CompletionResult]::new('branch', 'branch', [CompletionResultType]::ParameterValue, 'Git branch helpers for sandboxed reasoning')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Print shell completion script')
            [CompletionResult]::new('wal', 'wal', [CompletionResultType]::ParameterValue, 'Internal: process WAL tasks')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Print project status summary')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'chronicle;help;init' {
            break
        }
        'chronicle;help;remember' {
            break
        }
        'chronicle;help;recall' {
            break
        }
        'chronicle;help;consolidate' {
            break
        }
        'chronicle;help;forget' {
            break
        }
        'chronicle;help;log' {
            break
        }
        'chronicle;help;branch' {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create and checkout a new branch')
            [CompletionResult]::new('checkout', 'checkout', [CompletionResultType]::ParameterValue, 'Checkout an existing branch')
            [CompletionResult]::new('merge', 'merge', [CompletionResultType]::ParameterValue, 'Merge a branch into the current branch')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List local branches')
            [CompletionResult]::new('current', 'current', [CompletionResultType]::ParameterValue, 'Print current branch name')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a local branch')
            break
        }
        'chronicle;help;branch;create' {
            break
        }
        'chronicle;help;branch;checkout' {
            break
        }
        'chronicle;help;branch;merge' {
            break
        }
        'chronicle;help;branch;list' {
            break
        }
        'chronicle;help;branch;current' {
            break
        }
        'chronicle;help;branch;delete' {
            break
        }
        'chronicle;help;completions' {
            break
        }
        'chronicle;help;wal' {
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'run')
            break
        }
        'chronicle;help;wal;run' {
            break
        }
        'chronicle;help;status' {
            break
        }
        'chronicle;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
