#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "GameFramework/HUD.h"
#include "bb_ffi.h"
#include "BbHud.h"
#include "BbGameMode.generated.h"

/** Spawns the director, which builds the field and plays the game. No pawn. */
UCLASS()
class ABbGameMode : public AGameModeBase
{
	GENERATED_BODY()

public:
	ABbGameMode();
	virtual void StartPlay() override;
};

/** The game graphics (docs/hud.md): the score bug, the caption, the cards and the box
 *  score, drawn by BbHud from what the director puts here. */
UCLASS()
class ABbHUD : public AHUD
{
	GENERATED_BODY()

public:
	virtual void DrawHUD() override;
	/** For the panels in BbHud, which draw on it. */
	UCanvas* GetCanvas() const { return Canvas; }
	/** The score bug: clubs, runs, inning, bases, outs, count. */
	FBbBugInfo Bug;
	/** The strike-zone box, top right; Z hides and shows it. */
	FBbZoneInfo Zone;
	bool bZoneShown = true;
	/** Under the bug: the last pitch's speed, and the ball's off the bat once it is hit. */
	FString PitchLine;
	/** While an earlier pitch is being replayed: "REPLAY   seed 7   play 123". */
	FString Replay;
	/** A short note under it, such as where a saved play went. */
	FString Notice;
	/** Saved play (-BbSavedPlay): comment under the bug/tags, and an error banner when the
	 *  file has no recorded timeline. */
	FString SavedCommentTitle;
	FString SavedComment;
	FString SavedErrorTitle;
	FString SavedErrorBody;
	/** A saved play that does not record its names or park: what is shown is not as it was seen. */
	FString SavedWarning;
	/** The call on the current pitch, shown large once it is known. */
	FString Call;
	/** The umpire's signal of the latest out ("OUT at first"), shown as he makes it. */
	FString Umpire;
	/** That out was a bang-bang play: the umpire sells it. */
	bool bUmpireClose = false;
	/** Key help and playback state, bottom left. */
	FString Footer;
	/** Between innings, while the picture is cut away: "Middle of the 3rd" and the score. */
	FString Card;
	/** The runs on the card: [0] the visitors, [1] the home side. */
	int32 CardRuns[2] = { 0, 0 };
	/** 0 hidden, 1 fully shown. */
	float CardAlpha = 0.f;
	/** The card is held until Enter or its time is up: say so under it. */
	bool bCardHint = false;
	/** The line score on that card: the sim's box score through the pitches shown (the same
	 *  one the scoreboard in the park reads). Without it the card shows the runs alone. */
	BbBoxScore LineBox{};
	bool bLineBox = false;
	/** A home run (A16): the batter's family name, "412 ft   108 mph", and what it scored
	 *  ("Two-run home run"), as a card low on the screen while he circles the bases. */
	FString HomeRunName;
	FString HomeRunLine;
	FString HomeRunKind;
	/** The hitter's side, for his club's colour on the card. */
	int32 HomeRunSide = 0;
	/** 0 hidden, 1 fully shown. */
	float HomeRunAlpha = 0.f;
	/** The box score (A12; hud.md step 4): a full-screen panel in the HUD's look. Held with Tab
	 *  during a game, over a dimmed field with what has happened so far; at the end of the game
	 *  it fades in over black and stays until the player quits. */
	BbBoxScore Box{};
	/** Box holds a score read from the sim. */
	bool bBoxData = false;
	/** The linescore's second line and the title: "FINAL", or "GAME IN PROGRESS". */
	FString BoxTitle;
	FString BoxSubtitle;
	/** 0 hidden, 1 fully shown. */
	float BoxAlpha = 0.f;
	/** How dark the field is behind the panel (Tab dims it; the final box sits on black). */
	float BoxDim = 0.f;
	/** -BbBodySwing (player-visuals §14): lines of plain text, top left, in place of every game
	 *  panel; the key help is the Footer. */
	TArray<FString> BodyLines;
	/** -BbHudHold: every panel stays up (made-up words where the game has none yet) so a still
	 *  catches them all (hud.md §4). */
	bool bHold = false;
};
