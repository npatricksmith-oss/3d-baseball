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
	if (!FnCreate || !FnAdvance || !FnBlobFree || !FnGameFree || !FnInfo || !FnSample)
	{
		UE_LOG(LogBbSim, Error, TEXT("The sim library at %s is missing part of the C ABI"), *LibraryPath);
		return false;
	}
	Game = FnCreate(Seed);
	return Game != nullptr;
}

FBbPlay FBbSim::Next()
{
	FBbPlay Play;
	if (Game && FnAdvance(Game, &Play.Blob, &Play.Len) == 1)
	{
		Play.bValid = FnInfo(Play.Blob, Play.Len, &Play.Info) == 1;
	}
	return Play;
}

bool FBbSim::Sample(const FBbPlay& Play, float T, BbFrame& Out) const
{
	return Play.bValid && FnSample(Play.Blob, Play.Len, T, &Out) == 1;
}

void FBbSim::Release(FBbPlay& Play) const
{
	if (Play.Blob)
	{
		FnBlobFree(Play.Blob, Play.Len);
	}
	Play = FBbPlay();
}
