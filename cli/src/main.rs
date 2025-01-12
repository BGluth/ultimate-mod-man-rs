use clap::Parser;
use prog_args::ProgArgs;
use ultimate_mod_man_rs_lib::mod_db::ModDb;

mod prog_args;

fn main() -> anyhow::Result<()> {
    let p_args = ProgArgs::parse();

    let db = ModDb::load_from_path(&p_args.state_dir_path)?;

    Ok(())
}
