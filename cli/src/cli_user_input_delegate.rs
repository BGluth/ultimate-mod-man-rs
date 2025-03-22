use std::{
    fmt::{self},
    io,
};

use ultimate_mod_man_rs_utils::{
    types::{
        AssetSlot, AvailableSlotsToSwapToInfo, PickedNonSwappableResolutionOption,
        PickedResolutionOption, VariantAndId,
    },
    user_input_delegate::{UserInputDelegate, VariantConflictSummary},
};

#[derive(Debug)]
pub(crate) struct CliUserInputDelegate {
    buf: String,
}

impl CliUserInputDelegate {
    pub(crate) fn new() -> Self {
        Self { buf: String::new() }
    }

    fn read_user_input(&mut self) {
        self.buf.clear();
        io::stdin()
            .read_line(&mut self.buf)
            .expect("Unable to read from stdin!");
    }

    fn get_item_index_of_item(&mut self, num_items: usize) -> usize {
        loop {
            self.read_user_input();
            let input = self.buf.trim();

            let n = match input.parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("Is not parsable to a non-negative integer.");
                    continue;
                },
            };

            if n >= num_items {
                println!(
                    "{} is not an item selection between 0 - {}",
                    n,
                    num_items - 1
                );
                continue;
            }

            return n;
        }
    }
}

impl UserInputDelegate for CliUserInputDelegate {
    fn get_yes_no_resp(&mut self) -> bool {
        print!("(y/n)");
        self.read_user_input();

        // Assume anything else is a `no` for now.
        let input = self.buf.trim().to_lowercase();
        input.starts_with("y") || input.starts_with("yes")
    }

    fn select_item_from_list<T: fmt::Display>(&mut self, items: &[T]) -> usize {
        for (i, item) in items.iter().enumerate() {
            println!("{i} - {item}");
        }

        self.get_item_index_of_item(items.len())
    }

    fn display_variant_conflict_summary(&mut self, summary: &VariantConflictSummary) {
        todo!()
    }

    fn get_variant_conflict_resolution_option_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &AssetSlot,
        available_slots_info: &AvailableSlotsToSwapToInfo,
    ) -> PickedResolutionOption {
        todo!()
    }

    fn get_variant_conflict_resolution_option_non_swappable(
        &mut self,
        existing: &VariantAndId,
        new: &VariantAndId,
        slot: &AssetSlot,
    ) -> PickedNonSwappableResolutionOption {
        todo!()
    }
}
