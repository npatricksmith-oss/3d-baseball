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
	if (CardAlpha > 0.f && !Card.IsEmpty())
	{
		// Centered on the black between innings; it comes and goes with the fade.
		float W = 0.f, H = 0.f, W2 = 0.f, H2 = 0.f;
		GetTextSize(Card, W, H, Font, 3.0f * S);
		GetTextSize(CardScore, W2, H2, Font, 2.2f * S);
		const float Y = Canvas->ClipY * 0.5f - H;
		DrawText(Card, FLinearColor(1.f, 1.f, 1.f, CardAlpha), (Canvas->ClipX - W) * 0.5f, Y, Font, 3.0f * S);
		DrawText(CardScore, FLinearColor(1.f, 0.95f, 0.6f, CardAlpha), (Canvas->ClipX - W2) * 0.5f, Y + H + 14.f * S, Font, 2.2f * S);
	}
	if (HomeRunAlpha > 0.f && !HomeRunName.IsEmpty())
	{
		// The home-run card: low and centered, over the view of the trot.
		float W1 = 0.f, H1 = 0.f, W2 = 0.f, H2 = 0.f, W3 = 0.f, H3 = 0.f;
		GetTextSize(HomeRunName, W1, H1, Font, 2.6f * S);
		GetTextSize(HomeRunLine, W2, H2, Font, 1.8f * S);
		GetTextSize(HomeRunKind, W3, H3, Font, 1.5f * S);
		const float W = FMath::Max3(W1, W2, W3) + 48.f * S;
		const float H = H1 + H2 + H3 + 40.f * S;
		const float X = (Canvas->ClipX - W) * 0.5f;
		const float Y = Canvas->ClipY - H - 70.f * S;
		const float A = HomeRunAlpha;
		DrawRect(FLinearColor(0.02f, 0.05f, 0.15f, 0.8f * A), X, Y, W, H);
		DrawRect(FLinearColor(1.f, 0.85f, 0.1f, A), X, Y, W, 4.f * S);
		DrawText(HomeRunName, FLinearColor(1.f, 1.f, 1.f, A), (Canvas->ClipX - W1) * 0.5f, Y + 12.f * S, Font, 2.6f * S);
		DrawText(HomeRunLine, FLinearColor(1.f, 0.95f, 0.6f, A), (Canvas->ClipX - W2) * 0.5f, Y + 16.f * S + H1, Font, 1.8f * S);
		DrawText(HomeRunKind, FLinearColor(0.85f, 0.9f, 1.f, A), (Canvas->ClipX - W3) * 0.5f, Y + 20.f * S + H1 + H2, Font, 1.5f * S);
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
	if (!Umpire.IsEmpty())
	{
		// The umpire's out signal, below the call line: larger and red on a close play.
		const float Scale = (bUmpireClose ? 2.8f : 2.4f) * S;
		float W = 0.f, H = 0.f;
		GetTextSize(Umpire, W, H, Font, Scale);
		const float X = (Canvas->ClipX - W) * 0.5f;
		const float Y = 172.f * S;
		DrawRect(FLinearColor(0.f, 0.f, 0.f, 0.6f), X - 18.f * S, Y - 8.f * S, W + 36.f * S, H + 16.f * S);
		DrawText(Umpire, bUmpireClose ? FLinearColor(1.f, 0.35f, 0.3f) : FLinearColor::White, X, Y, Font, Scale);
	}
}
