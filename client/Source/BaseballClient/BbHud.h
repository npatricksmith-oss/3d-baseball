// The game graphics (docs/hud.md): each panel is a function over a plain struct of what it
// shows. The director fills the structs from the sim; nothing here counts or decides anything.
// Drawn on the canvas from a layout made for 720 lines: one reference unit is S pixels.
#pragma once

#include "CoreMinimal.h"
#include "bb_ffi.h"

class ABbHUD;

/** The score bug, top left (step 1). All of it read from the sim's record of the play. */
struct FBbBugInfo
{
	/** False before the first pitch: the bug shows the clubs at nil. */
	bool bValid = false;
	/** [0] the visitors, [1] the home side. */
	int32 Runs[2] = { 0, 0 };
	int32 Inning = 1;
	bool bTop = true;
	int32 Outs = 0;
	int32 Balls = 0;
	int32 Strikes = 0;
	/** Runners on base: bit 0 first, bit 1 second, bit 2 third. */
	uint8 Bases = 0;
	/** The two rows under the score (owner, 2026-11-16): the pitcher with the pitches he has
	 *  thrown, and the batter with his game so far ("1-2, 2B, RBI"). Empty names: no rows. */
	FString Pitcher;
	int32 PitcherNumber = 0;
	int32 Pitches = 0;
	FString Batter;
	int32 BatterNumber = 0;
	/** "CF". */
	FString BatterPosition;
	FString BatterToday;
};

/** One pitch in the strike-zone box: where it crossed (m: x across the plate in the sim's
 *  frame, z up from the ground) and its call (BbPlayInfo.pitch_call). */
struct FBbZoneMark
{
	float X = 0.f;
	float Z = 0.f;
	uint8 Call = 0;
};

/** The strike-zone box (step 3): the batter's zone from the sim and the pitches of this
 *  at-bat, oldest first, numbered in that order. Seen from the pitcher's side, as the
 *  centre-field camera sees it. */
struct FBbZoneInfo
{
	bool bValid = false;
	float Bottom = 0.f;
	float Top = 0.f;
	float HalfWidth = 0.f;
	TArray<FBbZoneMark> Marks;
};

namespace BbHud
{
	/** The bug at the top left; returns its bottom edge in reference units. */
	float DrawBug(ABbHUD& Hud, float S, const FBbBugInfo& Bug);
	/** A slim line under the bug (the pitch, the replay, a note); returns its bottom edge. */
	float DrawTag(ABbHUD& Hud, float S, float Y, const FString& Text, const FLinearColor& TextColor);
	/** A word-wrapped note box under the bug and tags (saved-play comment); returns its bottom edge. */
	float DrawComment(ABbHUD& Hud, float S, float Y, const FString& Title, const FString& Body, const FLinearColor& Edge);
	/** A centred notice card for a saved play that cannot be played back (missing timeline). */
	void DrawBanner(ABbHUD& Hud, float S, const FString& Title, const FString& Body, const FLinearColor& Edge);
	/** The play caption: low and centred, edged in the batting side's colour. */
	void DrawCaption(ABbHUD& Hud, float S, const FString& Text, int32 BattingSide);
	/** The umpire's out signal, above the caption; a bang-bang play is edged in red. */
	void DrawUmpire(ABbHUD& Hud, float S, const FString& Text, bool bClose);
	/** A home run: name, distance and speed, what it scored; above the caption, or Lift
	 *  reference units higher still. */
	void DrawHomeRun(ABbHUD& Hud, float S, const FString& Name, const FString& Line, const FString& Kind, int32 Side, float Alpha, float Lift = 0.f);
	/** The strike-zone box at the top right. */
	void DrawZone(ABbHUD& Hud, float S, const FBbZoneInfo& Zone);
	/** Between innings, on the black: "Middle of the 3rd" over the line score (an inning to a
	 *  column, then runs and hits), read from the sim's box score. Without a box score (Box
	 *  null) it shows the two clubs' runs alone. */
	/** The sim opens a line-score slot for the next half inning in the play that makes the third
	 *  out, so a box read through that play holds an inning that has not been played. Drop what
	 *  the half shown (Inning, bTop) has not reached: the visitors have batted through Inning,
	 *  the home side through Inning - 1 in the top half and Inning in the bottom. */
	void TrimUnplayed(BbBoxScore& Box, int32 Inning, bool bTop);
	void DrawInningCard(ABbHUD& Hud, float S, const FString& Title, const int32 Runs[2], const BbBoxScore* Box, float Alpha);
	/** The box score, full screen (step 4): the score in words, the line score, each club's
	 *  batting lines and the notes under them. Dim is how dark the field behind it goes. */
	void DrawBoxScore(ABbHUD& Hud, float S, const BbBoxScore& Box, const FString& Title, const FString& Subtitle, float Alpha, float Dim);
}
