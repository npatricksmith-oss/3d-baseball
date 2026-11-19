using System;
using System.Diagnostics;
using System.IO;
using System.Text.RegularExpressions;
using UnrealBuildTool;

public class BaseballClient : ModuleRules
{
	private static string RunGit(string RepoRoot, string Args)
	{
		try
		{
			var Psi = new ProcessStartInfo("git", Args)
			{
				WorkingDirectory = RepoRoot,
				RedirectStandardOutput = true,
				RedirectStandardError = true,
				UseShellExecute = false,
				CreateNoWindow = true,
			};
			using (var Proc = Process.Start(Psi))
			{
				if (Proc == null)
				{
					return null;
				}
				string Output = Proc.StandardOutput.ReadToEnd();
				Proc.WaitForExit(3000);
				return Proc.ExitCode == 0 ? Output : null;
			}
		}
		catch
		{
			return null;
		}
	}

	public BaseballClient(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
		PublicDependencyModuleNames.AddRange(new string[] { "Core", "CoreUObject", "Engine", "InputCore", "ProceduralMeshComponent", "MeshDescription", "StaticMeshDescription", "Slate", "SlateCore" });
		// The sim's C header (sim/crates/bb_ffi/include/bb_ffi.h). The library itself is
		// loaded at run time (BbSim.cpp), so nothing links against Rust here.
		string RepoRoot = Path.GetFullPath(Path.Combine(ModuleDirectory, "..", "..", ".."));
		PublicIncludePaths.Add(Path.Combine(RepoRoot, "sim", "crates", "bb_ffi", "include"));

		string Commit = "unknown";
		string Sha = RunGit(RepoRoot, "rev-parse --short HEAD");
		if (!string.IsNullOrWhiteSpace(Sha))
		{
			Sha = Sha.Trim();
			if (Regex.IsMatch(Sha, "^[0-9a-fA-F]+$"))
			{
				string Status = RunGit(RepoRoot, "status --porcelain");
				Commit = string.IsNullOrEmpty(Status) ? Sha : Sha + "-dirty";
			}
		}
		foreach (string GitPathArg in new[] { "rev-parse --git-path HEAD", "rev-parse --git-path index" })
		{
			string GitPath = RunGit(RepoRoot, GitPathArg);
			if (!string.IsNullOrWhiteSpace(GitPath))
			{
				GitPath = GitPath.Trim();
				string Full = Path.IsPathRooted(GitPath) ? GitPath : Path.GetFullPath(Path.Combine(RepoRoot, GitPath));
				if (File.Exists(Full))
				{
					ExternalDependencies.Add(Full);
				}
			}
		}
		PublicDefinitions.Add("BB_GIT_COMMIT=" + Commit);
	}
}
