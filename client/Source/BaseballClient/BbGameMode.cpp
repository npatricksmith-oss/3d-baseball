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
	if (bBoxData && BoxAlpha > 0.f)
	{
		DrawBox(S);
	}
}

namespace
{
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

void ABbHUD::DrawBox(float S)
{
	UFont* Font = GEngine->GetLargeFont();
	const float A = BoxAlpha;
	const float W = Canvas->ClipX;
	const float H = Canvas->ClipY;
	// The field behind the paper.
	DrawRect(FLinearColor(0.f, 0.f, 0.f, BoxDim * A), 0.f, 0.f, W, H);
	// Newspaper stock and ink. One reference unit is S pixels; the layout is drawn for 1280 x 720.
	const FLinearColor Paper(0.93f, 0.90f, 0.80f, A);
	const FLinearColor Ink(0.09f, 0.08f, 0.07f, A);
	const FLinearColor Faint(0.09f, 0.08f, 0.07f, 0.72f * A);
	const float RefW = W / S;
	const float PaperX = FMath::Max(20.f, (RefW - 1220.f) * 0.5f);
	const float PaperW = RefW - 2.f * PaperX;
	DrawRect(Paper, PaperX * S, 18.f * S, PaperW * S, 684.f * S);
	DrawRect(Ink, (PaperX + 8.f) * S, 26.f * S, (PaperW - 16.f) * S, 2.f * S);
	DrawRect(Ink, (PaperX + 8.f) * S, 31.f * S, (PaperW - 16.f) * S, 1.f * S);

	const float TextScale = 1.0f * S;
	auto Right = [&](const FString& Str, float RightEdge, float Y, const FLinearColor& Col, float Scale)
	{
		float TW = 0.f, TH = 0.f;
		GetTextSize(Str, TW, TH, Font, Scale);
		DrawText(Str, Col, RightEdge * S - TW, Y * S, Font, Scale);
	};
	auto Centered = [&](const FString& Str, float CenterX, float Y, const FLinearColor& Col, float Scale)
	{
		float TW = 0.f, TH = 0.f;
		GetTextSize(Str, TW, TH, Font, Scale);
		DrawText(Str, Col, CenterX * S - TW * 0.5f, Y * S, Font, Scale);
	};

	// Title: the score in words, and whether the game is over.
	Centered(BoxTitle, RefW * 0.5f, 36.f, Ink, 2.1f * S);
	Centered(BoxSubtitle, RefW * 0.5f, 84.f, Faint, 1.1f * S);

	// The linescore: a row per side, an inning to a column, then runs and hits. The innings
	// shown start at Box.first_inning; at least nine columns, as in the paper.
	const int32 Cols = FMath::Max<int32>(9, FMath::Max<int32>(Box.teams[0].n_batted, Box.teams[1].n_batted));
	const float ColW = Cols > 12 ? 30.f : 38.f;
	const float NameW = 120.f, RHW = 46.f, Gap = 22.f;
	const float LineW = NameW + Cols * ColW + Gap + 2.f * RHW;
	float X0 = (RefW - LineW) * 0.5f;
	float Y = 116.f;
	const float RowH = 29.f;
	for (int32 C = 0; C < Cols; ++C)
	{
		Right(FString::FromInt(Box.first_inning + C), X0 + NameW + (C + 1) * ColW - 6.f, Y, Faint, TextScale);
	}
	Right(TEXT("R"), X0 + NameW + Cols * ColW + Gap + RHW - 6.f, Y, Ink, TextScale);
	Right(TEXT("H"), X0 + NameW + Cols * ColW + Gap + 2.f * RHW - 6.f, Y, Ink, TextScale);
	DrawRect(Ink, X0 * S, (Y + RowH - 2.f) * S, LineW * S, 1.5f * S);
	static const TCHAR* TeamName[2] = { TEXT("AWAY"), TEXT("HOME") };
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const BbBoxTeam& T = Box.teams[Side];
		const float RowY = Y + RowH * (Side + 1);
		DrawText(TeamName[Side], Ink, X0 * S, RowY * S, Font, TextScale);
		for (int32 C = 0; C < Cols; ++C)
		{
			// A side that did not bat an inning shows a dash ("x" if the game ended without the
			// home half of the last inning).
			const FString Cell = C < T.n_batted ? FString::FromInt(T.runs[C]) : (Side == 1 && Box.is_final && C == Box.innings - Box.first_inning ? TEXT("x") : TEXT("-"));
			Right(Cell, X0 + NameW + (C + 1) * ColW - 6.f, RowY, C < T.n_batted ? Ink : Faint, TextScale);
		}
		Right(FString::FromInt(T.totals.r), X0 + NameW + Cols * ColW + Gap + RHW - 6.f, RowY, Ink, TextScale);
		Right(FString::FromInt(T.totals.h), X0 + NameW + Cols * ColW + Gap + 2.f * RHW - 6.f, RowY, Ink, TextScale);
	}
	DrawRect(Faint, X0 * S, (Y + RowH * 3.f + 2.f) * S, LineW * S, 1.f * S);

	// The batting lines, one table a side, Away on the left.
	const float TableY = Y + RowH * 3.f + 14.f;
	const float TableW = FMath::Min(580.f, (PaperW - 48.f) * 0.5f);
	const float StatW = 50.f, PosW = 46.f;
	const float TableNameW = TableW - PosW - 6.f * StatW;
	static const TCHAR* Heads[6] = { TEXT("AB"), TEXT("R"), TEXT("H"), TEXT("RBI"), TEXT("BB"), TEXT("K") };
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const BbBoxTeam& T = Box.teams[Side];
		const float TX = Side == 0 ? PaperX + 24.f : PaperX + PaperW - 24.f - TableW;
		DrawText(Side == 0 ? TEXT("AWAY") : TEXT("HOME"), Ink, TX * S, TableY * S, Font, 1.15f * S);
		Right(FString::Printf(TEXT("LOB %d"), T.totals.lob), TX + TableW - 6.f, TableY + 3.f, Faint, TextScale);
		const float HeadY = TableY + 30.f;
		for (int32 K = 0; K < 6; ++K)
		{
			Right(Heads[K], TX + TableNameW + PosW + (K + 1) * StatW - 6.f, HeadY, Faint, TextScale);
		}
		DrawRect(Ink, TX * S, (HeadY + RowH - 3.f) * S, TableW * S, 1.5f * S);
		const float RowTop = HeadY + RowH;
		for (uint32 I = 0; I < T.n_lines; ++I)
		{
			const BbBatLine& L = T.lines[I];
			const float RY = RowTop + I * 27.f;
			DrawText(FString::Printf(TEXT("%s"), UTF8_TO_TCHAR(L.family)), Ink, TX * S, RY * S, Font, TextScale);
			DrawText(PositionCode(L.position), Faint, (TX + TableNameW) * S, RY * S, Font, TextScale);
			const int32 Vals[6] = { L.ab, L.r, L.h, L.rbi, L.bb, L.k };
			for (int32 K = 0; K < 6; ++K)
			{
				// A hit or a run batted in is what the eye looks for: ink; a zero is quieter.
				Right(FString::FromInt(Vals[K]), TX + TableNameW + PosW + (K + 1) * StatW - 6.f, RY, Vals[K] > 0 ? Ink : Faint, TextScale);
			}
		}
		const float TotY = RowTop + T.n_lines * 27.f + 2.f;
		DrawRect(Ink, TX * S, (TotY - 2.f) * S, TableW * S, 1.5f * S);
		DrawText(TEXT("Totals"), Ink, TX * S, (TotY + 1.f) * S, Font, TextScale);
		const int32 Tot[6] = { T.totals.ab, T.totals.r, T.totals.h, T.totals.rbi, T.totals.bb, T.totals.k };
		for (int32 K = 0; K < 6; ++K)
		{
			Right(FString::FromInt(Tot[K]), TX + TableNameW + PosW + (K + 1) * StatW - 6.f, TotY + 1.f, Ink, TextScale);
		}
		// Notes in the paper's way: the extra-base hits, sacrifice flies, hit batters, and men left on.
		float NY = TotY + 33.f;
		const float NoteScale = 0.82f * S;
		auto Note = [&](const TCHAR* Label, const FString& Names)
		{
			if (!Names.IsEmpty())
			{
				DrawText(FString::Printf(TEXT("%s  %s"), Label, *Names), Ink, TX * S, NY * S, Font, NoteScale);
				NY += 22.f;
			}
		};
		Note(TEXT("2B"), NoteList(T, [](const BbBatLine& L) { return (int32)L.doubles; }));
		Note(TEXT("3B"), NoteList(T, [](const BbBatLine& L) { return (int32)L.triples; }));
		Note(TEXT("HR"), NoteList(T, [](const BbBatLine& L) { return (int32)L.hr; }));
		Note(TEXT("SF"), NoteList(T, [](const BbBatLine& L) { return (int32)L.sf; }));
		Note(TEXT("HBP"), NoteList(T, [](const BbBatLine& L) { return (int32)L.hbp; }));
	}
}
