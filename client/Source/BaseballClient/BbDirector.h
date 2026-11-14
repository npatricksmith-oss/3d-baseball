#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Async/Future.h"
#include "BbSim.h"
#include "BbDirector.generated.h"

class AStaticMeshActor;
class ASkeletalMeshActor;
class USkeletalMesh;
class UAnimSequence;
class ACameraActor;
class UStaticMesh;
class UMaterialInterface;

/**
 * Builds the field from engine primitives, then plays the sim's timelines: capsule players,
 * the ball, a broadcast camera, and the scoreboard. The sim runs on a worker thread, one
 * pitch ahead; nothing here decides anything about the game (PLAN §3.4, D4).
 *
 * Command line: -BbSimLib=<path to libbb_ffi.so> -BbSeed=<n> -BbSpeed=<x>
 *               -BbCamera=<0 broadcast | 1 behind home | 2 overhead>
 *               -BbShots=<n> (take n screenshots, then quit) -BbShotEvery=<seconds> -BbShotStart=<seconds>
 *               -BbDiveShots=<n> (photograph n dives: in the air, lying, getting up; then quit)
 *               -BbAnimTest (stock clips looping) or -BbAnimTest=dive (dive pose candidates;
 *               -BbAnimFrac=<0..1> -BbAnimTilt=<degrees>)
 *               -BbBenchSeconds=<s> (log frame rate and memory, then quit)
 *               -BbStagingLog (log each reset, each player leaving the field and each throw's
 *               first animation phase as BB_RESET / BB_EXIT / BB_THROW, and a BB_STAGING summary)
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
	void BuildField();
	void BuildLights();
	void BuildStands();
	void RequestNext();
	void BeginPlayback(FBbPlay&& Play);
	void UpdateCamera(const BbFrame& Frame, float DeltaSeconds);
	/** Puts the ball's shadow under the ball, sized for the camera as it now stands. */
	void PlaceBallShadow(const BbFrame& Frame, bool bShown);
	void Finish();

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
	float PreRoll = 1.0f;

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
	};
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
		/** Down from a dive (the sim's BbDive): seconds since he left his feet, or negative
		 *  for none; seconds until he is up; which way he dove (Unreal yaw, if bDiveDir). */
		float DiveSince = -1.f;
		float DiveUpIn = 0.f;
		bool bDiveDir = false;
		float DiveYaw = 0.f;
	};
	void PoseFigure(int32 Slot, const BbActor& A, const BbFrame& Frame, const FPoseInput& In, float SimDt, float DeltaSeconds);
	/** A dive's layout (client presentation): he tips forward to DiveTiltDeg from the feet
	 *  over DiveFallS (the sim's lunge takes 0.3 s) and comes down DiveDropCm onto the turf
	 *  (the leap clip, held DiveClipFrac through, has him in the air and leaning 45° already),
	 *  lies there, and gets up over the last DiveRiseS before the sim has him up. Tipped
	 *  flat, his middle is DiveBodyBackCm from his feet. Check with -BbDiveShots. */
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
	};
	TArray<FThrowNote> Throws;
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
	static FVector2D Dugout(uint16 Agent);
	/** Index into Movers of whoever returns the ball, or -1 when no ball is thrown back. */
	int32 ResetThrower = -1;
	FVector ResetBallFrom = FVector::ZeroVector; // sim frame, metres
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
	/** Diameter the ball is drawn at, m: twice the real 7.4 cm (engine primitives are 1 m
	 *  across at scale 1). */
	static constexpr float BallDrawnM = 0.15f;
	/** Smallest diameter of the ball's ground shadow with the ball on the ground, m: the drawn
	 *  ball's own size, as the sun's shadow of a ball is. Close to the camera it stays this size. */
	static constexpr float BallShadowM = BallDrawnM;
	/** With the ball on the ground, a far shadow keeps at least this share of the view's width
	 *  across (about 11 px of a 1600 px view), so it can still be seen deep in the outfield.
	 *  The climb then shrinks it like the 15 cm floor. Sized so the shadow is never larger on
	 *  screen than the 50 cm disc the owner found too big (2026-11-12): from the follow camera
	 *  a ball in the near infield gets about 11 x 6.5 px against that disc's 18 x 6; at the
	 *  pitch camera's long lens the 15 cm floor governs. */
	static constexpr float BallShadowViewShare = 0.007f;
	/** Most the shadow is drawn longer along the line of sight than across it. A camera low
	 *  over a far ball sees the disc nearly edge-on (19 m up and 155 m away it shows an eighth
	 *  of its width); the length makes up the square root of that, so the shadow still reads
	 *  as lying flat on the ground but stays a few pixels tall (deep fly: 11 x 3.4 px). */
	static constexpr float BallShadowMaxStretch = 2.5f;
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> Bat;
	UPROPERTY()
	TObjectPtr<ACameraActor> Camera;

	FVector CamLook = FVector::ZeroVector;
	bool bBallInPlay = false;
	/** 0 broadcast (cuts), 1 behind home, 2 overhead. */
	int32 CameraMode = 0;
	int32 ShownCam = -1;
	bool bPaused = false;

	// Unattended checks.
	int32 ShotsLeft = 0;
	int32 ShotIndex = 0;
	/** -BbDiveShots=<n>: photograph the next n dives (in the air, lying, getting up), then quit. */
	int32 DiveShotsLeft = 0;
	int32 DiveShotIndex = 0;
	float ShotEvery = 4.f;
	float NextShot = 3.f;
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
