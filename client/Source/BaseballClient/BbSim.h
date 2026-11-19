// Run-time binding to the Rust sim core's C ABI (libbb_ffi.so).
#pragma once

#include "CoreMinimal.h"
#include "bb_ffi.h"
#include "HAL/CriticalSection.h"

/** One pitch's timeline blob, owned by the sim library (bSimOwned) or by Storage; release it
 *  with FBbSim::Release. */
struct FBbPlay
{
	uint8* Blob = nullptr;
	size_t Len = 0;
	BbPlayInfo Info{};
	bool bValid = false;
	bool bSimOwned = true;
	TSharedPtr<TArray<uint8>> Storage;
};

class FBbSim
{
public:
	~FBbSim();
	/** Load the library and start a game. False (with an error logged) if anything is missing. */
	bool Start(const FString& LibraryPath, uint64 Seed);
	/** Simulate the next pitch. Not valid once the game is over. Safe to call off the game thread. */
	FBbPlay Next();
	/** Build a playable FBbPlay from recorded FlatBuffer bytes (does not advance the sim). */
	FBbPlay LoadTimeline(const TArray<uint8>& Bytes) const;
	/** LoadTimeline plus the checks a recording must pass before it is played: the summary is
	 *  finite with a duration inside (0, MaxRecordedPlayS], and frames sampled across the whole
	 *  play (the first and the last included) all read back with a finite ball and between one
	 *  and BB_MAX_ACTORS finite actors. An empty or half-empty timeline, or one whose duration
	 *  would stall playback, comes back invalid with Why saying what is wrong. */
	FBbPlay LoadRecorded(const TArray<uint8>& Bytes, FString& Why) const;
	/** The summary checks of LoadRecorded alone (testable on a made-up summary). */
	static bool InfoSane(const BbPlayInfo& Info, FString& Why);
	/** The longest recorded play accepted, s. A real pitch with a home run's trot is under a minute. */
	static constexpr float MaxRecordedPlayS = 120.f;
	bool Sample(const FBbPlay& Play, float T, BbFrame& Out) const;
	void Release(FBbPlay& Play) const;
	/** Every player of both sides once, by side and batting order. */
	TArray<BbPlayer> AllPlayers() const;
	/** For viewing a saved play (D9): replaces the rosters and, when Park is given, the park
	 *  with what the save recorded, so the play is shown with the names, numbers, hands and
	 *  walls it was seen with, not today's. Call before anything reads them. */
	void UseRecorded(const TArray<BbPlayer>& Roster, const BbPark* RecordedPark);
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
	/** Who is behind this agent id (either kind), or null. Read once at the start like ThrowsLeft. */
	const BbPlayer* Player(uint16 Agent) const { return Players.Find(Agent); }
	/** The park the game is played in, as the physics has it. Read once at the start. */
	const BbPark& Park() const { return ParkDesc; }

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
	/** The players by both agent ids. */
	TMap<uint16, BbPlayer> Players;
	int32 (*FnPark)(const BbGame*, BbPark*) = nullptr;
	BbPark ParkDesc{};
};
