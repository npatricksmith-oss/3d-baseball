// The game graphics (docs/hud.md): each panel is a function over a plain struct of what it
// shows. The director fills the structs from the sim; nothing here counts or decides anything.
// Drawn on the canvas from a layout made for 720 lines: one reference unit is S pixels.
#pragma once

#include "CoreMinimal.h"

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

namespace BbHud
{
	/** The bug at the top left; returns its bottom edge in reference units. */
	float DrawBug(ABbHUD& Hud, float S, const FBbBugInfo& Bug);
	/** A slim line under the bug (the pitch, the replay, a note); returns its bottom edge. */
	float DrawTag(ABbHUD& Hud, float S, float Y, const FString& Text, const FLinearColor& TextColor);
	/** The play caption: low and centred, edged in the batting side's colour. */
	void DrawCaption(ABbHUD& Hud, float S, const FString& Text, int32 BattingSide);
	/** The umpire's out signal, above the caption; a bang-bang play is edged in red. */
	void DrawUmpire(ABbHUD& Hud, float S, const FString& Text, bool bClose);
	/** A home run: name, distance and speed, what it scored; above the caption, or Lift
	 *  reference units higher still. */
	void DrawHomeRun(ABbHUD& Hud, float S, const FString& Name, const FString& Line, const FString& Kind, int32 Side, float Alpha, float Lift = 0.f);
	/** Between innings, on the black: "Middle of the 3rd" and the two clubs' runs. */
	void DrawInningCard(ABbHUD& Hud, float S, const FString& Title, const int32 Runs[2], float Alpha);
}
