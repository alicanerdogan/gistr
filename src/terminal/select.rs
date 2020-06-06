use dialoguer::{theme::ColorfulTheme, Select};

pub fn create_select<S: Into<String>>(selections: &Vec<String>, prompt: S) -> Option<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt.into())
        .paged(true)
        .default(0)
        .items(&selections)
        .interact_opt()
        .unwrap()
}
