use std::io;
use std::path::Path;

const GODOT_RUNNER: &str =
    include_str!("../../plugins/godot/addons/talenode/talenode_runner.gd");
const GODOT_EXPR: &str =
    include_str!("../../plugins/godot/addons/talenode/talenode_expr.gd");
const GODOT_CFG: &str =
    include_str!("../../plugins/godot/addons/talenode/plugin.cfg");

const UNITY_RUNNER: &str =
    include_str!("../../plugins/unity/TaleNode/TaleNodeRunner.cs");
const UNITY_EXPR: &str =
    include_str!("../../plugins/unity/TaleNode/TaleNodeExpression.cs");

const UNREAL_RUNNER_H: &str =
    include_str!("../../plugins/unreal/TaleNode/TaleNodeRunner.h");
const UNREAL_RUNNER_CPP: &str =
    include_str!("../../plugins/unreal/TaleNode/TaleNodeRunner.cpp");
const UNREAL_VALUE_CPP: &str =
    include_str!("../../plugins/unreal/TaleNode/TaleNodeValue.cpp");
const UNREAL_PROCESS_CPP: &str =
    include_str!("../../plugins/unreal/TaleNode/TaleNodeProcess.cpp");

/// Export the Godot plugin into `dir/addons/talenode/`.
pub fn export_godot_plugin(dir: &Path) -> io::Result<()> {
    let addon_dir = dir.join("addons").join("talenode");
    std::fs::create_dir_all(&addon_dir)?;
    std::fs::write(addon_dir.join("plugin.cfg"), GODOT_CFG)?;
    std::fs::write(addon_dir.join("talenode_expr.gd"), GODOT_EXPR)?;
    std::fs::write(addon_dir.join("talenode_runner.gd"), GODOT_RUNNER)?;
    Ok(())
}

/// Export the Unity plugin into `dir/TaleNode/`.
pub fn export_unity_plugin(dir: &Path) -> io::Result<()> {
    let plugin_dir = dir.join("TaleNode");
    std::fs::create_dir_all(&plugin_dir)?;
    std::fs::write(plugin_dir.join("TaleNodeExpression.cs"), UNITY_EXPR)?;
    std::fs::write(plugin_dir.join("TaleNodeRunner.cs"), UNITY_RUNNER)?;
    Ok(())
}

/// Export the Unreal plugin into `dir/TaleNode/`.
pub fn export_unreal_plugin(dir: &Path) -> io::Result<()> {
    let plugin_dir = dir.join("TaleNode");
    std::fs::create_dir_all(&plugin_dir)?;
    std::fs::write(plugin_dir.join("TaleNodeRunner.h"), UNREAL_RUNNER_H)?;
    std::fs::write(plugin_dir.join("TaleNodeRunner.cpp"), UNREAL_RUNNER_CPP)?;
    std::fs::write(plugin_dir.join("TaleNodeValue.cpp"), UNREAL_VALUE_CPP)?;
    std::fs::write(plugin_dir.join("TaleNodeProcess.cpp"), UNREAL_PROCESS_CPP)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_files_are_not_empty() {
        assert!(!GODOT_RUNNER.is_empty());
        assert!(!GODOT_EXPR.is_empty());
        assert!(!GODOT_CFG.is_empty());
        assert!(!UNITY_RUNNER.is_empty());
        assert!(!UNITY_EXPR.is_empty());
        assert!(!UNREAL_RUNNER_H.is_empty());
        assert!(!UNREAL_RUNNER_CPP.is_empty());
        assert!(!UNREAL_VALUE_CPP.is_empty());
        assert!(!UNREAL_PROCESS_CPP.is_empty());
    }

    #[test]
    fn export_godot_creates_files() {
        let tmp = std::env::temp_dir().join("talenode_test_godot");
        let _ = std::fs::remove_dir_all(&tmp);
        export_godot_plugin(&tmp).unwrap();

        let addon = tmp.join("addons").join("talenode");
        assert!(addon.join("plugin.cfg").exists());
        assert!(addon.join("talenode_expr.gd").exists());
        assert!(addon.join("talenode_runner.gd").exists());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn export_unity_creates_files() {
        let tmp = std::env::temp_dir().join("talenode_test_unity");
        let _ = std::fs::remove_dir_all(&tmp);
        export_unity_plugin(&tmp).unwrap();

        let plugin = tmp.join("TaleNode");
        assert!(plugin.join("TaleNodeExpression.cs").exists());
        assert!(plugin.join("TaleNodeRunner.cs").exists());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn export_unreal_creates_files() {
        let tmp = std::env::temp_dir().join("talenode_test_unreal");
        let _ = std::fs::remove_dir_all(&tmp);
        export_unreal_plugin(&tmp).unwrap();

        let plugin = tmp.join("TaleNode");
        assert!(plugin.join("TaleNodeRunner.h").exists());
        assert!(plugin.join("TaleNodeRunner.cpp").exists());
        assert!(plugin.join("TaleNodeValue.cpp").exists());
        assert!(plugin.join("TaleNodeProcess.cpp").exists());

        std::fs::remove_dir_all(&tmp).ok();
    }
}
