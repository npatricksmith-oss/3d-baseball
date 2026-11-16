#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Async/Future.h"
#include "BbSim.h"
#include "BbPose.h"
#include "BbDirector.generated.h"

class AStaticMeshActor;
class ABbPark;
class UBbFigureShop;
class ASkeletalMeshActor;
class USkeletalMesh;
class UAnimSequence;
class ACameraActor;
class UStaticMesh;
class UMaterialInterface;

/**
 * Plays the sim's timelines in the park that BbPark builds: capsule players,
 * the ball, a broadcast camera, and the scoreboard. The sim runs on a worker thread, one
 * pitch ahead; nothing here decides anything about the game (PLAN §3.4, D4).
 *
 * Command line: -BbSimLib=<path to libbb_ffi.so> -BbSeed=<n> -BbSpeed=<x>
 *               -BbCamera=<0 broadcast | 1 behind home | 2 overhead>
 *               -BbShots=<n> (take n screenshots, then quit) -BbShotEvery=<seconds> -BbShotStart=<seconds>
 *               -BbFigureShots (both nines stood in a row, front and back, one figure turned
 *               through eight angles, and the kit, as figure_<n>_<what>.png, then quit; no game
 *               is played);
 *               -BbMannequin (the grey mannequin instead of the figures);
 *               -BbStockMotions (the stock clips in place of the coded delivery, crouch, swing,
 *               catch and throw);
 *               -BbMotionShots (the delivery, the catcher's receive, a ball hit, a swing and a
 *               miss, a pitch taken, a ball fielded and thrown and a throw taken photographed
 *               at their named moments as motion_<strip>_<n>_<what>.png, then quit);
 *               -BbParkShots (photograph the park from its fixed spots as park_<spot>.png, then quit;
 *               -BbParkShotsAt=<seconds> when to start)
 *               -BbBoxShots=<n> (photograph the Tab box at pitch n and the final box, then quit)
 *               -BbDiveShots=<n> (photograph n dives: driving off, in the air, down, on hands
 *               and knees, getting up; then quit)
 *               -BbSlideShots=<n> (photograph n slides: going down, sliding, at the bag, sitting
 *               up, getting up; then quit; with -BbStagingLog each slide is logged as BB_SLIDE)
 *               -BbAnimTest (stock clips looping) or -BbAnimTest=dive (dive pose candidates;
 *               -BbAnimFrac=<0..1> -BbAnimTilt=<degrees>)
 *               -BbBenchSeconds=<s> (log frame rate and memory, then quit)
 *               -BbStagingLog (log each reset, each player leaving the field and each throw's
 *               first animation phase as BB_RESET / BB_EXIT / BB_THROW, and a BB_STAGING summary)
 *               -BbRushAt=<s> (press Enter once, that many seconds in; logs BB_RUSH)
 *               -BbReplayAt=<s> (press R twice that many seconds in, F six seconds later; logs BB_REPLAY)
 *               -BbSaveAt=<s> (press S that many seconds in and Save a second later; logs BB_SAVE)
 *               -BbClipSpeeds (log the ground speed each gait clip is animated for as BB_CLIP)
 */
UCLASS()
class ABbDirector : public AActor
{
	GENERATED_BODY()

public:
	ABbDirector();
	virtual void BeginPlay() override;
	virtual void Tick(float DeltaSeconds) override;
	virtual void EndPlay(const EEndPlayReason::Type Reason) override;

private:
	AStaticMeshActor* Shape(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FLinearColor& Color, const FRotator& Rotation = FRotator::ZeroRotator);
	/** The sim's frame of a play at T as it is drawn: on the drawn ground (the mound). */
	bool SampleDrawn(const FBbPlay& Play, float T, BbFrame& Out) const;
	void RequestNext();
	void BeginPlayback(FBbPlay&& Play);
	void UpdateCamera(const BbFrame& Frame, float DeltaSeconds);
	/** Puts the ball's shadow under the ball, sized for the camera as it now stands. */
	void PlaceBallShadow(const BbFrame& Frame, bool bShown);
	void Finish();
	/** Reads the box score of the first Through pitches from the sim (A12), once per change. */
	void RefreshBox(uint32 Through);
	/** The box-score screen: held with Tab during the game, final at its end. */
	void UpdateBoxScreen(float DeltaSeconds, bool bTabHeld);
	/** Words for the box's title and subtitle. */
	void BoxTitles(FString& Title, FString& Subtitle) const;

	TUniquePtr<FBbSim> Sim;
	FBbPlay Current;
	TFuture<FBbPlay> Pending;
	bool bPending = false;
	bool bGameOver = false;
	float PlayTime = 0.f;
	float Speed = 1.f;
	float HoldAfter = 0.f;
	/** The scoreboard as the current play started (the previous play's result). During a
	 *  play the outs on it advance as the umpire calls each one. */
	BbPlayInfo Before{};
	/** How long the umpire's out signal stays up, s of play time. */
	static constexpr float UmpireShowS = 2.0f;
	/** A play this close (runner's time short of the base, s) is called with emphasis. */
	static constexpr float CloseCallS = 0.15f;
	/** Calls of the current play already logged (BB_CALL). */
	int32 CallsLogged = 0;
	/** Seconds shown before each pitch's release: the pitcher's wind-up. */
	float PreRoll = -BbMotion::DeliveryStartS;

	/** Everything that stands still: field, walls, stands, lights (BbPark). */
	UPROPERTY()
	TObjectPtr<ABbPark> Park;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Cube;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Sphere;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Cylinder;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Plane;
	UPROPERTY()
	TObjectPtr<UMaterialInterface> BaseMaterial;
	UPROPERTY()
	TArray<TObjectPtr<AStaticMeshActor>> Bodies;
	UPROPERTY()
	TArray<TObjectPtr<AStaticMeshActor>> Heads;
	// Human figures (the engine's mannequin, copied into Content by client/run.sh). When
	// the assets are missing the cylinders above are used instead.
	UPROPERTY()
	TObjectPtr<USkeletalMesh> FigureMesh;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimIdle;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimWalk;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimJog;
	// Stand-ins from the engine's stock clips until real baseball animation exists: a
	// wind-up-and-strike for the pitcher's delivery, a punch for a fielder's throw, a landing
	// crouch for the catcher, a guard stance for the batter.
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimPitch;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimThrow;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimCrouch;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimStance;
	// A dive: a stock fall, played forward as he goes down and backward as he gets up.
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimDive;
	// Sidestep and back-step walks, for a short shuffle that does not turn the body.
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimWalkBack;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimWalkLeft;
	UPROPERTY()
	TObjectPtr<UAnimSequence> AnimWalkRight;
	/** Figures (and cylinder stand-ins): the first MaxActors show the frame's actors, the
	 *  rest (MaxExiting) players still walking off the field after their last pitch. */
	UPROPERTY()
	TArray<TObjectPtr<ASkeletalMeshActor>> Figures;
	struct FFigureState
	{
		uint16 Agent = 0xFFFF;
		FVector Pos = FVector::ZeroVector;
		float Yaw = 0.f;
		int32 Gait = -1; // 0 idle, 1 walk, 2 jog, 3 back-step, 4 sidestep left, 5 sidestep right
		float SpeedCm = 0.f;
		int32 Team = -1;
		// The clip being driven by hand (null while the gait clips play on their own).
		const UAnimSequence* Manual = nullptr;
		// A coded motion (BbPose.h): which (0 none, 1 the delivery, 2 the catcher's crouch,
		// 3 the batter at the plate, 4 a fielder's ready, catch and throw, 5 a dive, 6 a
		// slide), how much of it shows over the clip, and the motion's moment last shown.
		int32 Motion = 0;
		float MotionW = 0.f;
		float MotionT = 0.f;
		/** A fielder (motion 4): how far down in the ready position he is, how much he is
		 *  carrying the ball in his hands, and how much his legs are the pose's rather than
		 *  the running clip's. Eased on play time. */
		float Ready = 0.f, Carry = 0.f, Lower = 1.f;
		/** Where the figure is drawn from the sim's position, cm on the ground: a motion can
		 *  carry him off it (the pitcher finishes down the mound, the catcher sits back of
		 *  his spot), and he steps back in his own time. */
		FVector2D Off = FVector2D::ZeroVector;
		/** The ball in his hand (or glove) when the pose has it: the pitcher's before he lets
		 *  go, a fielder's from the catch to the throw. */
		bool bHand = false;
		FVector Hand = FVector::ZeroVector;
	};
	/** The mannequin's joints, for the coded motions. */
	FBbSkeleton Skel;
	/** How long a figure takes to go into a coded motion or come out of one, s of play time. */
	static constexpr float MotionBlendS = 0.3f;
	/** The pitcher comes set, and the catcher squats, this long before the reset ends. */
	static constexpr float SetLeadS = 0.8f;
	/** A pitcher off the rubber after his delivery steps back at this pace between pitches,
	 *  and is gathered in this fast once the play has him running, cm/s. */
	static constexpr float StepBackCmps = 130.f;
	static constexpr float GatherCmps = 300.f;
	/** The pitch on screen: where the ball is let go; where and when it ends in the mitt (or
	 *  would have, had it not been hit); until when the catcher stays down. */
	FVector PitchFrom = FVector::ZeroVector;
	FVector PitchEndAt = FVector::ZeroVector;
	float PitchEndT = -1.f;
	float CrouchUntil = 0.6f;
	void NotePitch();
	/** The catcher holds the ball he caught this long before he comes up, s. */
	static constexpr float ReceiveHoldS = 0.7f;
	/** The mitt goes from its target to the ball over this long, ending this long before the
	 *  ball is there, s. */
	static constexpr float ReceiveMoveS = 0.22f;
	static constexpr float ReceiveEarlyS = 0.03f;
	/** -BbMotionShots: the first pitches photographed at the motions' named moments, from
	 *  beside the pitcher, from the plate, and beside the catcher, as motion_<pitch>_<n>_<what>.png;
	 *  then quit. The play is held on each moment exactly. */
	bool bMotionShots = false;
	int32 MotionShotNext = 0;
	/** Which strip the pitch on screen is photographed for, or -1: 0 to 2 as above; then 3
	 *  the first swing, from in front of the batter; 4 the next swing, from the mound's side;
	 *  5 the first pitch taken; 6 the first ball fielded and thrown, from beside the fielder;
	 *  7 the next throw taken by a fielder, from beside him. */
	int32 MotionStrip = -1;
	bool bStripDone[8] = { false, false, false, false, false, false, false, false };
	/** The catch and the throw that strips 6 and 7 are about (indices into Catches, Throws). */
	int32 StripCatch = -1, StripThrow = -1;
	TArray<FFigureState> FigureState;
	/** What PoseFigure knows beyond the frame: the client's own plan for a staged move, and
	 *  a throw whose release time is known (so its clip phase never depends on which frames
	 *  happened to be drawn). */
	struct FPoseInput
	{
		/** Between pitches: posed from the planned move, not from measured speed. */
		bool bStaged = false;
		/** Walking off the field: no baseball motions. */
		bool bExiting = false;
		/** Under way on the planned path (set off and not yet arrived). */
		bool bMoving = false;
		/** Turned to face along the path (a move long enough to turn for, from just before
		 *  he sets off until he arrives); otherwise a short move is a shuffle facing the plate. */
		bool bFacePath = false;
		FVector2D Dir = FVector2D::ZeroVector; // sim frame, unit
		float PlannedMps = 0.f;
		/** Seconds into AnimThrow (its release is ThrowReleaseS in), or negative for no throw. */
		float ThrowClip = -1.f;
		float ThrowYaw = 0.f; // Unreal yaw, degrees
		/** The coded throw: seconds from the release, and where the sim lets the ball go. */
		bool bThrow = false;
		float ThrowT = 0.f;
		bool bThrowFrom = false;
		FVector ThrowFrom = FVector::ZeroVector;
		/** A catch of his near in time: seconds since it (negative before), and where the
		 *  ball is as he takes it. */
		bool bCatch = false;
		float CatchDt = 0.f;
		FVector CatchAt = FVector::ZeroVector;
		/** He has the ball. */
		bool bHolds = false;
		/** Down from a dive (the sim's BbDive): seconds since he left his feet, or negative
		 *  for none; seconds until he is up; which way he dove (Unreal yaw, if bDiveDir). */
		float DiveSince = -1.f;
		float DiveUpIn = 0.f;
		bool bDiveDir = false;
		float DiveYaw = 0.f;
		/** A runner's slide: seconds from his arrival at the bag, and which way he came. */
		bool bSlide = false;
		float SlideT = 0.f;
		float SlideYaw = 0.f;
	};
	void PoseFigure(int32 Slot, const BbActor& A, const BbFrame& Frame, const FPoseInput& In, float SimDt, float DeltaSeconds);
	/** A dive's layout (client presentation): he tips forward to DiveTiltDeg from the feet
	 *  over DiveFallS (the sim's lunge takes 0.3 s) and comes down DiveDropCm onto the turf
	 *  (the leap clip, held DiveClipFrac through, has him in the air and leaning 45° already),
	 *  lies there, and gets up over the last DiveRiseS before the sim has him up. Tipped
	 *  flat, his middle is DiveBodyBackCm from his feet. Check with -BbDiveShots. */
	/** The drawn dive begins this long before the sim's (PoseFigure's caller says why), s. */
	static constexpr float DiveLeadS = 0.17f;
	float DiveFallS = 0.3f;
	float DiveRiseS = 0.6f;
	float DiveTiltDeg = 45.f;
	float DiveTiltSign = -1.f;
	float DiveClipFrac = 0.5f;
	float DiveBodyBackCm = 90.f;
	float DiveDropCm = -40.f;
	/** Places a figure at Foot facing Yaw (Unreal degrees), tipped forward TiltDeg and raised LiftCm. */
	void PlaceDiver(ASkeletalMeshActor* F, const FVector& Foot, float Yaw, float TiltDeg, float LiftCm) const;
	/** The throw clip: its release this far in, and how long it runs. */
	static constexpr float ThrowReleaseS = 0.45f;
	static constexpr float ThrowClipS = 0.9f;
	/** A move shorter than this is a shuffle (no turn); longer, he turns first. */
	float ShuffleMaxM = 1.5f;
	/** Time a player who is turning to go takes before he sets off, s. */
	float ResetTurnS = 0.25f;
	/** Rate a figure's facing eases toward where it should face, per s of play time. */
	float TurnRate = 12.f;

	/** A throw in the current timeline, as soon as any sampled frame shows it coming. */
	struct FThrowNote
	{
		uint16 Agent = 0;
		float ReleaseAt = 0.f; // play time
		float Yaw = 0.f;       // Unreal yaw toward the ball's flight
		float FirstClip = -1.f; // the clip time first drawn (for -BbStagingLog), or -1
		/** Where the sim lets the ball go (the ball's place as its flight starts). */
		bool bFrom = false;
		FVector From = FVector::ZeroVector;
	};
	TArray<FThrowNote> Throws;
	/** A fielder taking the ball in the current timeline (a batted ball or a throw): when,
	 *  and where the ball is at that moment. Read from the whole timeline as the pitch comes
	 *  up (NoteCatches), so the glove can be on its way before the ball is there. */
	struct FCatchNote
	{
		uint16 Agent = 0;
		float T = 0.f;
		FVector At = FVector::ZeroVector;
		/** The ball one step of the sim earlier (SimTickS): from there the timeline's ball
		 *  jumps to the sim's holding point over his feet, so the drawn ball is carried on
		 *  along its own line into the glove instead. */
		FVector Prev = FVector::ZeroVector;
		/** It was thrown to him (someone held it before). */
		bool bThrown = false;
	};
	TArray<FCatchNote> Catches;
	void NoteCatches();
	/** A runner who slides in the current timeline (player-visuals §11). The sim has no
	 *  slide: a runner going on to a base he stops at runs into it at full speed and is still
	 *  from the next moment. Where a play is being made on him at second, third or home that
	 *  stop is drawn as a feet-first slide (the client's choice of how to show what the sim
	 *  did; nothing feeds back). */
	struct FSlideNote
	{
		uint16 Agent = 0;
		float T = 0.f;   // play time the sim has him at the bag
		float Yaw = 0.f; // Unreal yaw of his run into it
		int32 Base = 0;  // 2, 3 or 4 (home)
		FVector At = FVector::ZeroVector;
	};
	TArray<FSlideNote> Slides;
	void NoteSlides();
	/** He slides if he comes in at least this fast, m/s, and the ball is taken within
	 *  SlidePlayM of the bag within SlidePlayS of his getting there (or is already held
	 *  there, or he is called out there). */
	static constexpr float SlideMinMps = 5.f;
	static constexpr float SlidePlayM = 5.f;
	static constexpr float SlidePlayS = 1.5f;
	/** The live sim's step, s: the timeline has the ball at each one (bb_game::live DT). */
	static constexpr float SimTickS = 1.f / 60.f;
	void AddThrow(uint16 Agent, float ReleaseAt);
	/** The fielders stay down in the ready position until this play time. */
	float ReadyUntil = 0.8f;
	/** The glove sets out for the ball this long before the catch and is there this long
	 *  before it; it stays this long after, and has come in to the hands by GloveInS. */
	static constexpr float GloveOutS = 0.36f;
	static constexpr float GloveThereS = 0.04f;
	static constexpr float GloveStayS = 0.10f;
	static constexpr float GloveInS = 0.32f;
	/** Slower than this as the ball arrives, he turns to face it; faster, he takes it on
	 *  the run, facing where he runs, cm/s. */
	static constexpr float SettleMaxCmps = 300.f;
	/** Play time of the last frame sampled for throws; frames between it and the next are
	 *  sampled too, so fast playback cannot step over a throw. */
	float ThrowsSampledTo = 0.f;
	void NoteThrows(float From, float To);

	// Between pitches: the ball goes back to the pitcher and everyone walks or jogs to where
	// the next pitch starts; players leaving the field head for their dugout. This is the
	// client's own staging of dead time. The sim does not model it and nothing here feeds back.
	struct FResetMover
	{
		uint16 Agent = 0;
		FVector2D From = FVector2D::ZeroVector; // sim frame, metres
		FVector2D To = FVector2D::ZeroVector;
		float Depart = 0.f;
		/** Cruising speed; he speeds up to it and slows from it at ResetAccelMps2. */
		float SpeedMps = 0.f;
	};
	/** Seconds to cover Dist from rest to rest, cruising at V with acceleration A. */
	static float TravelSeconds(float Dist, float V, float A);
	/** Distance covered T seconds after setting off, on the same profile. */
	static float TravelGone(float Dist, float V, float A, float T);
	/** The cruising speed that covers Dist in Seconds (rest to rest), or Cap if none under it does. */
	static float SpeedFor(float Dist, float Seconds, float A, float Cap);
	/** The ground speed the mannequin's jog clip (MF_Unarmed_Jog_Fwd) is animated for, m/s:
	 *  its root bone travels 6.00 m/s over the 1.77 s loop, and its planted feet pass back
	 *  under the root at 5.9 m/s while sliding under 0.3 m/s themselves (measured with
	 *  -BbClipSpeeds, 2026-11-13). The jog clip plays at a figure's speed over this, so his
	 *  feet keep pace with the ground. */
	static constexpr float JogClipMps = 6.0f;
	/** At a change of sides, everyone going off (the fielders, the retired batter, runners
	 *  left on) and the fielders coming on jog at this steady pace, speeding up and slowing
	 *  at ResetAccelMps2 at the ends. It plays the jog clip at 0.8 of its own rate: about 2.8
	 *  strides of 1.7 m a second, an easy ballplayer's jog (a home-run trot is 4.5–5 m/s).
	 *  At the clip's own 6 m/s, 3.5 strides a second, he would be running. */
	static constexpr float SideChangeJogMps = 4.8f;
	// Staging values (client presentation only). Players walk a short way and jog easily
	// farther; to keep the next pitch from waiting past ResetMaxSeconds they quicken, but
	// never past ResetHurryMps (a brisk jog). Someone farther than that allows, such as a
	// pitcher who chased a fly ball into the outfield, takes as long as it takes. Nobody
	// starts or stops dead: speed builds and falls at ResetAccelMps2. A change of sides is
	// staged differently: everyone jogs at SideChangeJogMps, however far.
	float ResetWalkMps = 1.6f;
	float ResetJogMps = 3.6f;
	float ResetHurryMps = 5.f;
	float ResetAccelMps2 = 3.f;
	float ResetWalkWithinM = 6.f;
	float ResetMaxSeconds = 6.f;
	float ResetEnterMaxM = 30.f;
	float ResetThrowMps = 22.f;
	bool bReset = false;
	float ResetTime = 0.f;
	float ResetDuration = 0.f;
	// The cut at a change of sides (owner, 2026-11-14; to be a game setting): the jog off and
	// on takes 23-27 s, so the picture stays for SideCutLeadS while the sides start off, fades
	// to black over SideCutFadeS, skips the middle of the jog, holds the between-innings card
	// for SideCutHoldS, and fades back in SideCutTailS before the first pitch, as the fielders
	// reach their spots. Nothing in the staging changes: both staging clocks jump together
	// while the picture is black. -BbSideCut=0 plays the whole changeover.
	bool bSideCut = true;
	float SideCutLeadS = 4.f;
	float SideCutFadeS = 0.5f;
	float SideCutHoldS = 1.5f;
	float SideCutTailS = 3.5f;
	/** This reset is a change of sides long enough to cut. */
	bool bSideCutPlanned = false;
	/** The jump over the middle of the jog has been made. */
	bool bSideCutJumped = false;
	/** How black the picture is (0 clear, 1 black) at ResetTime; makes the jump when it gets there. */
	float SideCutFade();
	FBbPlay NextPlay;
	TArray<FResetMover> Movers;
	/** The planned pose of a staged mover at time T on his own clock. */
	FPoseInput StagedInput(const FResetMover& M, float T) const;
	/** Speed on the TravelGone profile T seconds after setting off. */
	static float TravelSpeed(float Dist, float V, float A, float T);

	// Players leaving the field (the retired batter, a runner who scored, the side going in)
	// walk to their dugout on their own clock, through the reset and into the next pitch if
	// need be, in figures of their own; at a change of sides the next pitch waits for them.
	static constexpr int32 MaxExiting = 24;
	struct FExitSlot
	{
		bool bActive = false;
		FResetMover Move; // Depart is on ExitClock
	};
	TArray<FExitSlot> Exits;
	float ExitClock = 0.f;
	/** Starts the walk off for Agent from From (a steady jog at SideChangeJogMps if bJog);
	 *  returns his seconds to the dugout. */
	float AddExit(const TArray<FFigureState>& Old, uint16 Agent, const FVector2D& From, bool bJog);
	/** Takes anyone in Frame out of the exit walk (he is in the next pitch after all). */
	void PurgeExits(const BbFrame& Frame);
	void EndExit(int32 Index, const TCHAR* Reason);
	FVector2D ExitAt(const FResetMover& M) const;
	/** Copies a figure's place, facing and speed from the slot that showed Agent until now. */
	void CarryFigure(const TArray<FFigureState>& Old, int32 Slot, uint16 Agent);
	FVector2D Dugout(uint16 Agent) const;
	/** Index into Movers of whoever returns the ball, or -1 when no ball is thrown back. */
	int32 ResetThrower = -1;
	FVector ResetBallFrom = FVector::ZeroVector; // sim frame, metres
	/** Where the returner lets the ball go: beside his head on his throwing side. */
	FVector ResetHand = FVector::ZeroVector;
	FVector ResetBallTo = FVector::ZeroVector;
	float ResetThrowAt = 0.f;
	float ResetFlight = 0.f;
	bool StartReset(const FBbPlay& Next);
	/** The staged frame at reset time T; false in bBallShown while the ball is in a glove. */
	void ResetFrame(float T, BbFrame& Out, bool& bBallShown) const;
	/** First throw-clip time drawn for the reset's throw (for -BbStagingLog), or -1. */
	float ResetThrowFirstClip = -1.f;
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> Ball;
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> BallShadow;
	/** Diameter the ball is drawn at, m: its real size, 7.4 cm (owner, 2026-11-16; it was twice
	 *  that). Engine primitives are 1 m across at scale 1. */
	static constexpr float BallDrawnM = 0.075f;
	/** Smallest diameter of the ball's ground shadow with the ball on the ground, m: the drawn
	 *  ball's own size, as the sun's shadow of a ball is. Close to the camera it stays this size. */
	static constexpr float BallShadowM = BallDrawnM;
	/** With the ball on the ground, a far shadow keeps at least this share of the view's width
	 *  across (about 5.6 px of a 1600 px view; it was twice that until the owner asked for
	 *  half the size, 2026-11-16), so it can still be seen deep in the outfield.
	 *  The ball's height then scales it like the 15 cm floor (below). Sized so the shadow is never larger on
	 *  screen than the 50 cm disc the owner found too big (2026-11-12): from the follow camera
	 *  a ball in the near infield gets about 11 x 6.5 px against that disc's 18 x 6; at the
	 *  pitch camera's long lens the 15 cm floor governs. */
	static constexpr float BallShadowViewShare = 0.0035f;
	/** The shadow's rim: it fades from solid to nothing over this share of its radius (owner,
	 *  2026-11-16: half as big as before and 30% more blurred; before, the edge was hard). */
	static constexpr float BallShadowSoftFrac = 0.3f;
	/** The shadow's size follows the ball's height (owner, 2026-11-15). At or below
	 *  BallShadowNoneM (the ball's centre resting or rolling on the ground, with a little room
	 *  for low skips) there is none; it opens to the sizes above by BallShadowFullM, and grows
	 *  to BallShadowHighScale times that by BallShadowHighM, so a high fly's shadow is the
	 *  largest and closes up as the ball comes down. */
	static constexpr float BallShadowNoneM = 0.12f;
	static constexpr float BallShadowFullM = 2.0f;
	static constexpr float BallShadowHighM = 30.0f;
	static constexpr float BallShadowHighScale = 2.5f;
	/** Most the shadow is drawn longer along the line of sight than across it. A camera low
	 *  over a far ball sees the disc nearly edge-on (19 m up and 155 m away it shows an eighth
	 *  of its width); the length makes up the square root of that, so the shadow still reads
	 *  as lying flat on the ground but stays a few pixels tall (deep fly: 11 x 3.4 px). */
	static constexpr float BallShadowMaxStretch = 2.5f;
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> Bat;
	/** The bat is the figure shop's turned one (its knob at its origin), not the cylinder. */
	bool bTurnedBat = false;
	void PlaceBat(const FVector& Knob, const FVector& Tip);
	/** The bat through one pitch, read from its timeline when the pitch comes up (NoteBat):
	 *  who stands where with it, how he holds it in the stance, and, if he swings, when the
	 *  bat leaves the stance, when his hands are furthest back (and where round him they are
	 *  then, BbMotion::HandsDeg), and when and where the sim's bat comes to rest. */
	struct FBatNote
	{
		bool bBat = false;
		/** Where the batter stands, sim m, and whether it is the first-base side's box (a
		 *  left-handed batter: shown in a mirror). */
		FVector2D Spot = FVector2D::ZeroVector;
		bool bLeftBox = false;
		FVector StanceKnob = FVector::ZeroVector, StanceTip = FVector::ZeroVector;
		bool bSwing = false;
		float StartT = -1.f, LaunchT = -1.f, EndT = -1.f;
		float LaunchDeg = 0.f;
		FVector EndKnob = FVector::ZeroVector, EndTip = FVector::ZeroVector;
	};
	FBatNote BatNote, NextBatNote;
	void NoteBat(const FBbPlay& Play, FBatNote& Out) const;
	/** A sim point in the batter's own frame (BbMotion::FSwingIn), cm, without the figure. */
	static FVector BatterFrame(const float P[3], const FBatNote& N);
	/** The bat as drawn this frame. The sim's while the sim has it in his hands (bBatSim);
	 *  the batter's own to carry otherwise, as long as he stands at the plate (bBatHeld): back
	 *  from the end of his swing to his shoulder, at ease between pitches, up into the stance. */
	FVector BatKnob = FVector::ZeroVector, BatTip = FVector(0, 0, 86);
	bool bBatSim = false, bBatHeld = false;
	uint16 BatHolder = 0xFFFF;
	/** How much of a swing is on, the front foot's step, how much he is at ease (FSwingIn),
	 *  and the point his eyes are on (eased, so his head does not snap round after a pitch). */
	float BatSwingW = 0.f, BatStride = 0.f, BatRest = 1.f;
	FVector BatLook = FVector::ZeroVector;
	/** After a swing the bat comes back to his shoulder over this long, s. */
	static constexpr float BatReturnS = 0.8f;
	static constexpr double BatLenCm = 86.36;
	UPROPERTY()
	TObjectPtr<ACameraActor> Camera;

	FVector CamLook = FVector::ZeroVector;
	bool bBallInPlay = false;
	/** The ball left the playing field and is dead (a foul or a throw out of play, a fair ball
	 *  bounding out; BbPlayInfo::dead_s, A10): the camera has stopped following it. */
	bool bBallDead = false;
	/** The cut back to the field has been made for this play. */
	bool bDeadCut = false;
	/** A home run (BbPlayInfo.dead_reason 2; A16). The sim's play ends as the ball clears
	 *  the fence; the timeline then has the batter and runners trot round the bases to home
	 *  (bb_game::timeline, presentation only), and the play's duration covers it. */
	bool bHomeRun = false;
	/** A foul ball into the seats (BbPlayInfo.dead_reason 4): no walk back after it; the next
	 *  pitch starts with everyone already in place. */
	bool bFoulOut = false;
	/** How long the field is shown after a foul into the seats before the next pitch, s. */
	static constexpr float FoulOutHoldS = 1.0f;
	/** The ball is over the wall: the camera is in center field and the card is up. */
	bool bHomeRunCut = false;
	/** The trot's view: from beyond center field, looking in at the diamond, wide enough for
	 *  all four bases (Unreal field of view, degrees). */
	static constexpr float HrViewFov = 30.f;
	/** After the last runner scores, the view holds this long before the reset, s. */
	static constexpr float HrHoldS = 1.5f;
	/** The card's lines: the batter's family name, "412 ft   108 mph", what it scored. */
	FString HrName, HrLine, HrKind;

	// The box score (A12). Box is the sim's reading of the log through BoxThrough pitches.
	BbBoxScore Box{};
	bool bBoxValid = false;
	uint32 BoxThrough = 0;
	/** The same reading for the outfield board: through the pitches whose result is known. */
	BbBoxScore BoardBox{};
	uint32 BoardThrough = 0xFFFFFFFEu;
	bool bBoardValid = false;
	/** How far the Tab view is up (0 to 1; it comes and goes in BoxPeekS). */
	float BoxPeek = 0.f;
	static constexpr float BoxPeekS = 0.15f;
	/** The game is over: the picture fades to black, then the box fades in and stays. */
	bool bBoxFinal = false;
	float BoxFinalTime = 0.f;
	static constexpr float BoxFadeS = 1.0f;
	static constexpr float BoxInS = 0.7f;
	/** Wall seconds an unattended run waits on the final box before it quits. */
	float BoxHoldS = 3.f;
	// -BbBoxShots=<n>: unattended check. At pitch n the Tab view is held for a moment and
	// photographed (box_midgame.png), and at the end of the game the final box is (box_final.png).
	int32 BoxShotAtPlay = -1;
	float BoxShotWall = -1.f;
	bool bBoxMidShot = false;
	bool bBoxFinalShot = false;

	/** 0 broadcast (cuts), 1 behind home, 2 overhead. */
	int32 CameraMode = 0;
	int32 ShownCam = -1;
	bool bPaused = false;
	/** Replay (owner, 2026-11-15), for looking at a play again. The pitches already shown are
	 *  kept (the newest HistoryMax), each with the board as it stood before it. R goes back:
	 *  to the start of the pitch on screen, or, within ReplayBackS of its start, to the pitch
	 *  before. F goes forward a pitch; from the newest one it leaves the replay. A replayed
	 *  pitch runs on into the next, up to the live one, and the game carries on from there.
	 *  The speed keys, pause and Enter work as always. */
	struct FShown
	{
		FBbPlay Play;
		BbPlayInfo Before{};
	};
	TArray<FShown> History;
	static constexpr int32 HistoryMax = 80;
	static constexpr float ReplayBackS = 1.0f;
	bool bReplay = false;
	/** Which pitch is on screen during a replay: an index into History, or History.Num() for
	 *  the live pitch (LivePlay), which is not in History until the next one begins. */
	int32 ReplayIdx = 0;
	FBbPlay LivePlay;
	BbPlayInfo LiveBefore{};
	uint64 SeedShown = 0;
	void ReplayBack();
	void ReplayForward();
	void ShowReplay(int32 Index);
	/** -BbReplayAt=<s>: seconds of real time after which R is pressed twice and, 6 s later, F
	 *  once (unattended check; logs BB_REPLAY). */
	float ReplayAt = -1.f;
	int32 ReplayAuto = 0;
	/** Saving a play (owner, 2026-11-15; BbSavePlay.h). During a replay a "Save this play"
	 *  button shows at the top right and the mouse is free to click it; it, or S at any
	 *  time, opens the dialog for the pitch on screen. The game pauses while it is open. */
	TSharedPtr<SWidget> SaveButtonLayer;
	TSharedPtr<SWidget> SaveDialogLayer;
	TSharedPtr<class SBbSavePlayDialog> SaveDialog;
	bool bSaveOpen = false;
	/** Where the dialog was left: its distance from the screen's right and top edges (Slate units). */
	FVector2D SaveDialogOffset = FVector2D(28.f, 76.f);
	bool bPausedBeforeSave = false;
	/** The input mode last set: -1 none yet, 0 game only, 1 game and UI (replay), 2 UI only (dialog). */
	int32 InputModeSet = -1;
	/** The dialog's buttons close it on the next tick, not from inside their own click:
	 *  0 nothing, 1 cancel, 2 save SaveComment. */
	int32 SaveClose = 0;
	FString SaveComment;
	/** "Saved: saved-plays/seed7-play12-....toml" under the REPLAY line, until SavedUntil (Wall). */
	FString SavedNote;
	double SavedUntil = 0.0;
	void OpenSaveDialog();
	void CloseSaveDialog(bool bSave);
	/** Shows the mouse and lets Slate have clicks during a replay; hides it otherwise. */
	void SyncCursor();
	/** -BbSaveAt=<s>: seconds of real time after which S is pressed and, 1 s later, Save with a
	 *  test comment (unattended check; logs BB_SAVE). */
	float SaveAt = -1.f;
	int32 SaveAuto = 0;
	/** Drawing a held ball in the hand: the hand's offset from the sim's ball (cm), kept from
	 *  the last frame it was held, and how much of it is applied (1 in the hand, 0 on the
	 *  sim's path). It eases in over HeldTakeS when a fielder takes the ball and out over
	 *  HeldLetGoS once it is thrown, in play time. */
	FVector HeldOffset = FVector::ZeroVector;
	float HeldBlend = 0.f;
	static constexpr float HeldTakeS = 0.15f;
	static constexpr float HeldLetGoS = 0.12f;
	/** The stand-in throw clip (MM_Attack_02) swings its left arm; the pitch clip its right. */
	static constexpr bool bThrowClipLeftArm = true;
	/** Set by Enter: play at RushSpeed (or Speed, if that is faster) until the next pitch is
	 *  ready to be thrown (BeginPlayback), then return to Speed. Enter again cancels it. */
	bool bRush = false;
	static constexpr float RushSpeed = 4.f;
	/** -BbRushAt=<s>: seconds of real time after which Enter is pressed once (unattended check). */
	float RushAt = -1.f;
	float RushClock = 0.f;

	// Unattended checks.
	int32 ShotsLeft = 0;
	int32 ShotIndex = 0;
	/** -BbDiveShots=<n>: photograph the next n dives at five moments each, then quit. */
	int32 DiveShotsLeft = 0;
	int32 DiveShotIndex = 0;
	/** -BbSlideShots=<n>: the same for the next n slides. */
	int32 SlideShotsLeft = 0;
	int32 SlideShotIndex = 0;
	/** The moments of the pitch on screen that those two photograph (play time, file name),
	 *  in time order, and the next to take; which slide and which diver the camera is on. */
	TArray<TPair<float, FString>> ExtraShots;
	int32 ExtraShotNext = 0;
	int32 ShotSlide = -1;
	int32 ShotDiver = -1;
	float ShotDiveFrom = 0.f;
	float ShotDiveYaw = 0.f;
	float ShotEvery = 4.f;
	float NextShot = 3.f;
	/** -BbParkShots: the spot now being photographed, or -1; when its picture is asked for
	 *  (the camera sits there a moment first, so the picture is settled), and when it is taken. */
	int32 ParkShot = -1;
	/** -BbFigureShots: the figures stood in rows and photographed, no game played: which
	 *  picture is being taken, or -1. */
	int32 FigureShot = -1;
	TArray<BbPlayer> ShotPlayers;
	void ArrangeFigureShot(int32 Step);
	/** -BbMannequin: the grey mannequin as it was before the figures, for comparison. */
	bool bMannequin = false;
	UPROPERTY()
	TObjectPtr<UBbFigureShop> FigureShop;
	float ParkShotsAt = 12.f;
	double ParkShotSince = 0.0;
	bool bParkShotAsked = false;
	float BenchSeconds = 0.f;
	double Wall = 0.0;
	int32 Frames = 0;
	double WorstFrame = 0.0;
	int32 PlaysShown = 0;
	bool bStagingLog = false;
	int32 ExitsArrived = 0;
	int32 ExitsCut = 0;
	int32 ExitsRejoined = 0;
	int32 ThrowsShown = 0;
	/** Figure-frames drawn mirrored (a left-hander pitching, throwing or batting). */
	int32 MirroredFrames = 0;
	int32 ThrowsMissed = 0;
};
