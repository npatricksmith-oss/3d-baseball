// Run-time binding to the Rust sim core's C ABI (libbb_ffi.so).
#pragma once

#include "CoreMinimal.h"
#include "bb_ffi.h"

/** One pitch's timeline blob, owned by the sim library: release it exactly once (FBbSim::Release). */
struct FBbPlay
{
	uint8* Blob = nullptr;
	size_t Len = 0;
	BbPlayInfo Info{};
	bool bValid = false;
};

class FBbSim
{
public:
	~FBbSim();
	/** Load the library and start a game. False (with an error logged) if anything is missing. */
	bool Start(const FString& LibraryPath, uint64 Seed);
	/** Simulate the next pitch. Not valid once the game is over. Safe to call off the game thread. */
	FBbPlay Next();
	bool Sample(const FBbPlay& Play, float T, BbFrame& Out) const;
	void Release(FBbPlay& Play) const;

private:
	void* Handle = nullptr;
	BbGame* Game = nullptr;
	BbGame* (*FnCreate)(uint64) = nullptr;
	int32 (*FnAdvance)(BbGame*, uint8**, size_t*) = nullptr;
	void (*FnBlobFree)(uint8*, size_t) = nullptr;
	void (*FnGameFree)(BbGame*) = nullptr;
	int32 (*FnInfo)(const uint8*, size_t, BbPlayInfo*) = nullptr;
	int32 (*FnSample)(const uint8*, size_t, float, BbFrame*) = nullptr;
};
