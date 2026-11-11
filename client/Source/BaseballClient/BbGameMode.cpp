#include "BbGameMode.h"
#include "BbDirector.h"
#include "Engine/Canvas.h"
#include "Engine/Engine.h"
#include "Engine/World.h"

ABbGameMode::ABbGameMode()
{
	DefaultPawnClass = nullptr;
	HUDClass = ABbHUD::StaticClass();
}

void ABbGameMode::StartPlay()
{
	Super::StartPlay();
	GetWorld()->SpawnActor<ABbDirector>();
}

void ABbHUD::DrawHUD()
{
	Super::DrawHUD();
	if (!Canvas || !GEngine)
	{
		return;
	}
	const float S = Canvas->ClipY / 720.f;
	UFont* Font = GEngine->GetLargeFont();
	// A dark strip behind the text so it reads against grass and sky.
	DrawRect(FLinearColor(0.f, 0.f, 0.f, 0.55f), 20.f * S, 20.f * S, 520.f * S, 78.f * S);
	DrawText(Scoreboard, FLinearColor::White, 32.f * S, 28.f * S, Font, 1.6f * S);
	DrawText(PitchLine, FLinearColor(0.85f, 0.9f, 1.f), 32.f * S, 62.f * S, Font, 1.2f * S);
}
