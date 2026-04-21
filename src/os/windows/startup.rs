use std::env;

use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE_NAME: &str = "CloverViewer";
const STARTUP_ARG: &str = "--startup";

pub fn set_launch_on_startup(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu
        .create_subkey(RUN_KEY_PATH)
        .map_err(|err| format!("open run key failed: {err}"))?;

    if enabled {
        let exe_path = env::current_exe().map_err(|err| format!("read exe path failed: {err}"))?;
        let command = format!("\"{}\" {}", exe_path.display(), STARTUP_ARG);
        run_key
            .set_value(RUN_VALUE_NAME, &command)
            .map_err(|err| format!("write run value failed: {err}"))?;
    } else {
        match run_key.delete_value(RUN_VALUE_NAME) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(format!("delete run value failed: {err}")),
        }
    }

    Ok(())
}