use clap::Parser;
use prog_args::ProgArgs;
use stats::cmd_status;
use ultimate_mod_man_rs_lib::mod_db::ModDb;

mod prog_args;
mod stats;

fn main() -> anyhow::Result<()> {
    let p_args = ProgArgs::parse();

    let db = ModDb::load_from_path(&p_args.state_dir_path)?;

    match p_args.command {
        prog_args::Command::Status(status_args) => cmd_status(&status_args, &db)?,
        prog_args::Command::Add(add_args) => todo!(),
        prog_args::Command::Delete => todo!(),
        prog_args::Command::Install(install_args) => todo!(),
        prog_args::Command::CheckForUpdates => todo!(),
        prog_args::Command::UpdateInstalled => todo!(),
        prog_args::Command::EnableDisable(enable_disable_args) => todo!(),
        prog_args::Command::Resolve => todo!(),
        prog_args::Command::ChangeSlot => todo!(),
        prog_args::Command::CleanCache => todo!(),
        prog_args::Command::SwitchCompare => todo!(),
    }

    Ok(())
}
