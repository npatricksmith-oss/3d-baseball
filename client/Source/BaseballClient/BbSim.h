// Run-time binding to the Rust sim core's C ABI (libbb_ffi.so).
#pragma once

#include "CoreMinimal.h"
#include "bb_ffi.h"
#include "HAL/CriticalSection.h"

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
	/** The box score of the pitches shown so far (the first ThroughPlays of the game), read by
	 *  the sim from its event log when asked. False if the library has no box score or the log
	 *  cannot be read. The worker runs a pitch ahead, which is why the count is passed: the
	 *  screen shows only what has happened on it. Safe beside a worker running Next(): it waits
	 *  for the pitch in hand. */
	bool BoxScore(uint32 ThroughPlays, BbBoxScore& Out) const;
	/** True if the player behind this agent id (batting or fielding) throws left-handed.
	 *  Read once at the start of the game, so it is safe on any thread. */
	bool ThrowsLeft(uint16 Agent) const { return LeftHanded.Contains(Agent); }
	/** The family name of the player behind this agent id (either kind), or empty. Read once
	 *  at the start of the game like ThrowsLeft. */
	FString FamilyName(uint16 Agent) const { const FString* N = Family.Find(Agent); return N ? *N : FString(); }

private:
	void* Handle = nullptr;
	BbGame* Game = nullptr;
	BbGame* (*FnCreate)(uint64) = nullptr;
	int32 (*FnAdvance)(BbGame*, uint8**, size_t*) = nullptr;
	void (*FnBlobFree)(uint8*, size_t) = nullptr;
	void (*FnGameFree)(BbGame*) = nullptr;
	int32 (*FnInfo)(const uint8*, size_t, BbPlayInfo*) = nullptr;
	int32 (*FnSample)(const uint8*, size_t, float, BbFrame*) = nullptr;
	int32 (*FnRoster)(const BbGame*, uint32, uint32, BbPlayer*) = nullptr;
	int32 (*FnBoxScore)(const BbGame*, uint32, BbBoxScore*) = nullptr;
	/** Held while the game is stepped or read for the box score: the library's game handle is
	 *  not shared between threads. */
	mutable FCriticalSection GameLock;
	/** Both agent ids of every left-handed thrower. */
	TSet<uint16> LeftHanded;
	/** Family names by both agent ids. */
	TMap<uint16, FString> Family;
};
