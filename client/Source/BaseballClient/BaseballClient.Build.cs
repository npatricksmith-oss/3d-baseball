using System.IO;
using UnrealBuildTool;

public class BaseballClient : ModuleRules
{
	public BaseballClient(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
		PublicDependencyModuleNames.AddRange(new string[] { "Core", "CoreUObject", "Engine", "InputCore", "ProceduralMeshComponent", "MeshDescription", "StaticMeshDescription", "Slate", "SlateCore" });
		// The sim's C header (sim/crates/bb_ffi/include/bb_ffi.h). The library itself is
		// loaded at run time (BbSim.cpp), so nothing links against Rust here.
		string RepoRoot = Path.GetFullPath(Path.Combine(ModuleDirectory, "..", "..", ".."));
		PublicIncludePaths.Add(Path.Combine(RepoRoot, "sim", "crates", "bb_ffi", "include"));

		// The build's commit stamp (BbBuildStamp.h). Client/Build/stamp-commit.sh, a pre-build
		// step of both targets, rewrites it before every build; reading git here would be
		// cached by UBT between builds and go stale. A placeholder is written only if the
		// step has never run (a build by another route), so the include always resolves.
		string StampDir = Path.GetFullPath(Path.Combine(ModuleDirectory, "..", "..", "Intermediate", "BbStamp"));
		string StampHeader = Path.Combine(StampDir, "BbBuildStamp.h");
		if (!File.Exists(StampHeader))
		{
			Directory.CreateDirectory(StampDir);
			File.WriteAllText(StampHeader, "#pragma once\n#define BB_BUILD_COMMIT \"unknown\"\n");
		}
		PrivateIncludePaths.Add(StampDir);
	}
}
