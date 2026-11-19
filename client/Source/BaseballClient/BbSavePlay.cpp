#include "BbSavePlay.h"

#include "HAL/FileManager.h"
#include "Misc/DateTime.h"
#include "Misc/FileHelper.h"
#include "Misc/Paths.h"
#include "Misc/SecureHash.h"
#include "Styling/CoreStyle.h"
#include "Widgets/Input/SButton.h"
#include "Widgets/Input/SMultiLineEditableTextBox.h"
#include "Widgets/Layout/SBorder.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SSpacer.h"
#include "Widgets/SBoxPanel.h"
#include "Widgets/Text/STextBlock.h"

// BB_BUILD_COMMIT: written by Build/stamp-commit.sh before every build (a pre-build step of
// both targets), so it is the commit and dirtiness the binary is built from, never a stale one.
#include "BbBuildStamp.h"

namespace
{
	FString UnquoteBasic(const FString& S)
	{
		FString Out;
		Out.Reserve(S.Len());
		for (int32 I = 0; I < S.Len(); ++I)
		{
			if (S[I] == TEXT('\\') && I + 1 < S.Len())
			{
				const TCHAR N = S[++I];
				switch (N)
				{
				case TEXT('\\'): Out.AppendChar(TEXT('\\')); break;
				case TEXT('"'): Out.AppendChar(TEXT('"')); break;
				case TEXT('n'): Out.AppendChar(TEXT('\n')); break;
				case TEXT('r'): Out.AppendChar(TEXT('\r')); break;
				case TEXT('t'): Out.AppendChar(TEXT('\t')); break;
				default: Out.AppendChar(N); break;
				}
			}
			else
			{
				Out.AppendChar(S[I]);
			}
		}
		return Out;
	}

	FString ParseStringValue(const FString& Val)
	{
		FString S = Val.TrimStartAndEnd();
		if (S.Len() >= 2 && S[0] == TEXT('"') && S[S.Len() - 1] == TEXT('"'))
		{
			return UnquoteBasic(S.Mid(1, S.Len() - 2));
		}
		return S;
	}

	/** A float written so that reading it back gives the same float. */
	FString F(double V)
	{
		return FString::Printf(TEXT("%.9g"), V);
	}

	/** Text safe inside a "key=value;key=value" string of the TOML. */
	FString Plain(const FString& S)
	{
		FString Out = S;
		for (TCHAR& C : Out)
		{
			if (C == TEXT(';') || C == TEXT('"') || C == TEXT('\\') || C == TEXT('=') || C < 32)
			{
				C = TEXT(' ');
			}
		}
		return Out;
	}

	/** "k=v;k=v" into a map (keys and values trimmed). */
	TMap<FString, FString> ParseFields(const FString& S)
	{
		TMap<FString, FString> Out;
		TArray<FString> Parts;
		S.ParseIntoArray(Parts, TEXT(";"), true);
		for (const FString& Part : Parts)
		{
			FString K, V;
			if (Part.Split(TEXT("="), &K, &V))
			{
				Out.Add(K.TrimStartAndEnd(), V.TrimStartAndEnd());
			}
		}
		return Out;
	}

	/** "1.5,2,3" into numbers. */
	TArray<double> ParseNumbers(const FString& S, const TCHAR* Delim = TEXT(","))
	{
		TArray<FString> Parts;
		S.ParseIntoArray(Parts, Delim, true);
		TArray<double> Out;
		for (const FString& P : Parts)
		{
			Out.Add(FCString::Atod(*P.TrimStartAndEnd()));
		}
		return Out;
	}

	FString RunsList(const TArray<uint8>& Runs)
	{
		FString Out;
		for (int32 I = 0; I < Runs.Num(); ++I)
		{
			Out += (I ? TEXT(",") : TEXT("")) + FString::FromInt(Runs[I]);
		}
		return Out;
	}

	FString BoardText(const BbSavePlay::FBoard& B)
	{
		return FString::Printf(TEXT("first=%d;away=%s;home=%s;hits=%d,%d"), B.FirstInning, *RunsList(B.Runs[0]), *RunsList(B.Runs[1]), B.Hits[0], B.Hits[1]);
	}

	bool ParseBoard(const FString& S, BbSavePlay::FBoard& Out)
	{
		const TMap<FString, FString> M = ParseFields(S);
		const FString* First = M.Find(TEXT("first"));
		const FString* Away = M.Find(TEXT("away"));
		const FString* Home = M.Find(TEXT("home"));
		const FString* Hits = M.Find(TEXT("hits"));
		if (!First || !Away || !Home || !Hits)
		{
			return false;
		}
		const TArray<double> H = ParseNumbers(*Hits);
		if (H.Num() != 2)
		{
			return false;
		}
		Out = BbSavePlay::FBoard();
		Out.FirstInning = FMath::Max(1, FCString::Atoi(**First));
		Out.Hits[0] = FMath::Clamp((int32)H[0], 0, 65535);
		Out.Hits[1] = FMath::Clamp((int32)H[1], 0, 65535);
		for (int32 T = 0; T < 2; ++T)
		{
			for (double R : ParseNumbers(T == 0 ? *Away : *Home))
			{
				Out.Runs[T].Add((uint8)FMath::Clamp((int32)R, 0, 254));
			}
		}
		Out.bValid = true;
		return true;
	}

	FString PlayerText(const BbPlayer& P)
	{
		return FString::Printf(TEXT("id=%u;bat=%u;field=%u;number=%u;pos=%u;bats=%u;throws=%u;given=%s;family=%s"),
			P.player_id, P.bat_agent, P.field_agent, P.number, P.position, P.bats, P.throws,
			*Plain(UTF8_TO_TCHAR(P.given)), *Plain(UTF8_TO_TCHAR(P.family)));
	}

	bool ParsePlayer(const FString& S, uint32 Side, uint32 Order, BbPlayer& Out)
	{
		const TMap<FString, FString> M = ParseFields(S);
		auto Num = [&M](const TCHAR* Key, int32& V) { const FString* P = M.Find(Key); if (P) { V = FCString::Atoi(**P); } return P != nullptr; };
		int32 Id = 0, Bat = 0, Field = 0, Number = 0, Pos = 0, Bats = 0, Throws = 0;
		if (!Num(TEXT("id"), Id) || !Num(TEXT("bat"), Bat) || !Num(TEXT("field"), Field) || !Num(TEXT("number"), Number)
			|| !Num(TEXT("pos"), Pos) || !Num(TEXT("bats"), Bats) || !Num(TEXT("throws"), Throws))
		{
			return false;
		}
		const FString* Given = M.Find(TEXT("given"));
		const FString* Family = M.Find(TEXT("family"));
		if (!Given || !Family)
		{
			return false;
		}
		FMemory::Memzero(Out);
		Out.player_id = (uint32)Id;
		Out.bat_agent = (uint16)Bat;
		Out.field_agent = (uint16)Field;
		Out.side = (uint8)Side;
		Out.number = (uint8)Number;
		Out.position = (uint8)Pos;
		Out.batting_order = (uint8)Order;
		Out.bats = (uint8)Bats;
		Out.throws = (uint8)Throws;
		FCStringAnsi::Strncpy(Out.given, TCHAR_TO_UTF8(**Given), BB_NAME_LEN);
		FCStringAnsi::Strncpy(Out.family, TCHAR_TO_UTF8(**Family), BB_NAME_LEN);
		return true;
	}

	FString ParkText(const BbPark& D)
	{
		FString Wall;
		for (uint32 I = 0; I < D.n_wall; ++I)
		{
			const BbWallPiece& W = D.wall[I];
			Wall += FString::Printf(TEXT("%s%s,%s,%s,%u"), I ? TEXT("/") : TEXT(""), *F(W.from[0]), *F(W.from[1]), *F(W.top_m), W.kind);
		}
		return FString::Printf(TEXT("fence=%s;foul=%s;backstop=%s;dugout=%s;rubber=%s;dirt=%s;poles=%s,%s,%s,%s;bases=%s,%s,%s,%s,%s,%s;wall=%s"),
			*F(D.fence_height_m), *F(D.foul_wall_height_m), *F(D.backstop_height_m), *F(D.dugout_roof_m), *F(D.rubber_m), *F(D.dirt_radius_m),
			*F(D.poles[0][0]), *F(D.poles[0][1]), *F(D.poles[1][0]), *F(D.poles[1][1]),
			*F(D.bases[0][0]), *F(D.bases[0][1]), *F(D.bases[1][0]), *F(D.bases[1][1]), *F(D.bases[2][0]), *F(D.bases[2][1]), *Wall);
	}

	bool ParsePark(const FString& S, BbPark& Out)
	{
		const TMap<FString, FString> M = ParseFields(S);
		const FString* Fence = M.Find(TEXT("fence"));
		const FString* Foul = M.Find(TEXT("foul"));
		const FString* Back = M.Find(TEXT("backstop"));
		const FString* Dugout = M.Find(TEXT("dugout"));
		const FString* Rubber = M.Find(TEXT("rubber"));
		const FString* Dirt = M.Find(TEXT("dirt"));
		const FString* Poles = M.Find(TEXT("poles"));
		const FString* Bases = M.Find(TEXT("bases"));
		const FString* Wall = M.Find(TEXT("wall"));
		if (!Fence || !Foul || !Back || !Dugout || !Rubber || !Dirt || !Poles || !Bases || !Wall)
		{
			return false;
		}
		const TArray<double> P = ParseNumbers(*Poles), B = ParseNumbers(*Bases);
		TArray<FString> Pieces;
		Wall->ParseIntoArray(Pieces, TEXT("/"), true);
		if (P.Num() != 4 || B.Num() != 6 || Pieces.Num() < 3 || Pieces.Num() > BB_PARK_MAX_WALL)
		{
			return false;
		}
		FMemory::Memzero(Out);
		Out.n_wall = (uint32)Pieces.Num();
		for (int32 I = 0; I < Pieces.Num(); ++I)
		{
			const TArray<double> W = ParseNumbers(Pieces[I]);
			if (W.Num() != 4)
			{
				return false;
			}
			Out.wall[I].from[0] = (float)W[0];
			Out.wall[I].from[1] = (float)W[1];
			Out.wall[I].top_m = (float)W[2];
			Out.wall[I].kind = (uint8)W[3];
		}
		for (int32 I = 0; I < 4; ++I)
		{
			Out.poles[I / 2][I % 2] = (float)P[I];
		}
		for (int32 I = 0; I < 6; ++I)
		{
			Out.bases[I / 2][I % 2] = (float)B[I];
		}
		Out.fence_height_m = (float)FCString::Atod(**Fence);
		Out.foul_wall_height_m = (float)FCString::Atod(**Foul);
		Out.backstop_height_m = (float)FCString::Atod(**Back);
		Out.dugout_roof_m = (float)FCString::Atod(**Dugout);
		Out.rubber_m = (float)FCString::Atod(**Rubber);
		Out.dirt_radius_m = (float)FCString::Atod(**Dirt);
		return true;
	}
}

namespace BbSavePlay
{
	FString BuildCommit()
	{
		return FString(ANSI_TO_TCHAR(BB_BUILD_COMMIT));
	}

	FString Folder()
	{
		return FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT(".."), TEXT("saved-plays")));
	}

	FString ResolveTomlPath(const FString& Arg)
	{
		if (Arg.IsEmpty())
		{
			return FString();
		}
		if (FPaths::FileExists(Arg))
		{
			return FPaths::ConvertRelativePathToFull(Arg);
		}
		const FString FromRepo = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT(".."), Arg));
		if (FPaths::FileExists(FromRepo))
		{
			return FromRepo;
		}
		const FString FromFolder = FPaths::Combine(Folder(), Arg);
		if (FPaths::FileExists(FromFolder))
		{
			return FromFolder;
		}
		return Arg;
	}

	FString WriteTo(const FString& Dir, const FWhat& What, const FContext& Context, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen)
	{
		if (!TimelineBlob || TimelineLen == 0)
		{
			return FString();
		}
		const FDateTime Now = FDateTime::Now();
		IFileManager::Get().MakeDirectory(*Dir, true);
		const FString Stem = FString::Printf(TEXT("seed%llu-play%u-%s"), What.Seed, What.Play, *Now.ToString(TEXT("%Y%m%d-%H%M%S")));
		FString Base = Stem;
		for (int32 Suffix = 2; IFileManager::Get().FileExists(*FPaths::Combine(Dir, Base + TEXT(".toml"))) || IFileManager::Get().FileExists(*FPaths::Combine(Dir, Base + TEXT(".timeline"))); ++Suffix)
		{
			Base = FString::Printf(TEXT("%s-%d"), *Stem, Suffix);
		}
		const FString TimelineName = Base + TEXT(".timeline");
		const FString TimelinePath = FPaths::Combine(Dir, TimelineName);
		const FString TomlName = Base + TEXT(".toml");
		const FString Path = FPaths::Combine(Dir, TomlName);
		if (!FFileHelper::SaveArrayToFile(TArrayView<const uint8>(TimelineBlob, static_cast<int32>(TimelineLen)), *TimelinePath))
		{
			return FString();
		}
		const FString Commit = BuildCommit();
		FString SeeArg = FPaths::IsSamePath(Dir, Folder()) ? FString::Printf(TEXT("saved-plays/%s"), *TomlName) : Path;
		if (SeeArg.Contains(TEXT(" ")))
		{
			SeeArg = TEXT("\"") + SeeArg + TEXT("\"");
		}
		// TOML. The comment is a multi-line basic string: backslashes and quotes escaped.
		FString Body = Comment.Replace(TEXT("\r"), TEXT(""));
		Body = Body.Replace(TEXT("\\"), TEXT("\\\\")).Replace(TEXT("\""), TEXT("\\\""));
		FString Text = FString::Printf(
			TEXT("# A play saved from the instant replay (client BbSavePlay).\n")
			TEXT("# See it again: client/run.sh -BbSavedPlay=%s\n")
			TEXT("seed = %llu\n")
			TEXT("play = %u\n")
			TEXT("commit = \"%s\"\n")
			TEXT("timeline = \"%s\"\n")
			TEXT("timeline_md5 = \"%s\"\n")
			TEXT("inning = %u\n")
			TEXT("half = \"%s\"\n")
			TEXT("# The board before the pitch.\n")
			TEXT("outs = %u\n")
			TEXT("count = \"%u-%u\"\n")
			TEXT("score = \"away %u, home %u\"\n"),
			*SeeArg, What.Seed, What.Play, *Commit, *TimelineName, *FMD5::HashBytes(TimelineBlob, TimelineLen), What.Inning, What.bTop ? TEXT("top") : TEXT("bottom"),
			What.Outs, What.Balls, What.Strikes, What.AwayRuns, What.HomeRuns);
		// What the viewer needs to show the play as it was seen (D9): the runners on, the
		// board's line score and hits (before the pitch and after the play), the players and
		// the park, all as the saving build had them.
		if (What.bHasBases)
		{
			Text += FString::Printf(TEXT("bases = %u\n"), What.Bases);
		}
		if (What.BoardBefore.bValid)
		{
			Text += FString::Printf(TEXT("board_before = \"%s\"\n"), *BoardText(What.BoardBefore));
		}
		if (What.BoardAfter.bValid)
		{
			Text += FString::Printf(TEXT("board_after = \"%s\"\n"), *BoardText(What.BoardAfter));
		}
		if (Context.HasRoster())
		{
			Text += TEXT("# The players as this play was seen: key = side and batting order.\n");
			for (const BbPlayer& P : Context.Roster)
			{
				Text += FString::Printf(TEXT("%s_%u = \"%s\"\n"), P.side == 0 ? TEXT("away") : TEXT("home"), P.batting_order, *PlayerText(P));
			}
		}
		if (Context.bHasPark)
		{
			Text += FString::Printf(TEXT("# The park (metres, sim frame).\npark = \"%s\"\n"), *ParkText(Context.Park));
		}
		Text += FString::Printf(TEXT("saved = %s\ncomment = \"\"\"\n%s\n\"\"\"\n"), *Now.ToString(TEXT("%Y-%m-%dT%H:%M:%S")), *Body);
		if (!FFileHelper::SaveStringToFile(Text, *Path, FFileHelper::EEncodingOptions::ForceUTF8WithoutBOM))
		{
			IFileManager::Get().Delete(*TimelinePath);
			return FString();
		}
		return Path;
	}

	FString Write(const FWhat& What, const FContext& Context, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen)
	{
		return WriteTo(Folder(), What, Context, Comment, TimelineBlob, TimelineLen);
	}

	FContext ContextOf(const FBbSim& Sim)
	{
		FContext C;
		C.Roster = Sim.AllPlayers();
		C.bHasPark = true;
		C.Park = Sim.Park();
		return C;
	}

	bool Read(const FString& TomlPath, FLoaded& Out)
	{
		Out = FLoaded();
		FString Text;
		if (!FFileHelper::LoadFileToString(Text, *TomlPath))
		{
			return false;
		}
		Text = Text.Replace(TEXT("\r\n"), TEXT("\n")).Replace(TEXT("\r"), TEXT("\n"));
		TArray<FString> Lines;
		Text.ParseIntoArray(Lines, TEXT("\n"), false);
		bool bInMultiline = false;
		FString MultilineKey;
		FString MultilineRaw;
		for (const FString& RawLine : Lines)
		{
			if (bInMultiline)
			{
				const int32 CloseIdx = RawLine.Find(TEXT("\"\"\""));
				if (CloseIdx != INDEX_NONE && (CloseIdx == 0 || RawLine[CloseIdx - 1] != TEXT('\\')))
				{
					MultilineRaw += RawLine.Left(CloseIdx);
					if (MultilineKey == TEXT("comment"))
					{
						if (MultilineRaw.EndsWith(TEXT("\n")))
						{
							MultilineRaw.LeftChopInline(1);
						}
						Out.Comment = UnquoteBasic(MultilineRaw);
					}
					bInMultiline = false;
					MultilineKey.Reset();
					MultilineRaw.Reset();
				}
				else
				{
					MultilineRaw += RawLine + TEXT("\n");
				}
				continue;
			}
			const FString Trimmed = RawLine.TrimStartAndEnd();
			if (Trimmed.IsEmpty() || Trimmed.StartsWith(TEXT("#")))
			{
				continue;
			}
			const int32 Eq = Trimmed.Find(TEXT("="));
			if (Eq == INDEX_NONE)
			{
				continue;
			}
			const FString Key = Trimmed.Left(Eq).TrimStartAndEnd();
			const FString Val = Trimmed.RightChop(Eq + 1).TrimStartAndEnd();
			if (Val.StartsWith(TEXT("\"\"\"")))
			{
				const FString Rest = Val.RightChop(3);
				const int32 CloseIdx = Rest.Find(TEXT("\"\"\""));
				if (CloseIdx != INDEX_NONE)
				{
					if (Key == TEXT("comment"))
					{
						Out.Comment = UnquoteBasic(Rest.Left(CloseIdx));
					}
				}
				else
				{
					bInMultiline = true;
					MultilineKey = Key;
					MultilineRaw = Rest.IsEmpty() ? FString() : (Rest + TEXT("\n"));
				}
				continue;
			}
			if (Key == TEXT("seed"))
			{
				Out.What.Seed = FCString::Strtoui64(*Val, nullptr, 10);
				Out.bHasSeed = true;
			}
			else if (Key == TEXT("play"))
			{
				Out.What.Play = (uint32)FCString::Atoi(*Val);
				Out.bHasPlay = true;
			}
			else if (Key == TEXT("commit"))
			{
				Out.Commit = ParseStringValue(Val);
			}
			else if (Key == TEXT("commit_inferred"))
			{
				Out.bCommitInferred = Val == TEXT("true");
			}
			else if (Key == TEXT("timeline"))
			{
				Out.Timeline = ParseStringValue(Val);
			}
			else if (Key == TEXT("inning"))
			{
				Out.What.Inning = (uint8)FCString::Atoi(*Val);
			}
			else if (Key == TEXT("half"))
			{
				Out.What.bTop = ParseStringValue(Val) != TEXT("bottom");
			}
			else if (Key == TEXT("outs"))
			{
				Out.What.Outs = (uint8)FCString::Atoi(*Val);
			}
			else if (Key == TEXT("count"))
			{
				FString B, S;
				if (ParseStringValue(Val).Split(TEXT("-"), &B, &S))
				{
					Out.What.Balls = (uint8)FCString::Atoi(*B);
					Out.What.Strikes = (uint8)FCString::Atoi(*S);
				}
			}
			else if (Key == TEXT("score"))
			{
				FString S = ParseStringValue(Val);
				FString Left, Right;
				if (S.Split(TEXT(","), &Left, &Right))
				{
					Left = Left.TrimStartAndEnd();
					Right = Right.TrimStartAndEnd();
					if (Left.StartsWith(TEXT("away ")))
					{
						Out.What.AwayRuns = (uint8)FCString::Atoi(*Left.RightChop(5));
					}
					if (Right.StartsWith(TEXT("home ")))
					{
						Out.What.HomeRuns = (uint8)FCString::Atoi(*Right.RightChop(5));
					}
				}
			}
			else if (Key == TEXT("saved"))
			{
				Out.Saved = ParseStringValue(Val);
			}
			else if (Key == TEXT("comment"))
			{
				Out.Comment = ParseStringValue(Val);
			}
			else if (Key == TEXT("timeline_md5"))
			{
				Out.TimelineMd5 = ParseStringValue(Val).ToLower();
			}
			else if (Key == TEXT("bases"))
			{
				Out.What.bHasBases = true;
				Out.What.Bases = (uint8)(FCString::Atoi(*Val) & 7);
			}
			else if (Key == TEXT("board_before"))
			{
				ParseBoard(ParseStringValue(Val), Out.What.BoardBefore);
			}
			else if (Key == TEXT("board_after"))
			{
				ParseBoard(ParseStringValue(Val), Out.What.BoardAfter);
			}
			else if (Key == TEXT("park"))
			{
				Out.Context.bHasPark = ParsePark(ParseStringValue(Val), Out.Context.Park);
			}
			else if ((Key.StartsWith(TEXT("away_")) || Key.StartsWith(TEXT("home_"))) && Key.Len() == 6 && FChar::IsDigit(Key[5]) && Key[5] != TEXT('0'))
			{
				BbPlayer P;
				if (ParsePlayer(ParseStringValue(Val), Key.StartsWith(TEXT("home_")) ? 1u : 0u, (uint32)(Key[5] - TEXT('0')), P))
				{
					Out.Context.Roster.Add(P);
				}
			}
		}
		if (!Out.Timeline.IsEmpty())
		{
			Out.TimelinePath = FPaths::IsRelative(Out.Timeline)
				? FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::GetPath(TomlPath), Out.Timeline))
				: Out.Timeline;
		}
		return true;
	}

	void BeginView(const FString& Arg, FView& Out)
	{
		Out = FView();
		if (Arg.TrimStartAndEnd().IsEmpty())
		{
			Out.ErrTitle = TEXT("NO SAVED PLAY GIVEN");
			Out.ErrBody = TEXT("-BbSavedPlay= has no path after the equals sign (an unset shell variable?)\nNo game is started.");
			UE_LOG(LogTemp, Error, TEXT("BB_SAVED_PLAY the -BbSavedPlay= argument is empty -- not starting a live game"));
			return;
		}
		Out.TomlPath = ResolveTomlPath(Arg);
		if (!Read(Out.TomlPath, Out.Meta))
		{
			Out.ErrTitle = TEXT("COULD NOT OPEN SAVED PLAY");
			Out.ErrBody = FString::Printf(TEXT("Could not read %s\nNot re-simulated."), *Out.TomlPath);
			UE_LOG(LogTemp, Error, TEXT("BB_SAVED_PLAY failed to read %s"), *Out.TomlPath);
			return;
		}
		Out.bRead = true;
		if (!Out.Meta.bHasSeed || !Out.Meta.bHasPlay)
		{
			Out.ErrTitle = TEXT("SAVED PLAY INCOMPLETE");
			Out.ErrBody = FString::Printf(TEXT("%s has no %s.\nNot re-simulated."), *Out.TomlPath, !Out.Meta.bHasSeed ? TEXT("seed") : TEXT("play number"));
			UE_LOG(LogTemp, Error, TEXT("BB_SAVED_PLAY %s lacks seed or play"), *Out.TomlPath);
			return;
		}
		// The board before the pitch; the runners are filled in by FinishView when the save does not say.
		const FWhat& W = Out.Meta.What;
		Out.Before.inning = W.Inning > 0 ? W.Inning : 1;
		Out.Before.top = W.bTop ? 1 : 0;
		Out.Before.outs = W.Outs;
		Out.Before.balls = W.Balls;
		Out.Before.strikes = W.Strikes;
		Out.Before.away_runs = W.AwayRuns;
		Out.Before.home_runs = W.HomeRuns;
		Out.Before.bases = W.bHasBases ? W.Bases : 0;
	}

	void FinishView(FBbSim& Sim, FView& Out)
	{
		Out.bOk = false;
		auto Fail = [&Out](const TCHAR* Title, const FString& Body)
		{
			Out.ErrTitle = Title;
			Out.ErrBody = Body;
			UE_LOG(LogTemp, Error, TEXT("BB_SAVED_PLAY %s: %s (%s) -- not re-simulated"), Title, *Body.Replace(TEXT("\n"), TEXT(" ")), *Out.TomlPath);
			return;
		};
		const FLoaded& M = Out.Meta;
		if (M.Timeline.IsEmpty())
		{
			const FString Need = M.Commit.IsEmpty()
				? FString(TEXT("the build that saved it"))
				: FString::Printf(TEXT("commit %s%s"), *M.Commit, M.bCommitInferred ? TEXT(" (inferred)") : TEXT(""));
			Fail(TEXT("NO RECORDED TIMELINE"), FString::Printf(TEXT("This save has no recording; it can be shown exactly only by %s.\nNot re-simulated."), *Need));
			return;
		}
		TArray<uint8> Bytes;
		if (!FFileHelper::LoadFileToArray(Bytes, *M.TimelinePath))
		{
			Fail(TEXT("TIMELINE FILE MISSING"), FString::Printf(TEXT("Could not read %s\nNot re-simulated."), *M.TimelinePath));
			return;
		}
		if (!M.TimelineMd5.IsEmpty())
		{
			const FString Got = FMD5::HashBytes(Bytes.GetData(), Bytes.Num());
			if (!Got.Equals(M.TimelineMd5, ESearchCase::IgnoreCase))
			{
				Fail(TEXT("TIMELINE DOES NOT MATCH ITS SAVE"), FString::Printf(TEXT("%s is not the file this save recorded (MD5 %s, expected %s): damaged, or from another play.\nNot re-simulated."), *M.TimelinePath, *Got, *M.TimelineMd5));
				return;
			}
		}
		FString Why;
		FBbPlay Loaded = Sim.LoadRecorded(Bytes, Why);
		if (!Loaded.bValid)
		{
			Fail(TEXT("TIMELINE UNUSABLE"), FString::Printf(TEXT("%s: %s.\nNot re-simulated."), *M.TimelinePath, *Why));
			return;
		}
		if (Loaded.Info.play_index != M.What.Play)
		{
			const uint32 Got = Loaded.Info.play_index;
			Sim.Release(Loaded);
			Fail(TEXT("TIMELINE IS ANOTHER PLAY"), FString::Printf(TEXT("%s is play %u, but this save is play %u.\nNot re-simulated."), *M.TimelinePath, Got, M.What.Play));
			return;
		}
		// The names, numbers, hands and park as the save's build had them. An older save has
		// none: the viewer's own are used, and the screen says so.
		TArray<FString> FromBuild;
		if (M.Context.HasRoster())
		{
			Sim.UseRecorded(M.Context.Roster, M.Context.bHasPark ? &M.Context.Park : nullptr);
		}
		else
		{
			FromBuild.Add(TEXT("names, numbers and throwing hands"));
		}
		if (!M.Context.bHasPark)
		{
			FromBuild.Add(TEXT("park"));
		}
		Out.CurrentBuildNote.Reset();
		if (FromBuild.Num() > 0)
		{
			Out.CurrentBuildNote = FString::Printf(TEXT("This save does not record its %s: they come from the current build and may differ from what was seen."), *FString::Join(FromBuild, TEXT(" or ")));
		}
		// The runners on before the pitch: recorded in a newer save; in an older one, who stands
		// near a base at release (a runner can lead off a good way, so the circle is wide).
		if (!M.What.bHasBases)
		{
			BbFrame F0;
			if (Sim.Sample(Loaded, 0.f, F0))
			{
				const double L = 27.432 / UE_SQRT_2;
				const FVector2D BaseAt[3] = { FVector2D(L, L), FVector2D(0.0, 2.0 * L), FVector2D(-L, L) };
				for (uint32 I = 0; I < F0.n_actors; ++I)
				{
					const BbActor& A = F0.actors[I];
					if ((A.agent & 0x80) != 0 || A.agent == Loaded.Info.batter)
					{
						continue;
					}
					const FVector2D P(A.pos[0], A.pos[1]);
					for (int32 B = 0; B < 3; ++B)
					{
						if ((P - BaseAt[B]).Size() < 8.0)
						{
							Out.Before.bases |= static_cast<uint8>(1u << B);
						}
					}
				}
			}
		}
		Out.Play = MoveTemp(Loaded);
		Out.bOk = true;
	}
}

void SBbSavePlayDialog::Construct(const FArguments& InArgs)
{
	OnCancel = InArgs._OnCancel;
	OnSave = InArgs._OnSave;
	Offset = InArgs._Offset;
	const FSlateFontInfo ButtonFont = FCoreStyle::GetDefaultFontStyle("Bold", 15);
	const FSlateFontInfo TitleFont = FCoreStyle::GetDefaultFontStyle("Bold", 18);
	const FSlateFontInfo LabelFont = FCoreStyle::GetDefaultFontStyle("Regular", 15);
	const FSlateFontInfo TypeFont = FCoreStyle::GetDefaultFontStyle("Regular", 14);
	ChildSlot
	[
		SNew(SBox)
		.WidthOverride(500.f)
		.HeightOverride(340.f)
		[
			SNew(SBorder)
			.BorderImage(FCoreStyle::Get().GetBrush("WhiteBrush"))
			.BorderBackgroundColor(FLinearColor(0.06f, 0.07f, 0.09f, 0.96f))
			.Padding(18.f)
			.Cursor(EMouseCursor::CardinalCross)
			[
				SNew(SVerticalBox)
				+ SVerticalBox::Slot().AutoHeight()
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot().AutoWidth()
					[
						SNew(SButton)
						.IsFocusable(false)
						.ContentPadding(FMargin(18.f, 6.f))
						.OnClicked(this, &SBbSavePlayDialog::Cancel)
						[
							SNew(STextBlock).Text(FText::FromString(TEXT("Cancel"))).Font(ButtonFont)
						]
					]
					+ SHorizontalBox::Slot().FillWidth(1.f)
					[
						SNew(SSpacer)
					]
					+ SHorizontalBox::Slot().AutoWidth()
					[
						SNew(SButton)
						.IsFocusable(false)
						.ContentPadding(FMargin(18.f, 6.f))
						.OnClicked(this, &SBbSavePlayDialog::Save)
						[
							SNew(STextBlock).Text(FText::FromString(TEXT("Save"))).Font(ButtonFont)
						]
					]
				]
				+ SVerticalBox::Slot().AutoHeight().Padding(4.f, 14.f, 4.f, 0.f)
				[
					SNew(STextBlock).Text(InArgs._Title).Font(TitleFont).ColorAndOpacity(FLinearColor(1.f, 0.85f, 0.2f))
				]
				+ SVerticalBox::Slot().AutoHeight().Padding(4.f, 12.f, 4.f, 6.f)
				[
					SNew(STextBlock).Text(FText::FromString(TEXT("Comment:"))).Font(LabelFont).ColorAndOpacity(FLinearColor::White)
				]
				+ SVerticalBox::Slot().FillHeight(1.f)
				[
					SAssignNew(Box, SMultiLineEditableTextBox)
					.Cursor(EMouseCursor::TextEditBeam)
					.Font(TypeFont)
					.AutoWrapText(true)
					.HintText(FText::FromString(TEXT("What happened on this play?   (Ctrl+Enter saves, Esc cancels)")))
					.OnKeyDownHandler_Lambda([this](const FGeometry& G, const FKeyEvent& K) { return OnKeyDown(G, K); })
				]
			]
		]
	];
}

FReply SBbSavePlayDialog::OnKeyDown(const FGeometry& Geometry, const FKeyEvent& Key)
{
	if (Key.GetKey() == EKeys::Escape)
	{
		return Cancel();
	}
	if (Key.GetKey() == EKeys::Enter && Key.IsControlDown())
	{
		return Save();
	}
	return FReply::Unhandled();
}

FReply SBbSavePlayDialog::OnMouseButtonDown(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	// Reached only when no button or the text box took the click: the window's own surface.
	if (Mouse.GetEffectingButton() != EKeys::LeftMouseButton)
	{
		return FReply::Unhandled();
	}
	bDragging = true;
	return FReply::Handled().CaptureMouse(SharedThis(this));
}

FReply SBbSavePlayDialog::OnMouseMove(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	if (!bDragging || !HasMouseCapture())
	{
		return FReply::Unhandled();
	}
	// Screen pixels to Slate units; the offset is from the right edge, so X runs backwards.
	const FVector2D D = Mouse.GetCursorDelta() / FMath::Max(Geometry.Scale, 0.01f);
	Offset.X = FMath::Max(0.f, Offset.X - D.X);
	Offset.Y = FMath::Max(0.f, Offset.Y + D.Y);
	return FReply::Handled();
}

FReply SBbSavePlayDialog::OnMouseButtonUp(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	if (!bDragging)
	{
		return FReply::Unhandled();
	}
	bDragging = false;
	return FReply::Handled().ReleaseMouseCapture();
}

FReply SBbSavePlayDialog::Cancel()
{
	OnCancel.ExecuteIfBound();
	return FReply::Handled();
}

FReply SBbSavePlayDialog::Save()
{
	OnSave.ExecuteIfBound(Box.IsValid() ? Box->GetText().ToString() : FString());
	return FReply::Handled();
}
