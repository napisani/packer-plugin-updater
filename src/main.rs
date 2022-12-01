use clap::Parser;
use std::{fs, path};
mod git;
mod lua;
mod prompt;
mod result;
use full_moon::{ast, visitors::*};
use result::Result;

fn get_default_repo_base_path() -> path::PathBuf {
    let p = home::home_dir().unwrap();
    p.join(".local")
        .join("share")
        .join("nvim")
        .join("site")
        .join("pack")
        .join("packer")
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct UpdaterOptions {
    #[arg(short, long)]
    pub plugin_lua_file: String,
    #[arg(short, long, default_value_t = get_default_repo_base_path().display().to_string())]
    pub repo_base_path: String,
    #[arg(short, long)]
    pub output_file: String,
}
fn get_repo_name(plugin_name: &str) -> String {
    if let Some(found) = plugin_name.find('/') {
        return plugin_name[(found + 1)..].to_owned();
    }
    plugin_name.to_owned()
}

fn identify_target_branch_name(default_branch: &str, locked_branch: Option<&str>) -> String {
    match locked_branch {
        Some(defined_locked_branch) => defined_locked_branch,
        _ => default_branch,
    }
    .to_owned()
}

fn main() -> Result<()> {
    let updater_opts = UpdaterOptions::parse();
    interactively_update_plugins(updater_opts)
}

fn interactively_update_plugins(updater_opts: UpdaterOptions) -> Result<()> {
    let tree = lua::parse_lua(&updater_opts.plugin_lua_file)?;
    let stmts: Vec<&ast::Stmt> = tree.nodes().stmts().collect();
    let define_plugins_func = lua::find_define_plugins_function(&stmts);
    let use_calls = lua::parse_packer_use_calls(define_plugins_func);
    struct FnCallVistor<'a> {
        use_calls: Vec<&'a ast::FunctionCall>, // use_calls: &'a UsePluginCall<'a>
        updater_opts: &'a UpdaterOptions,
    }
    impl VisitorMut for FnCallVistor<'_> {
        fn visit_table_constructor(
            &mut self,
            node: ast::TableConstructor,
        ) -> ast::TableConstructor {
            if let Some(_func_call) =
                lua::get_function_call_by_table_ctor(self.use_calls.clone(), &node)
            {
                let plugin_name = lua::get_plugin_name(&node).unwrap();
                let repo_name = get_repo_name(plugin_name);
                let commit = lua::get_commit(&node).unwrap();
                let locked_to_branch = lua::get_branch(&node);
                let mut repo_path = path::Path::new(&self.updater_opts.repo_base_path)
                    .join("start")
                    .join(repo_name.clone());
                if !repo_path.exists() {
                    repo_path = path::Path::new(&self.updater_opts.repo_base_path)
                        .join("opt")
                        .join(repo_name);
                }
                // lua::replace_table_constructor(&node, "AAAAAA");
                let repo = git::get_repo(&repo_path).unwrap();
                let default_branch = git::get_remote_branch_name(&repo_path).unwrap();
                let locked_to_branch =
                    identify_target_branch_name(&default_branch, locked_to_branch);
                let head_commits = git::find_latest_commits(&repo).unwrap();
                let head_commit: &git::RemoteHeadCommit = head_commits
                    .iter()
                    .find(|c| c.name == locked_to_branch)
                    .unwrap();

                let head_commits_prompt: Vec<String> = vec![
                    git::RemoteHeadCommit::from_current_commit(commit.to_owned()).to_string(),
                    head_commit.to_string(),
                ];

                println!("Plugin: {}", plugin_name);
                if commit == head_commit.sha {
                    println!("already up to date - skipping");
                    return node;
                }
                let idx = prompt::prompt_for_commit_selection(&head_commits_prompt).unwrap();
                if idx == 1 {
                    println!(
                        "updating plugin {} from {} -> {}",
                        plugin_name, commit, head_commit.sha
                    );
                    if let Some(new_ctor) = lua::replace_table_constructor(&node, &head_commit.sha)
                    {
                        return new_ctor;
                    }
                }
            }
            node
        }
    }

    let new_ast = FnCallVistor {
        use_calls,
        updater_opts: &updater_opts,
    }
    .visit_ast(tree.clone());
    fs::write(updater_opts.output_file, full_moon::print(&new_ast)).expect("Unable to write file");
    Ok(())

}
// #[cfg(test)]
// mod tests {
//     use crate::{UpdaterOptions, interactively_update_plugins};
//     #[test]
//     fn it_works() {
//         let opts = UpdaterOptions{
//             plugin_lua_file: "/Users/nick/.config/nvim/lua/user/plugins.lua".to_owned(),
//             output_file: "/tmp/f.lua".to_owned(),
//             repo_base_path: "/Users/nick/.local/share/nvim/site/pack/packer/".to_owned()
//         };
//         interactively_update_plugins(opts);
//     }
// }
