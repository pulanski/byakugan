use driver::Byakugan;
use miette::Result;
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    Byakugan::run()
}
