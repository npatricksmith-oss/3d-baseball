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
	// Runners on: a small diamond, lit bases in yellow.
	const FVector2D Diamond(470.f * S, 60.f * S);
	const FVector2D Offsets[3] = { FVector2D(18.f, 0.f), FVector2D(0.f, -18.f), FVector2D(-18.f, 0.f) };
	for (int32 I = 0; I < 3; ++I)
	{
		const FLinearColor Col = (Bases >> I) & 1 ? FLinearColor(1.f, 0.85f, 0.1f) : FLinearColor(0.35f, 0.35f, 0.35f);
		DrawRect(Col, Diamond.X + Offsets[I].X * S - 7.f * S, Diamond.Y + Offsets[I].Y * S - 7.f * S, 14.f * S, 14.f * S);
	}
	if (!Footer.IsEmpty())
	{
		DrawText(Footer, FLinearColor(1.f, 1.f, 1.f, 0.7f), 24.f * S, Canvas->ClipY - 30.f * S, Font, 1.0f * S);
	}
	if (!Call.IsEmpty())
	{
		float W = 0.f, H = 0.f;
		GetTextSize(Call, W, H, Font, 2.2f * S);
		const float X = (Canvas->ClipX - W) * 0.5f;
		const float Y = 118.f * S;
		DrawRect(FLinearColor(0.f, 0.f, 0.f, 0.55f), X - 18.f * S, Y - 8.f * S, W + 36.f * S, H + 16.f * S);
		DrawText(Call, FLinearColor(1.f, 0.95f, 0.6f), X, Y, Font, 2.2f * S);
	}
}
