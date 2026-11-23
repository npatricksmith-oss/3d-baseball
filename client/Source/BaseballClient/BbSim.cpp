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
	// The structs the client reads must have the layout it was compiled with (Q39 review P3-5):
	// a library of another ABI version would write past them.
	using FAbiVersion = uint32 (*)();
	const FAbiVersion FnAbi = reinterpret_cast<FAbiVersion>(Sym(TEXT("bb_abi_version")));
	const uint32 LibAbi = FnAbi ? FnAbi() : 1;
	if (LibAbi != BB_ABI_VERSION)
	{
		UE_LOG(LogBbSim, Error, TEXT("The sim library at %s speaks C ABI version %u; this client was built for version %u. Rebuild both from one checkout (client/run.sh does: cargo build --release -p bb_ffi, then the client)."), *LibraryPath, LibAbi, (uint32)BB_ABI_VERSION);
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

bool FBbSim::InfoSane(const BbPlayInfo& Info, FString& Why)
{
	auto Finite = [](float V) { return FMath::IsFinite(V); };
	if (!Finite(Info.duration_s) || Info.duration_s <= 0.f || Info.duration_s > MaxRecordedPlayS)
	{
		Why = FString::Printf(TEXT("its duration is %g s (must be above 0 and at most %g s)"), Info.duration_s, MaxRecordedPlayS);
		return false;
	}
	if (!Finite(Info.release_speed_mps) || !Finite(Info.exit_velocity_mps) || !Finite(Info.launch_angle_rad)
		|| !Finite(Info.contact_s) || !Finite(Info.call_s) || !Finite(Info.dead_s)
		|| !Finite(Info.projected_distance_m) || !Finite(Info.plate_x_m) || !Finite(Info.plate_z_m)
		|| !Finite(Info.zone_bottom_m) || !Finite(Info.zone_top_m) || !Finite(Info.zone_half_width_m))
	{
		Why = TEXT("its summary holds a value that is not a number");
		return false;
	}
	if (Info.contact_s > Info.duration_s || Info.call_s > Info.duration_s + 1.f || Info.dead_s > Info.duration_s + 1.f)
	{
		Why = TEXT("its contact, call or dead time lies beyond its end");
		return false;
	}
	if (Info.n_calls > BB_MAX_CALLS)
	{
		Why = FString::Printf(TEXT("it lists %u umpire calls (at most %d)"), Info.n_calls, BB_MAX_CALLS);
		return false;
	}
	for (uint32 I = 0; I < Info.n_calls; ++I)
	{
		if (!Finite(Info.calls[I].t_s) || !Finite(Info.calls[I].margin_s))
		{
			Why = TEXT("an umpire call holds a value that is not a number");
			return false;
		}
	}
	return true;
}

FBbPlay FBbSim::LoadRecorded(const TArray<uint8>& Bytes, FString& Why) const
{
	Why.Reset();
	if (Bytes.IsEmpty())
	{
		Why = TEXT("the file is empty");
		return FBbPlay();
	}
	FBbPlay Play = LoadTimeline(Bytes);
	if (!Play.bValid)
	{
		Why = TEXT("it is not a readable BBTL timeline");
		return FBbPlay();
	}
	if (!InfoSane(Play.Info, Why))
	{
		FBbPlay Bad = MoveTemp(Play);
		Release(Bad);
		return FBbPlay();
	}
	// Frames across the whole play, the ends included (a fixed number, so a long play costs the
	// same as a short one): each must read back and hold something sane.
	constexpr int32 Samples = 48;
	for (int32 K = 0; K <= Samples; ++K)
	{
		const float T = Play.Info.duration_s * static_cast<float>(K) / static_cast<float>(Samples);
		BbFrame F{};
		bool bOk = Sample(Play, T, F);
		if (bOk)
		{
			bOk = F.n_actors >= 1 && F.n_actors <= BB_MAX_ACTORS && F.n_dives <= BB_MAX_DIVES;
		}
		if (bOk)
		{
			bOk = FMath::IsFinite(F.ball[0]) && FMath::IsFinite(F.ball[1]) && FMath::IsFinite(F.ball[2])
				&& FMath::Abs(F.ball[0]) < 2000.f && FMath::Abs(F.ball[1]) < 2000.f && FMath::Abs(F.ball[2]) < 2000.f;
		}
		for (uint32 I = 0; bOk && I < F.n_actors; ++I)
		{
			const BbActor& A = F.actors[I];
			bOk = FMath::IsFinite(A.pos[0]) && FMath::IsFinite(A.pos[1]) && FMath::IsFinite(A.pos[2]) && FMath::IsFinite(A.yaw)
				&& FMath::Abs(A.pos[0]) < 2000.f && FMath::Abs(A.pos[1]) < 2000.f && FMath::Abs(A.pos[2]) < 2000.f;
		}
		if (!bOk)
		{
			Why = FString::Printf(TEXT("its frame at %.2f s is missing or not sane (no actors, or a position that is not a number)"), T);
			FBbPlay Bad = MoveTemp(Play);
			Release(Bad);
			return FBbPlay();
		}
	}
	return Play;
}

TArray<BbPlayer> FBbSim::AllPlayers() const
{
	TArray<BbPlayer> Out;
	for (const TPair<uint16, BbPlayer>& Pair : Players)
	{
		// Each player is in the map under both of his ids: keep the batting one.
		if (Pair.Key == Pair.Value.bat_agent)
		{
			Out.Add(Pair.Value);
		}
	}
	Out.Sort([](const BbPlayer& A, const BbPlayer& B) { return A.bat_agent < B.bat_agent; });
	return Out;
}

void FBbSim::UseRecorded(const TArray<BbPlayer>& Roster, const BbPark* RecordedPark)
{
	LeftHanded.Reset();
	Family.Reset();
	Players.Reset();
	for (const BbPlayer& P : Roster)
	{
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
	if (RecordedPark)
	{
		ParkDesc = *RecordedPark;
	}
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
