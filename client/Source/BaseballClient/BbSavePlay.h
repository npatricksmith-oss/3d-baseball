#pragma once

#include "CoreMinimal.h"
#include "Widgets/SCompoundWidget.h"

class SMultiLineEditableTextBox;

/**
 * Saving and loading a play from the instant replay (owner, 2026-11-15; D9 2026-11-18).
 * The replay shows a "Save this play" button (or the S key); it opens a dialog with Cancel
 * and Save, the game's seed and the play's number, and a box for a comment. Save writes
 * one .toml and its companion .timeline blob to saved-plays/ at the top of the repository,
 * stamped with the build's git commit, so the play plays back identically on later builds.
 */
namespace BbSavePlay
{
	/** The git commit this client was built from (short sha, with "-dirty", or "unknown"). */
	FString BuildCommit();

	/** What a saved file records besides the comment. */
	struct FWhat
	{
		uint64 Seed = 0;
		uint32 Play = 0;
		uint8 Inning = 0;
		bool bTop = true;
		uint8 Outs = 0;
		uint8 Balls = 0;
		uint8 Strikes = 0;
		uint8 AwayRuns = 0;
		uint8 HomeRuns = 0;
	};

	/** What Read() loads from a saved-play .toml file. */
	struct FLoaded
	{
		FWhat What;
		FString Commit;
		bool bCommitInferred = false;
		FString Timeline;
		FString TimelinePath;
		FString Saved;
		FString Comment;
		bool bHasSeed = false;
		bool bHasPlay = false;
	};

	/** saved-plays/ at the repository's top (the client project's parent folder). */
	FString Folder();

	/** Resolves a -BbSavedPlay=<path> argument against CWD, the repo root, and saved-plays/. */
	FString ResolveTomlPath(const FString& Arg);

	/** Writes <Dir>/seed<seed>-play<n>-<yyyymmdd-hhmmss>.toml and .timeline; returns the
	 *  TOML's full path, or an empty string if either could not be written. */
	FString WriteTo(const FString& Dir, const FWhat& What, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen);

	/** Writes into saved-plays/ via WriteTo. */
	FString Write(const FWhat& What, const FString& Comment, const uint8* TimelineBlob, size_t TimelineLen);

	/** Reads a saved-play .toml file. False if the file cannot be read. */
	bool Read(const FString& TomlPath, FLoaded& Out);
}

DECLARE_DELEGATE_OneParam(FBbOnSavePlay, const FString& /*Comment*/);

/** The dialog: Cancel and Save across the top, "[SEED 7   PLAY 123]", "Comment:", and the
 *  text box. Esc cancels, Ctrl+Enter saves; Enter alone starts a new line. It floats: placed
 *  by its Offset from the screen's right and top edges, and dragged by any part that is not
 *  a button or the text box. */
class SBbSavePlayDialog : public SCompoundWidget
{
public:
	SLATE_BEGIN_ARGS(SBbSavePlayDialog) {}
		SLATE_ARGUMENT(FText, Title)
		SLATE_ARGUMENT(FVector2D, Offset)
		SLATE_EVENT(FSimpleDelegate, OnCancel)
		SLATE_EVENT(FBbOnSavePlay, OnSave)
	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs);
	/** The comment box, to give it keyboard focus when the dialog opens. */
	TSharedPtr<SMultiLineEditableTextBox> Box;
	virtual FReply OnKeyDown(const FGeometry& Geometry, const FKeyEvent& Key) override;
	virtual bool SupportsKeyboardFocus() const override { return true; }
	virtual FReply OnMouseButtonDown(const FGeometry& Geometry, const FPointerEvent& Mouse) override;
	virtual FReply OnMouseMove(const FGeometry& Geometry, const FPointerEvent& Mouse) override;
	virtual FReply OnMouseButtonUp(const FGeometry& Geometry, const FPointerEvent& Mouse) override;
	/** Distance from the screen's right edge (X) and top edge (Y), never negative. */
	FVector2D Offset = FVector2D::ZeroVector;
	/** The overlay slot's padding that puts it there. */
	FMargin Margin() const { return FMargin(0.f, Offset.Y, Offset.X, 0.f); }

private:
	FSimpleDelegate OnCancel;
	FBbOnSavePlay OnSave;
	bool bDragging = false;
	FReply Cancel();
	FReply Save();
};
