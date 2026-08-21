mod commands;

#[macro_use]
extern crate log;

use std::time::Instant;
use tracel_xtask::prelude::*;

#[derive(clap::Subcommand, strum::Display)]
enum Command {
    Build(BuildCmdArgs),
    Bump(BumpCmdArgs),
    Check(CheckCmdArgs),
    Clean(CleanCmdArgs),
    Compile(CompileCmdArgs),
    Doc(DocCmdArgs),
    Fix(FixCmdArgs),
    Publish(PublishCmdArgs),
    Validate(ValidateCmdArgs),
    /// Generate bindings.
    Bindgen(commands::bindgen::BindgenCmdArgs),
    /// Test bindings.
    Test(commands::test::CubeClHipTestCmdArgs),
}

fn dispatch_base_commands(args: XtaskArgs<Command>, env: Environment) -> anyhow::Result<()> {
    match args.command {
        Command::Build(cmd) => base_commands::build::handle_command(cmd, env, args.context),
        Command::Bump(cmd) => base_commands::bump::handle_command(cmd, env, args.context),
        Command::Check(cmd) => base_commands::check::handle_command(cmd, env, args.context),
        Command::Clean(cmd) => base_commands::clean::handle_command(cmd, env, args.context),
        Command::Compile(cmd) => base_commands::compile::handle_command(cmd, env, args.context),
        Command::Doc(cmd) => base_commands::doc::handle_command(cmd, env, args.context),
        Command::Fix(cmd) => base_commands::fix::handle_command(cmd, env, args.context, None),
        Command::Publish(cmd) => base_commands::publish::handle_command(cmd, env, args.context),
        Command::Validate(cmd) => base_commands::validate::handle_command(cmd, env, args.context),
        _ => Err(anyhow::anyhow!("Unknown command")),
    }
}

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let (args, environment) = init_xtask::<Command>(parse_args::<Command>()?)?;
    match args.command {
        Command::Bindgen(cmd_args) => commands::bindgen::handle_command(cmd_args),
        Command::Test(cmd_args) => {
            commands::test::handle_command(cmd_args, environment, args.context)
        }
        _ => dispatch_base_commands(args, environment),
    }?;
    let duration = start.elapsed();
    info!(
        "\x1B[32;1mTime elapsed for the current execution: {}\x1B[0m",
        format_duration(&duration)
    );
    Ok(())
}
