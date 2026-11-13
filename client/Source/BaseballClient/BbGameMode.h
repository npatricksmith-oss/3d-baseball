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
};
