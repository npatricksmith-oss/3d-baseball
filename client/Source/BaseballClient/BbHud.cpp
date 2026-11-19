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

namespace
{
	/** A row under the bug: club colour, position, number and name, and a figure at the
	 *  right ("42 P", "1-2, 2B"). Returns its bottom edge (reference units). */
	float PersonRow(ABbHUD& Hud, float S, float Top, int32 Side, const FString& Position, int32 Number, const FString& Name, const FString& Right)
	{
		const float H = 27.f;
		const float X = BugX * S, Y = Top * S;
		Hud.DrawRect(BbLook::HudPanel, X, Y, BugW * S, H * S);
		Hud.DrawRect(BbLook::HudClub[Side & 1], X, Y, ChipW * S, H * S);
		Hud.DrawRect(BbLook::HudPanelLight, X + ChipW * S, Y + (H - 1.f) * S, (BugW - ChipW) * S, 1.f * S);
		const float Mid = Y + H * 0.5f * S;
		TextMid(Hud, Position, X + (ChipW + 10.f) * S, Mid, BbLook::HudDim, 0.9f * S);
		const FString Who = Number > 0 ? FString::Printf(TEXT("%d  %s"), Number, *Name) : Name;
		TextMid(Hud, Who, X + (ChipW + 44.f) * S, Mid, BbLook::HudText, 0.95f * S);
		TextRight(Hud, Right, X + (BugW - 10.f) * S, Mid, BbLook::HudGold, 0.95f * S);
		return Top + H;
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
	float Bottom = BugY + 2.f * RowH;
	// Who is facing whom: the pitcher in his club's colour, then the batter in his.
	const int32 Fielding = Bug.bTop ? 1 : 0;
	if (!Bug.Pitcher.IsEmpty())
	{
		Bottom = PersonRow(Hud, S, Bottom, Fielding, TEXT("P"), Bug.PitcherNumber, Bug.Pitcher, Bug.Pitches >= 0 ? FString::Printf(TEXT("%d P"), Bug.Pitches) : FString());
	}
	if (!Bug.Batter.IsEmpty())
	{
		Bottom = PersonRow(Hud, S, Bottom, 1 - Fielding, Bug.BatterPosition, Bug.BatterNumber, Bug.Batter, Bug.BatterToday);
	}
	return Bottom;
}

float BbHud::DrawTag(ABbHUD& Hud, float S, float Y, const FString& Text, const FLinearColor& TextColor)
{
	const float H = 24.f;
	Hud.DrawRect(BbLook::HudPanel, BugX * S, Y * S, BugW * S, H * S);
	TextMid(Hud, Text, (BugX + 10.f) * S, (Y + H * 0.5f) * S, TextColor, 0.95f * S);
	return Y + H;
}

namespace
{
	TArray<FString> WrapText(ABbHUD& Hud, const FString& Body, float MaxPx, float Scale)
	{
		TArray<FString> Out;
		FString Clean = Body;
		Clean.ReplaceInline(TEXT("\r\n"), TEXT("\n"));
		Clean.ReplaceInline(TEXT("\r"), TEXT("\n"));
		TArray<FString> Paras;
		Clean.ParseIntoArray(Paras, TEXT("\n"), false);
		for (const FString& Para : Paras)
		{
			if (Para.IsEmpty())
			{
				Out.Add(FString());
				continue;
			}
			TArray<FString> Words;
			Para.ParseIntoArray(Words, TEXT(" "), true);
			FString Line;
			auto Width = [&](const FString& T)
			{
				float W = 0.f, H = 0.f;
				Hud.GetTextSize(T, W, H, Font(), Scale);
				return W;
			};
			for (const FString& Word : Words)
			{
				const FString Cand = Line.IsEmpty() ? Word : (Line + TEXT(" ") + Word);
				if (!Line.IsEmpty() && Width(Cand) > MaxPx)
				{
					Out.Add(Line);
					Line = Word;
				}
				else
				{
					Line = Cand;
				}
				while (!Line.IsEmpty() && Width(Line) > MaxPx)
				{
					int32 Cut = Line.Len();
					while (Cut > 1 && Width(Line.Left(Cut)) > MaxPx)
					{
						--Cut;
					}
					Out.Add(Line.Left(Cut));
					Line = Line.RightChop(Cut);
				}
			}
			if (!Line.IsEmpty())
			{
				Out.Add(Line);
			}
		}
		return Out;
	}
}

float BbHud::DrawComment(ABbHUD& Hud, float S, float Y, const FString& Title, const FString& Body, const FLinearColor& Edge)
{
	const float TopBar = 4.f, PadX = 12.f, PadY = 10.f, Gap = 4.f;
	const float TitleScale = 0.85f, BodyScale = 0.9f;
	const float MaxPx = (BugW - 2.f * PadX) * S;
	const TArray<FString> Lines = WrapText(Hud, Body, MaxPx, BodyScale * S);
	float SampleW = 0.f, LinePx = 0.f, TitlePx = 0.f;
	Hud.GetTextSize(TEXT("Ag"), SampleW, LinePx, Font(), BodyScale * S);
	const float LineH = FMath::Max(LinePx / S, 14.f);
	float TitleH = 0.f;
	if (!Title.IsEmpty())
	{
		Hud.GetTextSize(Title, SampleW, TitlePx, Font(), TitleScale * S);
		TitleH = FMath::Max(TitlePx / S, 14.f) + 6.f;
	}
	const float BodyH = Lines.Num() > 0 ? Lines.Num() * (LineH + Gap) - Gap : 0.f;
	const float H = TopBar + PadY + TitleH + BodyH + PadY;
	Hud.DrawRect(BbLook::HudPanel, BugX * S, Y * S, BugW * S, H * S);
	Hud.DrawRect(Edge, BugX * S, Y * S, BugW * S, TopBar * S);
	float CurY = Y + TopBar + PadY;
	if (!Title.IsEmpty())
	{
		Hud.DrawText(Title, Edge, (BugX + PadX) * S, CurY * S, Font(), TitleScale * S);
		CurY += TitleH;
	}
	for (const FString& L : Lines)
	{
		if (!L.IsEmpty())
		{
			Hud.DrawText(L, BbLook::HudText, (BugX + PadX) * S, CurY * S, Font(), BodyScale * S);
		}
		CurY += LineH + Gap;
	}
	return Y + H;
}

void BbHud::DrawBanner(ABbHUD& Hud, float S, const FString& Title, const FString& Body, const FLinearColor& Edge)
{
	const float W = 620.f, PadX = 28.f, PadY = 18.f, Gap = 6.f;
	const float TitleScale = 1.45f, BodyScale = 1.05f;
	const TArray<FString> Lines = WrapText(Hud, Body, (W - 2.f * PadX) * S, BodyScale * S);
	float TW = 0.f, TitlePx = 0.f, BodyPx = 0.f;
	Hud.GetTextSize(Title, TW, TitlePx, Font(), TitleScale * S);
	Hud.GetTextSize(TEXT("Ag"), TW, BodyPx, Font(), BodyScale * S);
	const float TitleH = FMath::Max(TitlePx / S, 24.f);
	const float LineH = FMath::Max(BodyPx / S, 18.f);
	const float BodyH = Lines.Num() > 0 ? Lines.Num() * (LineH + Gap) - Gap : 0.f;
	const float H = 4.f + PadY + TitleH + (BodyH > 0.f ? 10.f + BodyH : 0.f) + PadY;
	const float Top = CentredPanel(Hud, S, W, H, 360.f + H * 0.5f, Edge, 1.f);
	const float CX = Hud.GetCanvas()->ClipX * 0.5f;
	float CurY = Top + 4.f + PadY;
	TextCentre(Hud, Title, CX, (CurY + TitleH * 0.5f) * S, Edge, TitleScale * S);
	CurY += TitleH + 10.f;
	for (const FString& L : Lines)
	{
		if (!L.IsEmpty())
		{
			TextCentre(Hud, L, CX, (CurY + LineH * 0.5f) * S, BbLook::HudText, BodyScale * S);
		}
		CurY += LineH + Gap;
	}
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

namespace
{
	// The line score's layout (reference units): the club, an inning to a column, runs, hits.
	constexpr float LineNameW = 150.f, LineRHW = 46.f, LineGap = 12.f, LineHeadH = 26.f;

	/** At least nine columns, as on a scoreboard; more once the game runs long. */
	int32 LineCols(const BbBoxScore& Box)
	{
		return FMath::Max<int32>(9, FMath::Max<int32>(Box.teams[0].n_batted, Box.teams[1].n_batted));
	}

	float LineColW(int32 Cols) { return Cols > 12 ? 27.f : 36.f; }

	float LineScoreW(const BbBoxScore& Box)
	{
		const int32 Cols = LineCols(Box);
		return ChipW + LineNameW + Cols * LineColW(Cols) + LineGap + 2.f * LineRHW;
	}

	/** The line score with its left edge at X and its top at Top (reference units): a row of
	 *  headings, then a row per club RowTall high. Returns its bottom edge. */
	float LineScore(ABbHUD& Hud, float S, float X, float Top, const BbBoxScore& Box, float RowTall, float Alpha)
	{
		const int32 Cols = LineCols(Box);
		const float ColW = LineColW(Cols);
		const float W = LineScoreW(Box);
		const float CellsX = X + ChipW + LineNameW;
		const float RX = CellsX + Cols * ColW + LineGap;
		const FLinearColor Text = Faded(BbLook::HudText, Alpha), Dim = Faded(BbLook::HudDim, Alpha);
		const float HeadMid = (Top + LineHeadH * 0.5f) * S;
		for (int32 C = 0; C < Cols; ++C)
		{
			TextCentre(Hud, FString::FromInt(Box.first_inning + C), (CellsX + (C + 0.5f) * ColW) * S, HeadMid, Dim, 0.85f * S);
		}
		TextCentre(Hud, TEXT("R"), (RX + LineRHW * 0.5f) * S, HeadMid, Text, 0.9f * S);
		TextCentre(Hud, TEXT("H"), (RX + LineRHW * 1.5f) * S, HeadMid, Dim, 0.9f * S);
		for (int32 Side = 0; Side < 2; ++Side)
		{
			const BbBoxTeam& T = Box.teams[Side];
			const float Y = Top + LineHeadH + Side * RowTall;
			const float Mid = (Y + RowTall * 0.5f) * S;
			Hud.DrawRect(Faded(BbLook::HudClub[Side], Alpha), X * S, Y * S, ChipW * S, RowTall * S);
			Hud.DrawRect(Faded(BbLook::HudPanelLight, Alpha), RX * S, Y * S, LineRHW * S, RowTall * S);
			TextMid(Hud, BbLook::Clubs[Side].Name, (X + ChipW + 10.f) * S, Mid, Text, 1.1f * S);
			for (int32 C = 0; C < Cols; ++C)
			{
				// An inning not batted yet is left empty; "x" for the home half of the last
				// inning that the home side did not need.
				FString Cell;
				if (C < T.n_batted)
				{
					Cell = FString::FromInt(T.runs[C]);
				}
				else if (Side == 1 && Box.is_final && C == Box.innings - Box.first_inning)
				{
					Cell = TEXT("x");
				}
				TextCentre(Hud, Cell, (CellsX + (C + 0.5f) * ColW) * S, Mid, C < T.n_batted && T.runs[C] > 0 ? Text : Dim, 1.05f * S);
			}
			TextCentre(Hud, FString::FromInt(T.totals.r), (RX + LineRHW * 0.5f) * S, Mid, Text, 1.3f * S);
			TextCentre(Hud, FString::FromInt(T.totals.h), (RX + LineRHW * 1.5f) * S, Mid, Dim, 1.1f * S);
		}
		// A hair under the headings and between the clubs.
		Hud.DrawRect(Faded(BbLook::HudOff, Alpha), X * S, (Top + LineHeadH - 1.f) * S, W * S, 1.f * S);
		Hud.DrawRect(Faded(BbLook::HudPanel, Alpha), X * S, (Top + LineHeadH + RowTall - 1.f) * S, W * S, 2.f * S);
		return Top + LineHeadH + 2.f * RowTall;
	}

	const TCHAR* PositionCode(int32 Pos)
	{
		static const TCHAR* Codes[] = { TEXT(""), TEXT("P"), TEXT("C"), TEXT("1B"), TEXT("2B"), TEXT("3B"), TEXT("SS"), TEXT("LF"), TEXT("CF"), TEXT("RF") };
		return Pos >= 1 && Pos <= 9 ? Codes[Pos] : TEXT("");
	}

	/** "Davis 2, Hayes" for the batters with a count in Pick; empty if none. */
	template <typename F>
	FString NoteList(const BbBoxTeam& T, F Pick)
	{
		FString Out;
		for (uint32 I = 0; I < T.n_lines; ++I)
		{
			const int32 N = Pick(T.lines[I]);
			if (N > 0)
			{
				Out += FString::Printf(TEXT("%s%s%s"), Out.IsEmpty() ? TEXT("") : TEXT(", "), UTF8_TO_TCHAR(T.lines[I].family), N > 1 ? *FString::Printf(TEXT(" %d"), N) : TEXT(""));
			}
		}
		return Out;
	}
}

void BbHud::DrawInningCard(ABbHUD& Hud, float S, const FString& Title, const int32 Runs[2], const BbBoxScore* Box, float Alpha)
{
	const float TitleH = 50.f;
	const float CX = Hud.GetCanvas()->ClipX * 0.5f;
	if (Box)
	{
		// The line score under the title, centred on the black.
		const float Row = 44.f, Pad = 14.f;
		const float W = LineScoreW(*Box) + 2.f * Pad;
		const float H = TitleH + LineHeadH + 2.f * Row + Pad;
		const float Top = CentredPanel(Hud, S, W, H, 360.f + H * 0.5f, BbLook::HudGold, Alpha);
		TextCentre(Hud, Title.ToUpper(), CX, (Top + 4.f + (TitleH - 4.f) * 0.5f) * S, Faded(BbLook::HudText, Alpha), 1.5f * S);
		LineScore(Hud, S, CX / S - W * 0.5f + Pad, Top + TitleH, *Box, Row, Alpha);
		return;
	}
	// Two club rows under a title.
	const float NameWide = 330.f, ScoreWide = 64.f, Row = 48.f;
	const float W = ChipW + NameWide + ScoreWide;
	const float H = TitleH + 2.f * Row;
	const float Bottom = 360.f + H * 0.5f;
	const float Top = CentredPanel(Hud, S, W, H, Bottom, BbLook::HudGold, Alpha);
	const float X = CX - W * 0.5f * S;
	TextCentre(Hud, Title.ToUpper(), CX, (Top + 4.f + (TitleH - 4.f) * 0.5f) * S, Faded(BbLook::HudText, Alpha), 1.5f * S);
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const FString Name = FString::Printf(TEXT("%s %s"), BbLook::Clubs[Side].Place, BbLook::Clubs[Side].Name);
		ClubRow(Hud, S, X, (Top + TitleH + Side * Row) * S, NameWide, ScoreWide, Row, Side, *Name, Runs[Side], 1.35f, 1.9f, Alpha);
	}
	Hud.DrawRect(Faded(BbLook::HudPanel, Alpha), X, (Top + TitleH + Row - 1.f) * S, W * S, 2.f * S);
}

void BbHud::DrawBoxScore(ABbHUD& Hud, float S, const BbBoxScore& Box, const FString& Title, const FString& Subtitle, float Alpha, float Dim)
{
	const float ClipW = Hud.GetCanvas()->ClipX, ClipH = Hud.GetCanvas()->ClipY;
	const FLinearColor Text = Faded(BbLook::HudText, Alpha), Quiet = Faded(BbLook::HudDim, Alpha);
	const FLinearColor Gold = Faded(BbLook::HudGold, Alpha), Hair = Faded(BbLook::HudOff, Alpha);
	// The field behind the panel. The layout is drawn for 1280 x 720.
	Hud.DrawRect(FLinearColor(0.f, 0.f, 0.f, Dim * Alpha), 0.f, 0.f, ClipW, ClipH);
	const float RefW = ClipW / S;
	const float PX = FMath::Max(20.f, (RefW - 1220.f) * 0.5f);
	const float PW = RefW - 2.f * PX;
	Hud.DrawRect(Faded(BbLook::HudPanel, Alpha), PX * S, 18.f * S, PW * S, 684.f * S);
	Hud.DrawRect(Gold, PX * S, 18.f * S, PW * S, 4.f * S);

	// The score in words, and whether the game is over.
	TextCentre(Hud, Title, ClipW * 0.5f, 56.f * S, Text, 2.1f * S);
	TextCentre(Hud, Subtitle, ClipW * 0.5f, 92.f * S, Gold, 1.05f * S);

	// The line score.
	const float LineRow = 30.f;
	const float LineBottom = LineScore(Hud, S, (RefW - LineScoreW(Box)) * 0.5f, 110.f, Box, LineRow, Alpha);

	// The batting lines, one table a side, the visitors on the left.
	const float TableY = LineBottom + 16.f;
	const float TableW = FMath::Min(580.f, (PW - 48.f) * 0.5f);
	const float StatW = 50.f, PosW = 46.f, RowTall = 27.f, HeadH = 30.f;
	const float StatsX = TableW - 6.f * StatW;
	static const TCHAR* Heads[6] = { TEXT("AB"), TEXT("R"), TEXT("H"), TEXT("RBI"), TEXT("BB"), TEXT("K") };
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const BbBoxTeam& T = Box.teams[Side];
		const float TX = Side == 0 ? PX + 24.f : PX + PW - 24.f - TableW;
		auto Stat = [&](int32 K, const FString& Str, float Mid, const FLinearColor& Col)
		{
			TextRight(Hud, Str, (TX + StatsX + (K + 1) * StatW - 8.f) * S, Mid * S, Col, 1.f * S);
		};
		// The club in its colour, and the men it left on.
		Hud.DrawRect(Faded(BbLook::HudPanelLight, Alpha), TX * S, TableY * S, TableW * S, HeadH * S);
		Hud.DrawRect(Faded(BbLook::HudClub[Side], Alpha), TX * S, TableY * S, ChipW * S, HeadH * S);
		const float HeadMid = TableY + HeadH * 0.5f;
		TextMid(Hud, FString::Printf(TEXT("%s %s"), BbLook::Clubs[Side].Place, BbLook::Clubs[Side].Name), (TX + ChipW + 10.f) * S, HeadMid * S, Text, 1.1f * S);
		TextRight(Hud, FString::Printf(TEXT("LOB %d"), T.totals.lob), (TX + TableW - 8.f) * S, HeadMid * S, Quiet, 0.9f * S);
		const float ColsMid = TableY + HeadH + RowTall * 0.5f;
		for (int32 K = 0; K < 6; ++K)
		{
			Stat(K, Heads[K], ColsMid, Quiet);
		}
		const float RowTop = TableY + HeadH + RowTall;
		Hud.DrawRect(Hair, TX * S, (RowTop - 2.f) * S, TableW * S, 1.f * S);
		for (uint32 I = 0; I < T.n_lines; ++I)
		{
			const BbBatLine& L = T.lines[I];
			const float Mid = RowTop + (I + 0.5f) * RowTall;
			TextMid(Hud, PositionCode(L.position), (TX + ChipW + 2.f) * S, Mid * S, Quiet, 0.9f * S);
			TextMid(Hud, UTF8_TO_TCHAR(L.family), (TX + ChipW + 2.f + PosW) * S, Mid * S, Text, 1.f * S);
			const int32 Vals[6] = { L.ab, L.r, L.h, L.rbi, L.bb, L.k };
			for (int32 K = 0; K < 6; ++K)
			{
				// A hit or a run batted in is what the eye looks for; a zero is quieter.
				Stat(K, FString::FromInt(Vals[K]), Mid, Vals[K] > 0 ? Text : Quiet);
			}
		}
		const float TotTop = RowTop + T.n_lines * RowTall + 2.f;
		Hud.DrawRect(Hair, TX * S, (TotTop - 2.f) * S, TableW * S, 1.f * S);
		const float TotMid = TotTop + RowTall * 0.5f;
		TextMid(Hud, TEXT("Totals"), (TX + ChipW + 2.f + PosW) * S, TotMid * S, Text, 1.f * S);
		const int32 Tot[6] = { T.totals.ab, T.totals.r, T.totals.h, T.totals.rbi, T.totals.bb, T.totals.k };
		for (int32 K = 0; K < 6; ++K)
		{
			Stat(K, FString::FromInt(Tot[K]), TotMid, Text);
		}
		// Under the table: the extra-base hits, sacrifice flies and hit batters, then the
		// club's pitchers with the pitches each threw.
		float NY = TotTop + RowTall + 16.f;
		auto Note = [&](const TCHAR* Label, const FString& Names)
		{
			if (!Names.IsEmpty())
			{
				TextMid(Hud, Label, (TX + ChipW + 2.f) * S, NY * S, Gold, 0.82f * S);
				TextMid(Hud, Names, (TX + ChipW + 2.f + PosW) * S, NY * S, Text, 0.82f * S);
				NY += 21.f;
			}
		};
		Note(TEXT("2B"), NoteList(T, [](const BbBatLine& L) { return (int32)L.doubles; }));
		Note(TEXT("3B"), NoteList(T, [](const BbBatLine& L) { return (int32)L.triples; }));
		Note(TEXT("HR"), NoteList(T, [](const BbBatLine& L) { return (int32)L.hr; }));
		Note(TEXT("SF"), NoteList(T, [](const BbBatLine& L) { return (int32)L.sf; }));
		Note(TEXT("HBP"), NoteList(T, [](const BbBatLine& L) { return (int32)L.hbp; }));
		FString Arms;
		for (int32 I = 0; I < Box.n_pitchers[Side]; ++I)
		{
			const BbPitchLine& P = Box.pitchers[Side][I];
			Arms += FString::Printf(TEXT("%s%s %d pitches"), I > 0 ? TEXT(", ") : TEXT(""), UTF8_TO_TCHAR(P.family), (int32)P.pitches);
		}
		Note(TEXT("P"), Arms);
	}
}

namespace
{
	// The zone box's layout (reference units): metres to units, and what of the plane shows.
	constexpr float ZoneK = 175.f;
	constexpr float ZoneHalfSpanM = 0.44f, ZoneLowM = 0.22f, ZoneHighM = 1.36f;
	/** A mark's radius: the ball is drawn half again as large as it is (its centre is where it
	 *  crossed), so its number can be read. */
	constexpr float ZoneMarkR = 1.5f * 0.0366f * ZoneK;
	constexpr float ZonePad = 12.f, ZoneLegendH = 40.f;

	FLinearColor CallColor(uint8 Call)
	{
		switch (Call)
		{
		case 1: return BbLook::HudBall;
		case 2: return BbLook::HudClose;
		case 6: return BbLook::HudInPlay;
		case 7: return BbLook::HudDim;
		default: return BbLook::HudGold; // swinging strike, foul, foul tip
		}
	}

	void Disc(ABbHUD& Hud, FVector2D C, float R, const FLinearColor& Col)
	{
		Hud.GetCanvas()->K2_DrawPolygon(nullptr, C, FVector2D(R, R), 24, Col);
	}
}

void BbHud::DrawZone(ABbHUD& Hud, float S, const FBbZoneInfo& Zone)
{
	if (!Zone.bValid)
	{
		return;
	}
	const float PlaneW = 2.f * ZoneHalfSpanM * ZoneK, PlaneH = (ZoneHighM - ZoneLowM) * ZoneK;
	const float W = PlaneW + 2.f * ZonePad, H = PlaneH + 2.f * ZonePad + ZoneLegendH;
	const float X0 = Hud.GetCanvas()->ClipX / S - 20.f - W, Y0 = BugY;
	Hud.DrawRect(BbLook::HudPanel, X0 * S, Y0 * S, W * S, H * S);
	// From the pitcher's side the first-base side (+x) is on the left.
	const float CX = X0 + ZonePad + PlaneW * 0.5f;
	auto At = [&](float X, float Z) { return FVector2D((CX - X * ZoneK) * S, (Y0 + ZonePad + (ZoneHighM - Z) * ZoneK) * S); };
	// The zone: its thirds faint, its edge clear.
	const FVector2D TL = At(Zone.HalfWidth, Zone.Top), BR = At(-Zone.HalfWidth, Zone.Bottom);
	const float ZW = BR.X - TL.X, ZH = BR.Y - TL.Y, L = 1.5f * S;
	Hud.DrawRect(BbLook::HudPanelLight, TL.X, TL.Y, ZW, ZH);
	for (int32 I = 1; I < 3; ++I)
	{
		Hud.DrawRect(BbLook::HudOff, TL.X + ZW * I / 3.f, TL.Y, 1.f * S, ZH);
		Hud.DrawRect(BbLook::HudOff, TL.X, TL.Y + ZH * I / 3.f, ZW, 1.f * S);
	}
	Hud.DrawRect(BbLook::HudText, TL.X, TL.Y, ZW, L);
	Hud.DrawRect(BbLook::HudText, TL.X, BR.Y - L, ZW, L);
	Hud.DrawRect(BbLook::HudText, TL.X, TL.Y, L, ZH);
	Hud.DrawRect(BbLook::HudText, BR.X - L, TL.Y, L, ZH);
	// The pitches, centred where they crossed; the newest ringed in white.
	const float R = ZoneMarkR * S;
	for (int32 I = 0; I < Zone.Marks.Num(); ++I)
	{
		const FBbZoneMark& M = Zone.Marks[I];
		const FVector2D P = At(FMath::Clamp(M.X, -ZoneHalfSpanM, ZoneHalfSpanM), FMath::Clamp(M.Z, ZoneLowM, ZoneHighM));
		if (I == Zone.Marks.Num() - 1)
		{
			Disc(Hud, P, R + 2.f * S, BbLook::HudText);
		}
		Disc(Hud, P, R, CallColor(M.Call));
		TextCentre(Hud, FString::FromInt(I + 1), P.X, P.Y, BbLook::HudPanel, 0.72f * S);
	}
	// The key.
	struct FKeyItem { uint8 Call; const TCHAR* Name; };
	static const FKeyItem Keys[4] = { { 1, TEXT("Ball") }, { 2, TEXT("Called strike") }, { 3, TEXT("Swing, foul") }, { 6, TEXT("In play") } };
	for (int32 I = 0; I < 4; ++I)
	{
		const float KX = X0 + ZonePad + (I % 2) * (W - 2.f * ZonePad) * 0.5f;
		const float KY = Y0 + ZonePad + PlaneH + 8.f + (I / 2) * 16.f;
		Disc(Hud, FVector2D((KX + 5.f) * S, (KY + 6.f) * S), 4.5f * S, CallColor(Keys[I].Call));
		TextMid(Hud, Keys[I].Name, (KX + 14.f) * S, (KY + 6.f) * S, BbLook::HudDim, 0.68f * S);
	}
}
