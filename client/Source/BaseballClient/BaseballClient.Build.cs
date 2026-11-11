using UnrealBuildTool;

public class BaseballClient : ModuleRules
{
	public BaseballClient(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
		PublicDependencyModuleNames.AddRange(new string[] { "Core", "CoreUObject", "Engine", "InputCore" });
		// The sim's C header (sim/crates/bb_ffi/include/bb_ffi.h). The library itself is
		// loaded at run time (BbSim.cpp), so nothing links against Rust here.
		PublicIncludePaths.Add(System.IO.Path.Combine(ModuleDirectory, "..", "..", "..", "sim", "crates", "bb_ffi", "include"));
	}
}
