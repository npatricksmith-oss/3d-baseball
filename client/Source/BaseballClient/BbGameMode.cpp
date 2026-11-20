#include "BbGameMode.h"
#include "BbDirector.h"
#include "BbLook.h"
#include "Engine/Canvas.h"
#include "Engine/Engine.h"
#include "Engine/World.h"
#include "Misc/CommandLine.h"

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
	static const bool bHoldFlag = FParse::Param(FCommandLine::Get(), TEXT("BbHudHold"));
	bHold = bHold || bHoldFlag;
	// The bug, and under it the pitch, the replay tag and any note.
	float Y = BbHud::DrawBug(*this, S, Bug);
	if (!PitchLine.IsEmpty())
	{
		Y = BbHud::DrawTag(*this, S, Y, PitchLine, BbLook::HudDim);
	}
	if (!Replay.IsEmpty())
	{
		Y = BbHud::DrawTag(*this, S, Y, Replay, BbLook::HudGold);
	}
	if (!Notice.IsEmpty())
	{
		Y = BbHud::DrawTag(*this, S, Y, Notice, FLinearColor::FromSRGBColor(FColor(140, 230, 150)));
	}
	if (!SavedComment.IsEmpty() || !SavedCommentTitle.IsEmpty())
	{
		Y = BbHud::DrawComment(*this, S, Y + 8.f, SavedCommentTitle, SavedComment, SavedErrorTitle.IsEmpty() ? BbLook::HudGold : BbLook::HudClose);
	}
	if (!SavedWarning.IsEmpty())
	{
		Y = BbHud::DrawComment(*this, S, Y + 8.f, TEXT("NOT EXACTLY AS SAVED"), SavedWarning, BbLook::HudClose);
	}
	if (bZoneShown && CardAlpha <= 0.f)
	{
		BbHud::DrawZone(*this, S, Zone);
	}
	if (!Footer.IsEmpty())
	{
		DrawText(Footer, FLinearColor(1.f, 1.f, 1.f, 0.6f), 24.f * S, Canvas->ClipY - 30.f * S, Font, 0.9f * S);
	}
	const int32 Batting = Bug.bTop ? 0 : 1;
	const FString CallShown = bHold && Call.IsEmpty() ? FString(TEXT("Hayes: Ground out, shortstop to first")) : Call;
	if (!CallShown.IsEmpty())
	{
		BbHud::DrawCaption(*this, S, CallShown, Batting);
	}
	// Above the caption, one at a time: a home run's card, else the umpire's out signal.
	const bool bHomeRun = HomeRunAlpha > 0.f && !HomeRunName.IsEmpty();
	if (bHomeRun)
	{
		BbHud::DrawHomeRun(*this, S, HomeRunName, HomeRunLine, HomeRunKind, HomeRunSide, HomeRunAlpha);
	}
	else if (!Umpire.IsEmpty() || bHold)
	{
		BbHud::DrawUmpire(*this, S, Umpire.IsEmpty() ? FString(TEXT("OUT! at first")) : Umpire, Umpire.IsEmpty() || bUmpireClose);
	}
	if (bHold && !bHomeRun)
	{
		// The home-run card where it would be, moved up out of the umpire's way.
		BbHud::DrawHomeRun(*this, S, TEXT("HAYES"), TEXT("412 ft     108 mph"), TEXT("Two-run home run"), Batting, 1.f, 70.f);
	}
	if (CardAlpha > 0.f && !Card.IsEmpty())
	{
		// Centred on the black between innings; it comes and goes with the fade.
		BbHud::DrawInningCard(*this, S, Card, CardRuns, bLineBox ? &LineBox : nullptr, CardAlpha);
		if (bCardHint)
		{
			const FString Hint(TEXT("Enter: continue"));
			float W = 0.f, Hh = 0.f;
			GetTextSize(Hint, W, Hh, Font, 1.0f * S);
			DrawText(Hint, FLinearColor(1.f, 1.f, 1.f, 0.55f * CardAlpha), (Canvas->ClipX - W) * 0.5f, Canvas->ClipY * 0.5f + 130.f * S, Font, 1.0f * S);
		}
	}
	else if (bHold)
	{
		const int32 Runs[2] = { Bug.Runs[0], Bug.Runs[1] };
		BbHud::DrawInningCard(*this, S, TEXT("Middle of the 3rd"), Runs, bLineBox ? &LineBox : nullptr, 0.85f);
	}
	if (bBoxData && BoxAlpha > 0.f)
	{
		BbHud::DrawBoxScore(*this, S, Box, BoxTitle, BoxSubtitle, BoxAlpha, BoxDim);
	}
	if (!SavedErrorTitle.IsEmpty())
	{
		BbHud::DrawBanner(*this, S, SavedErrorTitle, SavedErrorBody, BbLook::HudClose);
	}
}
