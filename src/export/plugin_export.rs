use std::io;
use std::path::Path;

const GODOT_RUNNER: &str =
    include_str!("../../plugins/godot/addons/talenode/talenode_runner.gd");
const GODOT_EXPR: &str =
    include_str!("../../plugins/godot/addons/talenode/talenode_expr.gd");
const GODOT_CFG: &str =
    include_str!("../../plugins/godot/addons/talenode/plugin.cfg");

// Unity — Runtime
const UNITY_RUNNER: &str =
    include_str!("../../plugins/unity/Runtime/TaleNodeRunner.cs");
const UNITY_EXPR: &str =
    include_str!("../../plugins/unity/Runtime/TaleNodeExpression.cs");
const UNITY_DATA: &str =
    include_str!("../../plugins/unity/Runtime/TaleNodeDialogueData.cs");
const UNITY_RUNTIME_ASMDEF: &str =
    include_str!("../../plugins/unity/Runtime/TaleNode.Runtime.asmdef");
const UNITY_PACKAGE_JSON: &str =
    include_str!("../../plugins/unity/package.json");

// Unity — Editor
const UNITY_EDITOR_ASMDEF: &str =
    include_str!("../../plugins/unity/Editor/TaleNode.Editor.asmdef");
const UNITY_DIALOGUE_ASSET: &str =
    include_str!("../../plugins/unity/Editor/Asset/TaleNodeDialogue.cs");
const UNITY_IMPORTER: &str =
    include_str!("../../plugins/unity/Editor/Asset/TaleNodeDialogueImporter.cs");
const UNITY_INSPECTOR: &str =
    include_str!("../../plugins/unity/Editor/Asset/TaleNodeDialogueEditor.cs");
const UNITY_GRAPH_WINDOW: &str =
    include_str!("../../plugins/unity/Editor/GraphView/TaleNodeGraphWindow.cs");
const UNITY_GRAPH_VIEW: &str =
    include_str!("../../plugins/unity/Editor/GraphView/TaleNodeGraphView.cs");
const UNITY_GRAPH_NODE: &str =
    include_str!("../../plugins/unity/Editor/GraphView/TaleNodeGraphNode.cs");
const UNITY_GRAPH_LAYOUT: &str =
    include_str!("../../plugins/unity/Editor/GraphView/TaleNodeGraphLayout.cs");
const UNITY_GRAPH_STYLES: &str =
    include_str!("../../plugins/unity/Editor/GraphView/TaleNodeGraphStyles.uss");
const UNITY_PLAYTEST: &str =
    include_str!("../../plugins/unity/Editor/Playtest/TaleNodePlaytestPanel.cs");

// Unreal
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

/// Export the Unity plugin as a UPM package into `dir/`.
pub fn export_unity_plugin(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("package.json"), UNITY_PACKAGE_JSON)?;

    // Runtime
    let runtime = dir.join("Runtime");
    std::fs::create_dir_all(&runtime)?;
    std::fs::write(runtime.join("TaleNode.Runtime.asmdef"), UNITY_RUNTIME_ASMDEF)?;
    std::fs::write(runtime.join("TaleNodeRunner.cs"), UNITY_RUNNER)?;
    std::fs::write(runtime.join("TaleNodeExpression.cs"), UNITY_EXPR)?;
    std::fs::write(runtime.join("TaleNodeDialogueData.cs"), UNITY_DATA)?;

    // Editor
    let editor = dir.join("Editor");
    std::fs::create_dir_all(&editor)?;
    std::fs::write(editor.join("TaleNode.Editor.asmdef"), UNITY_EDITOR_ASMDEF)?;

    let asset_dir = editor.join("Asset");
    std::fs::create_dir_all(&asset_dir)?;
    std::fs::write(asset_dir.join("TaleNodeDialogue.cs"), UNITY_DIALOGUE_ASSET)?;
    std::fs::write(asset_dir.join("TaleNodeDialogueImporter.cs"), UNITY_IMPORTER)?;
    std::fs::write(asset_dir.join("TaleNodeDialogueEditor.cs"), UNITY_INSPECTOR)?;

    let graph_dir = editor.join("GraphView");
    std::fs::create_dir_all(&graph_dir)?;
    std::fs::write(graph_dir.join("TaleNodeGraphWindow.cs"), UNITY_GRAPH_WINDOW)?;
    std::fs::write(graph_dir.join("TaleNodeGraphView.cs"), UNITY_GRAPH_VIEW)?;
    std::fs::write(graph_dir.join("TaleNodeGraphNode.cs"), UNITY_GRAPH_NODE)?;
    std::fs::write(graph_dir.join("TaleNodeGraphLayout.cs"), UNITY_GRAPH_LAYOUT)?;
    std::fs::write(graph_dir.join("TaleNodeGraphStyles.uss"), UNITY_GRAPH_STYLES)?;

    let playtest_dir = editor.join("Playtest");
    std::fs::create_dir_all(&playtest_dir)?;
    std::fs::write(playtest_dir.join("TaleNodePlaytestPanel.cs"), UNITY_PLAYTEST)?;

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
        assert!(!UNITY_DATA.is_empty());
        assert!(!UNITY_RUNTIME_ASMDEF.is_empty());
        assert!(!UNITY_PACKAGE_JSON.is_empty());
        assert!(!UNITY_EDITOR_ASMDEF.is_empty());
        assert!(!UNITY_DIALOGUE_ASSET.is_empty());
        assert!(!UNITY_IMPORTER.is_empty());
        assert!(!UNITY_INSPECTOR.is_empty());
        assert!(!UNITY_GRAPH_WINDOW.is_empty());
        assert!(!UNITY_GRAPH_VIEW.is_empty());
        assert!(!UNITY_GRAPH_NODE.is_empty());
        assert!(!UNITY_GRAPH_LAYOUT.is_empty());
        assert!(!UNITY_GRAPH_STYLES.is_empty());
        assert!(!UNITY_PLAYTEST.is_empty());
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

        assert!(tmp.join("package.json").exists());

        let runtime = tmp.join("Runtime");
        assert!(runtime.join("TaleNode.Runtime.asmdef").exists());
        assert!(runtime.join("TaleNodeRunner.cs").exists());
        assert!(runtime.join("TaleNodeExpression.cs").exists());
        assert!(runtime.join("TaleNodeDialogueData.cs").exists());

        let editor = tmp.join("Editor");
        assert!(editor.join("TaleNode.Editor.asmdef").exists());
        assert!(editor.join("Asset").join("TaleNodeDialogue.cs").exists());
        assert!(editor.join("Asset").join("TaleNodeDialogueImporter.cs").exists());
        assert!(editor.join("Asset").join("TaleNodeDialogueEditor.cs").exists());
        assert!(editor.join("GraphView").join("TaleNodeGraphWindow.cs").exists());
        assert!(editor.join("GraphView").join("TaleNodeGraphView.cs").exists());
        assert!(editor.join("GraphView").join("TaleNodeGraphNode.cs").exists());
        assert!(editor.join("GraphView").join("TaleNodeGraphLayout.cs").exists());
        assert!(editor.join("GraphView").join("TaleNodeGraphStyles.uss").exists());
        assert!(editor.join("Playtest").join("TaleNodePlaytestPanel.cs").exists());

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
