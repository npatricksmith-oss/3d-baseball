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

namespace BbSavePlay
{
	FString Folder()
	{
		return FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT(".."), TEXT("saved-plays")));
	}

	FString Write(const FWhat& What, const FString& Comment)
	{
		const FDateTime Now = FDateTime::Now();
		const FString Dir = Folder();
		IFileManager::Get().MakeDirectory(*Dir, true);
		const FString Path = FPaths::Combine(Dir, FString::Printf(TEXT("seed%llu-play%u-%s.toml"), What.Seed, What.Play, *Now.ToString(TEXT("%Y%m%d-%H%M%S"))));
		// TOML. The comment is a multi-line basic string: backslashes and quotes escaped.
		FString Body = Comment.Replace(TEXT("\r"), TEXT(""));
		Body = Body.Replace(TEXT("\\"), TEXT("\\\\")).Replace(TEXT("\""), TEXT("\\\""));
		const FString Text = FString::Printf(
			TEXT("# A play saved from the instant replay (client BbSavePlay).\n")
			TEXT("# See it again: client/run.sh %llu, then R back to play %u once it has been shown.\n")
			TEXT("seed = %llu\n")
			TEXT("play = %u\n")
			TEXT("inning = %u\n")
			TEXT("half = \"%s\"\n")
			TEXT("# The board before the pitch.\n")
			TEXT("outs = %u\n")
			TEXT("count = \"%u-%u\"\n")
			TEXT("score = \"away %u, home %u\"\n")
			TEXT("saved = %s\n")
			TEXT("comment = \"\"\"\n%s\n\"\"\"\n"),
			What.Seed, What.Play, What.Seed, What.Play, What.Inning, What.bTop ? TEXT("top") : TEXT("bottom"),
			What.Outs, What.Balls, What.Strikes, What.AwayRuns, What.HomeRuns,
			*Now.ToString(TEXT("%Y-%m-%dT%H:%M:%S")), *Body);
		if (!FFileHelper::SaveStringToFile(Text, *Path, FFileHelper::EEncodingOptions::ForceUTF8WithoutBOM))
		{
			return FString();
		}
		return Path;
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
