use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

pub fn prompt_for_commit_selection(items: &[String]) -> std::io::Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    match selection {
        Some(index) => {
            println!("User selected item : {}", items[index]);
            Ok(index)
        }
        None => Ok(0),
    }
}
