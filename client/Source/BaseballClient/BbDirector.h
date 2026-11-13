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
 *               -BbBenchSeconds=<s> (log frame rate and memory, then quit)
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
	void Finish();

	TUniquePtr<FBbSim> Sim;
	FBbPlay Current;
	TFuture<FBbPlay> Pending;
	bool bPending = false;
	bool bGameOver = false;
	float PlayTime = 0.f;
	float Speed = 1.f;
	float HoldAfter = 0.f;
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
	UPROPERTY()
	TArray<TObjectPtr<ASkeletalMeshActor>> Figures;
	struct FFigureState
	{
		uint16 Agent = 0xFFFF;
		FVector Pos = FVector::ZeroVector;
		float Yaw = 0.f;
		int32 Gait = -1; // 0 idle, 1 walk, 2 jog
		float SpeedCm = 0.f;
		int32 Team = -1;
		// A throw in progress: when it began (play time) and which way it goes.
		bool bThrowing = false;
		float ThrowStart = 0.f;
		float ThrowYaw = 0.f;
		// The clip being driven by hand (null while the gait clips play on their own).
		const UAnimSequence* Manual = nullptr;
	};
	TArray<FFigureState> FigureState;

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
	// Staging values (client presentation only). Players walk a short way and jog easily
	// farther; to keep the next pitch from waiting past ResetMaxSeconds they quicken, but
	// never past ResetHurryMps (the jog clip's own pace). Someone farther than that allows,
	// such as a pitcher who chased a fly ball into the outfield, takes as long as it takes.
	// Nobody starts or stops dead: speed builds and falls at ResetAccelMps2.
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
	/** Index into Movers of whoever returns the ball, or -1 when no ball is thrown back. */
	int32 ResetThrower = -1;
	FVector ResetBallFrom = FVector::ZeroVector; // sim frame, metres
	FVector ResetBallTo = FVector::ZeroVector;
	float ResetThrowAt = 0.f;
	float ResetFlight = 0.f;
	bool StartReset(const FBbPlay& Next);
	/** The staged frame at reset time T; false in bBallShown while the ball is in a glove. */
	void ResetFrame(float T, BbFrame& Out, bool& bBallShown) const;
	void PoseFigure(int32 Index, const BbFrame& Frame, const BbFrame& Later, float SimDt, float DeltaSeconds);
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> Ball;
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> BallShadow;
	/** Diameter of the ball's ground shadow with the ball on the ground, m (engine units are
	 *  set by the cylinder's 1 m size at scale 1). */
	static constexpr float BallShadowM = 0.50f;
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
	float ShotEvery = 4.f;
	float NextShot = 3.f;
	float BenchSeconds = 0.f;
	double Wall = 0.0;
	int32 Frames = 0;
	double WorstFrame = 0.0;
	int32 PlaysShown = 0;
};
