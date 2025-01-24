pub trait UserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool;

    /// The list provide is guarenteed to always have at least one element.
    fn select_item_from_list<T>(&mut self, items: &[T]) -> usize;
}
