use dialoguer::{theme::ColorfulTheme, PasswordInput};

pub fn ask_password(prompt: &str) -> Option<String> {
    let password_opt = PasswordInput::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact();

    match password_opt {
        Ok(password) => Some(password),
        Err(_) => None,
    }
}
