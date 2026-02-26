mod export;
mod import;
mod info;
mod validate;

const MAIN_USAGE: &str = "\
Usage: talenode <command> [options]

Commands:
  export     Export .talenode to various formats (json, xml, yarn, ...)
  validate   Check a .talenode file for errors and warnings
  info       Show project statistics
  import     Convert external formats to .talenode

Run 'talenode <command> --help' for details on each command.";

pub fn run_cli(command: &str, args: &[String]) -> Result<(), String> {
    match command {
        "export" => export::run_export(args),
        "validate" => validate::run_validate(args),
        "info" => info::run_info(args),
        "import" => import::run_import(args),
        "--help" | "-h" | "help" => {
            println!("{MAIN_USAGE}");
            Ok(())
        }
        _ => Err(format!("Unknown command: '{command}'\n\n{MAIN_USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_command() {
        assert!(run_cli("help", &[]).is_ok());
    }

    #[test]
    fn help_flag() {
        assert!(run_cli("--help", &[]).is_ok());
    }

    #[test]
    fn unknown_command_returns_error() {
        let result = run_cli("foobar", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }
}
