use std::path::Path;

use vterm_core::protocol::DebugLaunchConfig;

pub fn expand_launch_vars(launch: &mut DebugLaunchConfig, repo_root: &Path) {
    launch.command = expand_workspace_vars(&launch.command, repo_root);
    launch.program = expand_workspace_vars(&launch.program, repo_root);
    launch.runtime_executable = expand_workspace_vars(&launch.runtime_executable, repo_root);
    launch.args = launch
        .args
        .iter()
        .map(|arg| expand_workspace_vars(arg, repo_root))
        .collect();
    launch.runtime_args = launch
        .runtime_args
        .iter()
        .map(|arg| expand_workspace_vars(arg, repo_root))
        .collect();
    launch.pre_tasks = launch
        .pre_tasks
        .iter()
        .map(|task| expand_workspace_vars(task, repo_root))
        .collect();
    launch.pre_launch_task = launch
        .pre_launch_task
        .as_deref()
        .map(|task| expand_workspace_vars(task, repo_root));
    launch.cwd = launch
        .cwd
        .as_deref()
        .map(|cwd| expand_workspace_vars(cwd, repo_root));
    launch.env = launch
        .env
        .iter()
        .map(|(key, value)| (key.clone(), expand_workspace_vars(value, repo_root)))
        .collect();
}

pub fn expand_workspace_vars(value: &str, repo_root: &Path) -> String {
    if value.is_empty() {
        return String::new();
    }

    let repo_root = repo_root.display().to_string();
    value
        .replace("${workspaceFolder}", &repo_root)
        .replace("${workspaceRoot}", &repo_root)
        .replace("${workspace_folder}", &repo_root)
}
