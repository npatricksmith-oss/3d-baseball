#include "BbSavePlay.h"

#include "HAL/FileManager.h"
#include "Misc/DateTime.h"
#include "Misc/FileHelper.h"
#include "Misc/Paths.h"
#include "Styling/CoreStyle.h"
#include "Widgets/Input/SButton.h"
#include "Widgets/Input/SMultiLineEditableTextBox.h"
#include "Widgets/Layout/SBorder.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SSpacer.h"
#include "Widgets/SBoxPanel.h"
#include "Widgets/Text/STextBlock.h"

#define BB_STR_IMPL(x) #x
#define BB_STR(x) BB_STR_IMPL(x)
#ifndef BB_GIT_COMMIT
#define BB_GIT_COMMIT unknown
#endif

namespace
{
	FString UnquoteBasic(const FString& S)
	{
		FString Out;
		Out.Reserve(S.Len());
		for (int32 I = 0; I < S.Len(); ++I)
		{
			if (S[I] == TEXT('\\') && I + 1 < S.Len())
			{
				const TCHAR N = S[++I];
				switch (N)
				{
				case TEXT('\\'): Out.AppendChar(TEXT('\\')); break;
				case TEXT('"'): Out.AppendChar(TEXT('"')); break;
				case TEXT('n'): Out.AppendChar(TEXT('\n')); break;
				case TEXT('r'): Out.AppendChar(TEXT('\r')); break;
				case TEXT('t'): Out.AppendChar(TEXT('\t')); break;
				default: Out.AppendChar(N); break;
				}
			}
			else
			{
				Out.AppendChar(S[I]);
			}
		}
		return Out;
	}

	FString ParseStringValue(const FString& Val)
	{
		FString S = Val.TrimStartAndEnd();
		if (S.Len() >= 2 && S[0] == TEXT('"') && S[S.Len() - 1] == TEXT('"'))
		{
			return UnquoteBasic(S.Mid(1, S.Len() - 2));
		}
		return S;
	}
}

namespace BbSavePlay
{
	FString BuildCommit()
	{
		return FString(ANSI_TO_TCHAR(BB_STR(BB_GIT_COMMIT)));
	}

	FString Folder()
	{
		return FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT(".."), TEXT("saved-plays")));
	}

	FString ResolveTomlPath(const FString& Arg)
	{
		if (Arg.IsEmpty())
		{
			return FString();
		}
		if (FPaths::FileExists(Arg))
		{
			return FPaths::ConvertRelativePathToFull(Arg);
		}
		const FString FromRepo = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT(".."), Arg));
		if (FPaths::FileExists(FromRepo))
		{
			return FromRepo;
		}
		const FString FromFolder = FPaths::Combine(Folder(), Arg);
		if (FPaths::FileExists(FromFolder))
		{
			return FromFolder;
		}
		return Arg;
	}

	FString WriteTo(const FString& Dir, const FWhat& What, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen)
	{
		if (!TimelineBlob || TimelineLen == 0)
		{
			return FString();
		}
		const FDateTime Now = FDateTime::Now();
		IFileManager::Get().MakeDirectory(*Dir, true);
		const FString Stem = FString::Printf(TEXT("seed%llu-play%u-%s"), What.Seed, What.Play, *Now.ToString(TEXT("%Y%m%d-%H%M%S")));
		FString Base = Stem;
		for (int32 Suffix = 2; IFileManager::Get().FileExists(*FPaths::Combine(Dir, Base + TEXT(".toml"))) || IFileManager::Get().FileExists(*FPaths::Combine(Dir, Base + TEXT(".timeline"))); ++Suffix)
		{
			Base = FString::Printf(TEXT("%s-%d"), *Stem, Suffix);
		}
		const FString TimelineName = Base + TEXT(".timeline");
		const FString TimelinePath = FPaths::Combine(Dir, TimelineName);
		const FString TomlName = Base + TEXT(".toml");
		const FString Path = FPaths::Combine(Dir, TomlName);
		if (!FFileHelper::SaveArrayToFile(TArrayView<const uint8>(TimelineBlob, static_cast<int32>(TimelineLen)), *TimelinePath))
		{
			return FString();
		}
		const FString Commit = BuildCommit();
		const FString SeeArg = FPaths::IsSamePath(Dir, Folder()) ? FString::Printf(TEXT("saved-plays/%s"), *TomlName) : Path;
		// TOML. The comment is a multi-line basic string: backslashes and quotes escaped.
		FString Body = Comment.Replace(TEXT("\r"), TEXT(""));
		Body = Body.Replace(TEXT("\\"), TEXT("\\\\")).Replace(TEXT("\""), TEXT("\\\""));
		const FString Text = FString::Printf(
			TEXT("# A play saved from the instant replay (client BbSavePlay).\n")
			TEXT("# See it again: client/run.sh -BbSavedPlay=%s\n")
			TEXT("seed = %llu\n")
			TEXT("play = %u\n")
			TEXT("commit = \"%s\"\n")
			TEXT("timeline = \"%s\"\n")
			TEXT("inning = %u\n")
			TEXT("half = \"%s\"\n")
			TEXT("# The board before the pitch.\n")
			TEXT("outs = %u\n")
			TEXT("count = \"%u-%u\"\n")
			TEXT("score = \"away %u, home %u\"\n")
			TEXT("saved = %s\n")
			TEXT("comment = \"\"\"\n%s\n\"\"\"\n"),
			*SeeArg, What.Seed, What.Play, *Commit, *TimelineName, What.Inning, What.bTop ? TEXT("top") : TEXT("bottom"),
			What.Outs, What.Balls, What.Strikes, What.AwayRuns, What.HomeRuns,
			*Now.ToString(TEXT("%Y-%m-%dT%H:%M:%S")), *Body);
		if (!FFileHelper::SaveStringToFile(Text, *Path, FFileHelper::EEncodingOptions::ForceUTF8WithoutBOM))
		{
			IFileManager::Get().Delete(*TimelinePath);
			return FString();
		}
		return Path;
	}

	FString Write(const FWhat& What, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen)
	{
		return WriteTo(Folder(), What, Comment, TimelineBlob, TimelineLen);
	}

	bool Read(const FString& TomlPath, FLoaded& Out)
	{
		Out = FLoaded();
		FString Text;
		if (!FFileHelper::LoadFileToString(Text, *TomlPath))
		{
			return false;
		}
		Text = Text.Replace(TEXT("\r\n"), TEXT("\n")).Replace(TEXT("\r"), TEXT("\n"));
		TArray<FString> Lines;
		Text.ParseIntoArray(Lines, TEXT("\n"), false);
		bool bInMultiline = false;
		FString MultilineKey;
		FString MultilineRaw;
		for (const FString& RawLine : Lines)
		{
			if (bInMultiline)
			{
				const int32 CloseIdx = RawLine.Find(TEXT("\"\"\""));
				if (CloseIdx != INDEX_NONE && (CloseIdx == 0 || RawLine[CloseIdx - 1] != TEXT('\\')))
				{
					MultilineRaw += RawLine.Left(CloseIdx);
					if (MultilineKey == TEXT("comment"))
					{
						if (MultilineRaw.EndsWith(TEXT("\n")))
						{
							MultilineRaw.LeftChopInline(1);
						}
						Out.Comment = UnquoteBasic(MultilineRaw);
					}
					bInMultiline = false;
					MultilineKey.Reset();
					MultilineRaw.Reset();
				}
				else
				{
					MultilineRaw += RawLine + TEXT("\n");
				}
				continue;
			}
			const FString Trimmed = RawLine.TrimStartAndEnd();
			if (Trimmed.IsEmpty() || Trimmed.StartsWith(TEXT("#")))
			{
				continue;
			}
			const int32 Eq = Trimmed.Find(TEXT("="));
			if (Eq == INDEX_NONE)
			{
				continue;
			}
			const FString Key = Trimmed.Left(Eq).TrimStartAndEnd();
			const FString Val = Trimmed.RightChop(Eq + 1).TrimStartAndEnd();
			if (Val.StartsWith(TEXT("\"\"\"")))
			{
				const FString Rest = Val.RightChop(3);
				const int32 CloseIdx = Rest.Find(TEXT("\"\"\""));
				if (CloseIdx != INDEX_NONE)
				{
					if (Key == TEXT("comment"))
					{
						Out.Comment = UnquoteBasic(Rest.Left(CloseIdx));
					}
				}
				else
				{
					bInMultiline = true;
					MultilineKey = Key;
					MultilineRaw = Rest.IsEmpty() ? FString() : (Rest + TEXT("\n"));
				}
				continue;
			}
			if (Key == TEXT("seed"))
			{
				Out.What.Seed = FCString::Strtoui64(*Val, nullptr, 10);
				Out.bHasSeed = true;
			}
			else if (Key == TEXT("play"))
			{
				Out.What.Play = (uint32)FCString::Atoi(*Val);
				Out.bHasPlay = true;
			}
			else if (Key == TEXT("commit"))
			{
				Out.Commit = ParseStringValue(Val);
			}
			else if (Key == TEXT("commit_inferred"))
			{
				Out.bCommitInferred = Val == TEXT("true");
			}
			else if (Key == TEXT("timeline"))
			{
				Out.Timeline = ParseStringValue(Val);
			}
			else if (Key == TEXT("inning"))
			{
				Out.What.Inning = (uint8)FCString::Atoi(*Val);
			}
			else if (Key == TEXT("half"))
			{
				Out.What.bTop = ParseStringValue(Val) != TEXT("bottom");
			}
			else if (Key == TEXT("outs"))
			{
				Out.What.Outs = (uint8)FCString::Atoi(*Val);
			}
			else if (Key == TEXT("count"))
			{
				FString B, S;
				if (ParseStringValue(Val).Split(TEXT("-"), &B, &S))
				{
					Out.What.Balls = (uint8)FCString::Atoi(*B);
					Out.What.Strikes = (uint8)FCString::Atoi(*S);
				}
			}
			else if (Key == TEXT("score"))
			{
				FString S = ParseStringValue(Val);
				FString Left, Right;
				if (S.Split(TEXT(","), &Left, &Right))
				{
					Left = Left.TrimStartAndEnd();
					Right = Right.TrimStartAndEnd();
					if (Left.StartsWith(TEXT("away ")))
					{
						Out.What.AwayRuns = (uint8)FCString::Atoi(*Left.RightChop(5));
					}
					if (Right.StartsWith(TEXT("home ")))
					{
						Out.What.HomeRuns = (uint8)FCString::Atoi(*Right.RightChop(5));
					}
				}
			}
			else if (Key == TEXT("saved"))
			{
				Out.Saved = ParseStringValue(Val);
			}
			else if (Key == TEXT("comment"))
			{
				Out.Comment = ParseStringValue(Val);
			}
		}
		if (!Out.Timeline.IsEmpty())
		{
			Out.TimelinePath = FPaths::IsRelative(Out.Timeline)
				? FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::GetPath(TomlPath), Out.Timeline))
				: Out.Timeline;
		}
		return true;
	}
}

void SBbSavePlayDialog::Construct(const FArguments& InArgs)
{
	OnCancel = InArgs._OnCancel;
	OnSave = InArgs._OnSave;
	Offset = InArgs._Offset;
	const FSlateFontInfo ButtonFont = FCoreStyle::GetDefaultFontStyle("Bold", 15);
	const FSlateFontInfo TitleFont = FCoreStyle::GetDefaultFontStyle("Bold", 18);
	const FSlateFontInfo LabelFont = FCoreStyle::GetDefaultFontStyle("Regular", 15);
	const FSlateFontInfo TypeFont = FCoreStyle::GetDefaultFontStyle("Regular", 14);
	ChildSlot
	[
		SNew(SBox)
		.WidthOverride(500.f)
		.HeightOverride(340.f)
		[
			SNew(SBorder)
			.BorderImage(FCoreStyle::Get().GetBrush("WhiteBrush"))
			.BorderBackgroundColor(FLinearColor(0.06f, 0.07f, 0.09f, 0.96f))
			.Padding(18.f)
			.Cursor(EMouseCursor::CardinalCross)
			[
				SNew(SVerticalBox)
				+ SVerticalBox::Slot().AutoHeight()
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot().AutoWidth()
					[
						SNew(SButton)
						.IsFocusable(false)
						.ContentPadding(FMargin(18.f, 6.f))
						.OnClicked(this, &SBbSavePlayDialog::Cancel)
						[
							SNew(STextBlock).Text(FText::FromString(TEXT("Cancel"))).Font(ButtonFont)
						]
					]
					+ SHorizontalBox::Slot().FillWidth(1.f)
					[
						SNew(SSpacer)
					]
					+ SHorizontalBox::Slot().AutoWidth()
					[
						SNew(SButton)
						.IsFocusable(false)
						.ContentPadding(FMargin(18.f, 6.f))
						.OnClicked(this, &SBbSavePlayDialog::Save)
						[
							SNew(STextBlock).Text(FText::FromString(TEXT("Save"))).Font(ButtonFont)
						]
					]
				]
				+ SVerticalBox::Slot().AutoHeight().Padding(4.f, 14.f, 4.f, 0.f)
				[
					SNew(STextBlock).Text(InArgs._Title).Font(TitleFont).ColorAndOpacity(FLinearColor(1.f, 0.85f, 0.2f))
				]
				+ SVerticalBox::Slot().AutoHeight().Padding(4.f, 12.f, 4.f, 6.f)
				[
					SNew(STextBlock).Text(FText::FromString(TEXT("Comment:"))).Font(LabelFont).ColorAndOpacity(FLinearColor::White)
				]
				+ SVerticalBox::Slot().FillHeight(1.f)
				[
					SAssignNew(Box, SMultiLineEditableTextBox)
					.Cursor(EMouseCursor::TextEditBeam)
					.Font(TypeFont)
					.AutoWrapText(true)
					.HintText(FText::FromString(TEXT("What happened on this play?   (Ctrl+Enter saves, Esc cancels)")))
					.OnKeyDownHandler_Lambda([this](const FGeometry& G, const FKeyEvent& K) { return OnKeyDown(G, K); })
				]
			]
		]
	];
}

FReply SBbSavePlayDialog::OnKeyDown(const FGeometry& Geometry, const FKeyEvent& Key)
{
	if (Key.GetKey() == EKeys::Escape)
	{
		return Cancel();
	}
	if (Key.GetKey() == EKeys::Enter && Key.IsControlDown())
	{
		return Save();
	}
	return FReply::Unhandled();
}

FReply SBbSavePlayDialog::OnMouseButtonDown(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	// Reached only when no button or the text box took the click: the window's own surface.
	if (Mouse.GetEffectingButton() != EKeys::LeftMouseButton)
	{
		return FReply::Unhandled();
	}
	bDragging = true;
	return FReply::Handled().CaptureMouse(SharedThis(this));
}

FReply SBbSavePlayDialog::OnMouseMove(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	if (!bDragging || !HasMouseCapture())
	{
		return FReply::Unhandled();
	}
	// Screen pixels to Slate units; the offset is from the right edge, so X runs backwards.
	const FVector2D D = Mouse.GetCursorDelta() / FMath::Max(Geometry.Scale, 0.01f);
	Offset.X = FMath::Max(0.f, Offset.X - D.X);
	Offset.Y = FMath::Max(0.f, Offset.Y + D.Y);
	return FReply::Handled();
}

FReply SBbSavePlayDialog::OnMouseButtonUp(const FGeometry& Geometry, const FPointerEvent& Mouse)
{
	if (!bDragging)
	{
		return FReply::Unhandled();
	}
	bDragging = false;
	return FReply::Handled().ReleaseMouseCapture();
}

FReply SBbSavePlayDialog::Cancel()
{
	OnCancel.ExecuteIfBound();
	return FReply::Handled();
}

FReply SBbSavePlayDialog::Save()
{
	OnSave.ExecuteIfBound(Box.IsValid() ? Box->GetText().ToString() : FString());
	return FReply::Handled();
}
