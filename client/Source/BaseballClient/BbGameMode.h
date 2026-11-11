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
};
