use dialoguer::{
    Select,
    theme::ColorfulTheme
};
use console::Term;

pub fn prompt_for_commit_selection(items: &Vec<String>) -> std::io::Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    return match selection {
        Some(index) => {
            println!("User selected item : {}", items[index]);
            return Ok(index) 
        },
        None => Ok(0)
    }
}
