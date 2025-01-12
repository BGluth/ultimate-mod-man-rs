use std::fmt::{self, Display, Formatter};

use ptree::TreeBuilder;
use ultimate_mod_man_rs_lib::mod_db::ModDb;

use crate::prog_args::StatusArgs;

#[derive(Debug)]
struct InstalledModAndVariantsInfo {
    name: String,
    variants: Vec<VariantNameAndEnabled>,
}

#[derive(Debug)]
struct VariantNameAndEnabled {
    name: String,
    enabled: bool,
}

pub(crate) fn cmd_status(args: &StatusArgs, db: &ModDb) -> anyhow::Result<()> {
    let no_mods_specified = args.mods.mods.is_empty();

    match no_mods_specified {
        false => todo!(),
        true => print!("{}", GenericModStats::new(db)),
    };

    Ok(())
}

#[derive(Debug, Default)]
struct GenericModStats {
    scalars: ScalarStats,
    installed_mods: Vec<InstalledModAndVariantsInfo>,
}

#[derive(Debug, Default)]
struct ScalarStats {
    num_mods_installed: usize,
    num_variants_installed: usize,
    num_mod_variants_enabled: usize,
}

impl Display for ScalarStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Number of mods installed: {}", self.num_mods_installed)?;
        writeln!(
            f,
            "Number of mod variants installed: {}",
            self.num_variants_installed
        )?;
        writeln!(
            f,
            "Number of mod variants enabled: {}",
            self.num_mod_variants_enabled
        )
    }
}

impl GenericModStats {
    fn new(db: &ModDb) -> Self {
        let mut generic_stats = GenericModStats::default();

        for mod_entry in db.installed_mods() {
            generic_stats.scalars.num_mods_installed += 1;

            let mut variants = Vec::new();
            for variant in mod_entry.installed_variants.iter() {
                generic_stats.scalars.num_mods_installed += 1;

                if variant.enabled {
                    generic_stats.scalars.num_mod_variants_enabled += 1;
                }

                let variant_info = VariantNameAndEnabled {
                    name: variant.name.clone(),
                    enabled: variant.enabled,
                };

                variants.push(variant_info);
            }

            generic_stats
                .installed_mods
                .push(InstalledModAndVariantsInfo {
                    name: mod_entry.name.clone(),
                    variants,
                });
        }

        generic_stats
    }
}

impl Display for GenericModStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.installed_mods.is_empty() {
            return writeln!(f, "No mods installed.");
        }

        write!(f, "{}", self.scalars)?;

        let mut p_tree = TreeBuilder::new("Installed Mods".to_string());

        for mod_entry in self.installed_mods.iter() {
            match mod_entry.variants.len() {
                0 => {
                    p_tree.add_empty_child(format!("{} (No variants)", mod_entry.name));
                }
                1 => {
                    let single_variant = &mod_entry.variants[0];
                    p_tree.add_empty_child(format!(
                        "{} --> {} ({})",
                        mod_entry.name, single_variant.name, single_variant.enabled
                    ));
                }
                _ => {
                    for variant in mod_entry.variants.iter() {
                        p_tree.begin_child(mod_entry.name.clone());
                        p_tree.add_empty_child(format!("{} ({})", variant.name, variant.enabled));
                        p_tree.end_child();
                    }
                }
            }
        }

        write!(f, "{}", p_tree.build().text)
    }
}
