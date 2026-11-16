#include "BbHud.h"
#include "BbLook.h"
#include "Engine/Canvas.h"
#include "Engine/Engine.h"
#include "BbGameMode.h"

namespace
{
	// The bug's layout, in reference units (720 lines).
	constexpr float BugX = 20.f, BugY = 18.f;
	constexpr float RowH = 32.f;
	constexpr float ChipW = 8.f, NameW = 118.f, ScoreW = 44.f;
	constexpr float DiamondW = 70.f, StateW = 74.f;
	constexpr float BugW = ChipW + NameW + ScoreW + DiamondW + StateW;
	// The caption sits above the footer of keys; the panels above it stack upwards.
	constexpr float CaptionBottom = 682.f, CaptionH = 40.f, StackGap = 10.f;

	UFont* Font() { return GEngine->GetLargeFont(); }

	FLinearColor Faded(FLinearColor C, float A)
	{
		C.A *= A;
		return C;
	}

	/** Text with its left edge at X and its middle at Y (pixels). */
	void TextMid(ABbHUD& Hud, const FString& Str, float X, float Y, const FLinearColor& Col, float Scale)
	{
		float W = 0.f, H = 0.f;
		Hud.GetTextSize(Str, W, H, Font(), Scale);
		Hud.DrawText(Str, Col, X, Y - H * 0.5f, Font(), Scale);
	}

	/** Text centred on (X, Y) (pixels). */
	void TextCentre(ABbHUD& Hud, const FString& Str, float X, float Y, const FLinearColor& Col, float Scale)
	{
		float W = 0.f, H = 0.f;
		Hud.GetTextSize(Str, W, H, Font(), Scale);
		Hud.DrawText(Str, Col, X - W * 0.5f, Y - H * 0.5f, Font(), Scale);
	}

	/** Text with its right edge at X and its middle at Y (pixels). */
	void TextRight(ABbHUD& Hud, const FString& Str, float X, float Y, const FLinearColor& Col, float Scale)
	{
		float W = 0.f, H = 0.f;
		Hud.GetTextSize(Str, W, H, Font(), Scale);
		Hud.DrawText(Str, Col, X - W, Y - H * 0.5f, Font(), Scale);
	}

	void Triangle(ABbHUD& Hud, FVector2D A, FVector2D B, FVector2D C, const FLinearColor& Col)
	{
		FCanvasUVTri T;
		T.V0_Pos = A;
		T.V1_Pos = B;
		T.V2_Pos = C;
		T.V0_Color = T.V1_Color = T.V2_Color = Col;
		Hud.GetCanvas()->K2_DrawTriangle(nullptr, { T });
	}

	/** A square standing on its corner, centred at C with half-diagonal R (pixels). */
	void Diamond(ABbHUD& Hud, FVector2D C, float R, const FLinearColor& Col)
	{
		Triangle(Hud, C + FVector2D(0.f, -R), C + FVector2D(R, 0.f), C + FVector2D(0.f, R), Col);
		Triangle(Hud, C + FVector2D(0.f, -R), C + FVector2D(0.f, R), C + FVector2D(-R, 0.f), Col);
	}

	/** A panel centred across the screen, its bottom at Bottom (reference units), with a
	 *  coloured edge along its top. Returns its top in reference units. */
	float CentredPanel(ABbHUD& Hud, float S, float W, float H, float Bottom, const FLinearColor& Edge, float Alpha)
	{
		const float X = Hud.GetCanvas()->ClipX * 0.5f - W * 0.5f * S;
		const float Top = Bottom - H;
		Hud.DrawRect(Faded(BbLook::HudPanel, Alpha), X, Top * S, W * S, H * S);
		Hud.DrawRect(Faded(Edge, Alpha), X, Top * S, W * S, 4.f * S);
		return Top;
	}

	/** One club's row: its colour, its name, its runs (all in pixels from the row's top left). */
	void ClubRow(ABbHUD& Hud, float S, float X, float Y, float NameWide, float ScoreWide, float H, int32 Side, const TCHAR* Name, int32 Runs, float NameScale, float RunScale, float Alpha)
	{
		Hud.DrawRect(Faded(BbLook::HudClub[Side & 1], Alpha), X, Y, ChipW * S, H * S);
		Hud.DrawRect(Faded(BbLook::HudPanelLight, Alpha), X + (ChipW + NameWide) * S, Y, ScoreWide * S, H * S);
		TextMid(Hud, Name, X + (ChipW + 10.f) * S, Y + H * 0.5f * S, Faded(BbLook::HudText, Alpha), NameScale * S);
		TextCentre(Hud, FString::FromInt(Runs), X + (ChipW + NameWide + ScoreWide * 0.5f) * S, Y + H * 0.5f * S, Faded(BbLook::HudText, Alpha), RunScale * S);
	}
}

float BbHud::DrawBug(ABbHUD& Hud, float S, const FBbBugInfo& Bug)
{
	const float X = BugX * S, Y = BugY * S;
	Hud.DrawRect(BbLook::HudPanel, X, Y, BugW * S, 2.f * RowH * S);
	// The clubs, visitors above, each in its colour; a hair between the rows.
	for (int32 Side = 0; Side < 2; ++Side)
	{
		ClubRow(Hud, S, X, Y + Side * RowH * S, NameW, ScoreW, RowH, Side, BbLook::Clubs[Side].Name, Bug.Runs[Side], 1.05f, 1.35f, 1.f);
	}
	Hud.DrawRect(BbLook::HudPanel, X, Y + (RowH - 1.f) * S, (ChipW + NameW + ScoreW) * S, 2.f * S);

	// The bases: second at the top, first to the right, as the field is seen from home.
	const float DX = X + (ChipW + NameW + ScoreW + DiamondW * 0.5f) * S;
	const float DY = Y + 15.f * S;
	const float R = 8.f * S, Step = 15.f * S;
	const FVector2D Bases[3] = { FVector2D(DX + Step, DY + Step), FVector2D(DX, DY), FVector2D(DX - Step, DY + Step) };
	for (int32 I = 0; I < 3; ++I)
	{
		const bool bOn = Bug.bValid && ((Bug.Bases >> I) & 1);
		Diamond(Hud, Bases[I], R, bOn ? BbLook::HudGold : BbLook::HudOff);
	}
	// The outs, as three dots under the bases.
	for (int32 I = 0; I < 3; ++I)
	{
		const float OX = DX + (I - 1) * 13.f * S, OY = Y + 52.f * S;
		const bool bMade = Bug.bValid && I < Bug.Outs;
		Hud.DrawRect(bMade ? BbLook::HudGold : BbLook::HudOff, OX - 4.f * S, OY - 4.f * S, 8.f * S, 8.f * S);
	}

	// The inning above, the count below.
	const float SX = X + (BugW - StateW) * S;
	Hud.DrawRect(BbLook::HudPanelLight, SX, Y, StateW * S, 2.f * RowH * S);
	const float MidX = SX + StateW * 0.5f * S;
	const float RowMid = Y + RowH * 0.5f * S;
	const FString Inning = FString::FromInt(Bug.Inning);
	float IW = 0.f, IH = 0.f;
	Hud.GetTextSize(Inning, IW, IH, Font(), 1.3f * S);
	const float AX = MidX - (IW + 14.f * S) * 0.5f;
	const float A = 6.f * S;
	if (Bug.bTop)
	{
		Triangle(Hud, FVector2D(AX, RowMid + A * 0.6f), FVector2D(AX + 2.f * A, RowMid + A * 0.6f), FVector2D(AX + A, RowMid - A * 0.9f), BbLook::HudText);
	}
	else
	{
		Triangle(Hud, FVector2D(AX, RowMid - A * 0.6f), FVector2D(AX + 2.f * A, RowMid - A * 0.6f), FVector2D(AX + A, RowMid + A * 0.9f), BbLook::HudText);
	}
	TextMid(Hud, Inning, AX + 2.f * A + 4.f * S, RowMid, BbLook::HudText, 1.3f * S);
	const FString Count = Bug.bValid ? FString::Printf(TEXT("%d-%d"), Bug.Balls, Bug.Strikes) : TEXT("0-0");
	TextCentre(Hud, Count, MidX, Y + RowH * 1.5f * S, BbLook::HudText, 1.3f * S);
	return BugY + 2.f * RowH;
}

float BbHud::DrawTag(ABbHUD& Hud, float S, float Y, const FString& Text, const FLinearColor& TextColor)
{
	const float H = 24.f;
	Hud.DrawRect(BbLook::HudPanel, BugX * S, Y * S, BugW * S, H * S);
	TextMid(Hud, Text, (BugX + 10.f) * S, (Y + H * 0.5f) * S, TextColor, 0.95f * S);
	return Y + H;
}

void BbHud::DrawCaption(ABbHUD& Hud, float S, const FString& Text, int32 BattingSide)
{
	float W = 0.f, H = 0.f;
	Hud.GetTextSize(Text, W, H, Font(), 1.45f * S);
	const float PanelW = FMath::Max(W / S + 56.f, 280.f);
	const float Top = CentredPanel(Hud, S, PanelW, CaptionH, CaptionBottom, BbLook::HudClub[BattingSide & 1], 1.f);
	TextCentre(Hud, Text, Hud.GetCanvas()->ClipX * 0.5f, (Top + 2.f + CaptionH * 0.5f) * S, BbLook::HudText, 1.45f * S);
}

void BbHud::DrawUmpire(ABbHUD& Hud, float S, const FString& Text, bool bClose)
{
	const float Scale = (bClose ? 2.2f : 1.8f) * S;
	float W = 0.f, H = 0.f;
	Hud.GetTextSize(Text, W, H, Font(), Scale);
	const float PanelH = H / S + 18.f;
	const float Top = CentredPanel(Hud, S, W / S + 48.f, PanelH, CaptionBottom - CaptionH - StackGap, bClose ? BbLook::HudClose : BbLook::HudText, 1.f);
	TextCentre(Hud, Text, Hud.GetCanvas()->ClipX * 0.5f, (Top + 2.f + PanelH * 0.5f) * S, bClose ? BbLook::HudClose : BbLook::HudText, Scale);
}

void BbHud::DrawHomeRun(ABbHUD& Hud, float S, const FString& Name, const FString& Line, const FString& Kind, int32 Side, float Alpha, float Lift)
{
	float W1 = 0.f, H1 = 0.f, W2 = 0.f, H2 = 0.f, W3 = 0.f, H3 = 0.f;
	Hud.GetTextSize(Name, W1, H1, Font(), 2.3f * S);
	Hud.GetTextSize(Line, W2, H2, Font(), 1.5f * S);
	Hud.GetTextSize(Kind, W3, H3, Font(), 1.2f * S);
	const float W = FMath::Max3(W1, W2, W3) / S + 64.f;
	const float H = (H1 + H2 + H3) / S + 34.f;
	const float Top = CentredPanel(Hud, S, W, H, CaptionBottom - CaptionH - StackGap - Lift, BbLook::HudGold, Alpha);
	// The hitter's club down the left edge.
	const float X = Hud.GetCanvas()->ClipX * 0.5f - W * 0.5f * S;
	Hud.DrawRect(Faded(BbLook::HudClub[Side & 1], Alpha), X, (Top + 4.f) * S, ChipW * S, (H - 4.f) * S);
	const float CX = Hud.GetCanvas()->ClipX * 0.5f;
	float Y = (Top + 12.f) * S;
	Hud.DrawText(Kind, Faded(BbLook::HudGold, Alpha), CX - W3 * 0.5f, Y, Font(), 1.2f * S);
	Y += H3 + 2.f * S;
	Hud.DrawText(Name, Faded(BbLook::HudText, Alpha), CX - W1 * 0.5f, Y, Font(), 2.3f * S);
	Y += H1 + 2.f * S;
	Hud.DrawText(Line, Faded(BbLook::HudDim, Alpha), CX - W2 * 0.5f, Y, Font(), 1.5f * S);
}

void BbHud::DrawInningCard(ABbHUD& Hud, float S, const FString& Title, const int32 Runs[2], float Alpha)
{
	// Two club rows under a title, centred on the black.
	const float NameWide = 330.f, ScoreWide = 64.f, Row = 48.f, TitleH = 50.f;
	const float W = ChipW + NameWide + ScoreWide;
	const float H = TitleH + 2.f * Row;
	const float Bottom = 360.f + H * 0.5f;
	const float Top = CentredPanel(Hud, S, W, H, Bottom, BbLook::HudGold, Alpha);
	const float X = Hud.GetCanvas()->ClipX * 0.5f - W * 0.5f * S;
	TextCentre(Hud, Title.ToUpper(), Hud.GetCanvas()->ClipX * 0.5f, (Top + 4.f + (TitleH - 4.f) * 0.5f) * S, Faded(BbLook::HudText, Alpha), 1.5f * S);
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const FString Name = FString::Printf(TEXT("%s %s"), BbLook::Clubs[Side].Place, BbLook::Clubs[Side].Name);
		ClubRow(Hud, S, X, (Top + TitleH + Side * Row) * S, NameWide, ScoreWide, Row, Side, *Name, Runs[Side], 1.35f, 1.9f, Alpha);
	}
	Hud.DrawRect(Faded(BbLook::HudPanel, Alpha), X, (Top + TitleH + Row - 1.f) * S, W * S, 2.f * S);
}
