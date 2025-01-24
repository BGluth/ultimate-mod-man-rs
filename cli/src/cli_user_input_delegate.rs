use ultimate_mod_man_rs_utils::user_input_delegate::UserInputDelegate;

#[derive(Debug)]
pub(crate) struct CliUserInputDelegate {}

impl CliUserInputDelegate {
    pub(crate) fn new() -> Self {
        todo!()
    }
}

impl UserInputDelegate for CliUserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool {
        todo!()
    }

    fn select_item_from_list<T: std::fmt::Display>(&mut self, items: &[T]) -> usize {
        todo!()
    }
}
