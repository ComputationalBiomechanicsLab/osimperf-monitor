use std::io::stdout;

use anyhow::Result;
use env_logger::Env;
use osimperf_lib::*;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    do_main()?;

    Ok(())
}

fn do_main() -> Result<()> {
    let cmd = PipedCommands::parse("echo hello-world!\nanother-line|grep hello");

    let out = cmd.run_and_stream(&mut stdout())?;

    println!("out = {:#?}", out);

    Ok(())
}
