using UnrealBuildTool;

public class BaseballClientTarget : TargetRules
{
	public BaseballClientTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Game;
		DefaultBuildSettings = BuildSettingsVersion.Latest;
		IncludeOrderVersion = EngineIncludeOrderVersion.Latest;
		ExtraModuleNames.Add("BaseballClient");
	}
}
