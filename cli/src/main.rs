use clap::Parser;
use prog_args::{ProgArgs, StatusCliArgs};
use ultimate_mod_man_rs_core::{cmds::status::StatusCmdInfo, mod_manager::ModManager};

mod prog_args;

fn main() -> anyhow::Result<()> {
    let p_args = ProgArgs::parse();

    let mm = ModManager::new(&p_args.state_dir_path)?;

    match p_args.command {
        prog_args::Command::Status(status_args) => mm.status(status_args.into())?,
        prog_args::Command::Add(add_args) => todo!(),
        prog_args::Command::Delete => todo!(),
        prog_args::Command::CheckForUpdates => todo!(),
        prog_args::Command::SyncWithSwitch => todo!(),
        prog_args::Command::EnableDisable(enable_disable_args) => todo!(),
        prog_args::Command::ResolveConflicts => todo!(),
        prog_args::Command::ChangeSlot => todo!(),
        prog_args::Command::SwitchCompare => todo!(),
    }

    Ok(())
}

impl From<StatusCliArgs> for StatusCmdInfo {
    fn from(v: StatusCliArgs) -> Self {
        let no_mods_specified = v.mods.mods.is_empty();

        match no_mods_specified {
            false => StatusCmdInfo::Generic,
            true => StatusCmdInfo::Specific(v.mods.mods),
        }
    }
}
