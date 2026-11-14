#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "GameFramework/HUD.h"
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

/** Scoreboard and last-pitch line, drawn as text. */
UCLASS()
class ABbHUD : public AHUD
{
	GENERATED_BODY()

public:
	virtual void DrawHUD() override;
	FString Scoreboard;
	FString PitchLine;
	/** The call on the current pitch, shown large once it is known. */
	FString Call;
	/** The umpire's signal of the latest out ("OUT at first"), shown as he makes it. */
	FString Umpire;
	/** That out was a bang-bang play: the umpire sells it. */
	bool bUmpireClose = false;
	/** Runners on base: bit 0 first, bit 1 second, bit 2 third. */
	uint8 Bases = 0;
	/** Key help and playback state, bottom left. */
	FString Footer;
	/** Between innings, while the picture is cut away: "Middle of the 3rd" and the score. */
	FString Card;
	FString CardScore;
	/** 0 hidden, 1 fully shown. */
	float CardAlpha = 0.f;
	/** A home run (A16): the batter's family name, "412 ft   108 mph", and what it scored
	 *  ("Two-run home run"), as a card low on the screen while he circles the bases. */
	FString HomeRunName;
	FString HomeRunLine;
	FString HomeRunKind;
	/** 0 hidden, 1 fully shown. */
	float HomeRunAlpha = 0.f;
};
