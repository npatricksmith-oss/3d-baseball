using UnrealBuildTool;

public class BaseballClientEditorTarget : TargetRules
{
	public BaseballClientEditorTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.Latest;
		IncludeOrderVersion = EngineIncludeOrderVersion.Latest;
		ExtraModuleNames.Add("BaseballClient");
	}
}
