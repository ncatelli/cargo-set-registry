use cargo_edit::CargoResult;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(bin_name = "cargo")]
pub enum Command {
    SetRegistry(crate::set_registry::SetRegistryArgs),
}

impl Command {
    pub fn exec(self) -> CargoResult<()> {
        match self {
            Self::SetRegistry(sra) => sra.exec(),
        }
    }
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Command::command().debug_assert()
}
