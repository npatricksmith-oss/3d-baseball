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
	void PoseFigure(int32 Index, const BbFrame& Frame, const BbFrame& Soon, const BbFrame& Later, float SimDt, float DeltaSeconds);
	UPROPERTY()
	TObjectPtr<AStaticMeshActor> Ball;
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
