// TaleNode — ScriptedImporter for .talenode.json files.
// Automatically imports any *.talenode.json file as a TaleNodeDialogue asset.

using UnityEditor.AssetImporters;
using UnityEngine;

namespace TaleNode.Editor
{
    [ScriptedImporter(1, "talenode.json")]
    public class TaleNodeDialogueImporter : ScriptedImporter
    {
        public override void OnImportAsset(AssetImportContext ctx)
        {
            string json = System.IO.File.ReadAllText(ctx.assetPath);
            var asset = ScriptableObject.CreateInstance<TaleNodeDialogue>();

            if (!asset.Parse(json))
            {
                Debug.LogError($"[TaleNode] Failed to parse {ctx.assetPath}");
                ctx.AddObjectToAsset("main", asset);
                ctx.SetMainObject(asset);
                return;
            }

            asset.name = asset.DialogueName;
            ctx.AddObjectToAsset("main", asset);
            ctx.SetMainObject(asset);
        }
    }
}
