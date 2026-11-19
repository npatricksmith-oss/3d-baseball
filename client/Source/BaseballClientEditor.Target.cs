using UnrealBuildTool;

public class BaseballClientEditorTarget : TargetRules
{
	public BaseballClientEditorTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.Latest;
		IncludeOrderVersion = EngineIncludeOrderVersion.Latest;
		ExtraModuleNames.Add("BaseballClient");
		// The build's commit, rewritten before every build (Build/stamp-commit.sh).
		PreBuildSteps.Add("\"$(ProjectDir)/Build/stamp-commit.sh\" \"$(ProjectDir)\"");
	}
}
