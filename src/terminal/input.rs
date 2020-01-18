use dialoguer::{Confirmation, Input};

pub fn ask_input(prompt: &str) -> Option<String> {
  let input_opt = Input::<String>::new().with_prompt(prompt).interact();
  match input_opt {
    Ok(input) => Some(input),
    Err(_) => None,
  }
}

pub fn ask_yes_no_question(prompt: &str) -> Option<bool> {
  let input_opt = Confirmation::new().with_text(prompt).interact();
  match input_opt {
    Ok(input) => Some(input),
    Err(_) => None,
  }
}
