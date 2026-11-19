#include "BbSim.h"
#include "HAL/PlatformProcess.h"

DEFINE_LOG_CATEGORY_STATIC(LogBbSim, Log, All);

FBbSim::~FBbSim()
{
	if (Game && FnGameFree)
	{
		FnGameFree(Game);
	}
	if (Handle)
	{
		FPlatformProcess::FreeDllHandle(Handle);
	}
}

bool FBbSim::Start(const FString& LibraryPath, uint64 Seed)
{
	Handle = FPlatformProcess::GetDllHandle(*LibraryPath);
	if (!Handle)
	{
		UE_LOG(LogBbSim, Error, TEXT("Could not load the sim library at %s"), *LibraryPath);
		return false;
	}
	auto Sym = [this](const TCHAR* Name) { return FPlatformProcess::GetDllExport(Handle, Name); };
	FnCreate = reinterpret_cast<decltype(FnCreate)>(Sym(TEXT("bb_game_create")));
	FnAdvance = reinterpret_cast<decltype(FnAdvance)>(Sym(TEXT("bb_game_advance")));
	FnBlobFree = reinterpret_cast<decltype(FnBlobFree)>(Sym(TEXT("bb_blob_free")));
	FnGameFree = reinterpret_cast<decltype(FnGameFree)>(Sym(TEXT("bb_game_free")));
	FnInfo = reinterpret_cast<decltype(FnInfo)>(Sym(TEXT("bb_timeline_info")));
	FnSample = reinterpret_cast<decltype(FnSample)>(Sym(TEXT("bb_timeline_sample")));
	FnRoster = reinterpret_cast<decltype(FnRoster)>(Sym(TEXT("bb_game_roster")));
	// Optional: an older library without the box score still plays the game.
	FnBoxScore = reinterpret_cast<decltype(FnBoxScore)>(Sym(TEXT("bb_game_box_score")));
	FnPark = reinterpret_cast<decltype(FnPark)>(Sym(TEXT("bb_game_park")));
	if (!FnPark || !FnRoster || !FnCreate || !FnAdvance || !FnBlobFree || !FnGameFree || !FnInfo || !FnSample)
	{
		UE_LOG(LogBbSim, Error, TEXT("The sim library at %s is missing part of the C ABI"), *LibraryPath);
		return false;
	}
	Game = FnCreate(Seed);
	if (!Game)
	{
		return false;
	}
	if (FnPark(Game, &ParkDesc) != 1 || ParkDesc.n_wall < 3)
	{
		UE_LOG(LogBbSim, Error, TEXT("The sim gave no park"));
		return false;
	}
	// Who throws left: the rosters never change during a game, so read them once here.
	for (uint32 Side = 0; Side < 2; ++Side)
	{
		for (uint32 Slot = 0; Slot < 9; ++Slot)
		{
			BbPlayer P{};
			if (FnRoster(Game, Side, Slot, &P) != 1)
			{
				continue;
			}
			if (P.throws == 1)
			{
				LeftHanded.Add(P.bat_agent);
				LeftHanded.Add(P.field_agent);
			}
			const FString Name = UTF8_TO_TCHAR(P.family);
			Family.Add(P.bat_agent, Name);
			Family.Add(P.field_agent, Name);
			Players.Add(P.bat_agent, P);
			Players.Add(P.field_agent, P);
		}
	}
	return true;
}

FBbPlay FBbSim::Next()
{
	FBbPlay Play;
	FScopeLock Lock(&GameLock);
	if (Game && FnAdvance(Game, &Play.Blob, &Play.Len) == 1)
	{
		Play.bValid = FnInfo(Play.Blob, Play.Len, &Play.Info) == 1;
	}
	return Play;
}

FBbPlay FBbSim::LoadTimeline(const TArray<uint8>& Bytes) const
{
	FBbPlay Play;
	if (!FnInfo || Bytes.IsEmpty())
	{
		return Play;
	}
	Play.Storage = MakeShared<TArray<uint8>>(Bytes);
	Play.Blob = Play.Storage->GetData();
	Play.Len = static_cast<size_t>(Play.Storage->Num());
	Play.bSimOwned = false;
	Play.bValid = FnInfo(Play.Blob, Play.Len, &Play.Info) == 1;
	if (!Play.bValid)
	{
		Play = FBbPlay();
	}
	return Play;
}

bool FBbSim::BoxScore(uint32 ThroughPlays, BbBoxScore& Out) const
{
	if (!Game || !FnBoxScore)
	{
		return false;
	}
	FScopeLock Lock(&GameLock);
	return FnBoxScore(Game, ThroughPlays, &Out) == 1;
}

bool FBbSim::Sample(const FBbPlay& Play, float T, BbFrame& Out) const
{
	return Play.bValid && FnSample(Play.Blob, Play.Len, T, &Out) == 1;
}

void FBbSim::Release(FBbPlay& Play) const
{
	if (Play.Blob && Play.bSimOwned && FnBlobFree)
	{
		FnBlobFree(Play.Blob, Play.Len);
	}
	Play = FBbPlay();
}
