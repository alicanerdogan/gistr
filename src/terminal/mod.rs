use console::Term;

pub mod input;
pub mod password;
pub mod select;

pub fn clear_all() {
  let term = Term::stdout();
  match term.clear_screen() {
    _ => {}
  }
}
