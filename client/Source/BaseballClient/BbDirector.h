#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Async/Future.h"
#include "BbSim.h"
#include "BbDirector.generated.h"

class AStaticMeshActor;
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
 *               -BbShots=<n> (take n screenshots, then quit) -BbShotEvery=<seconds>
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
