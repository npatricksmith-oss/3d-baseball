#include "BbDirector.h"
#include "BbFrame.h"
#include "BbGameMode.h"
#include "Async/Async.h"
#include "Camera/CameraActor.h"
#include "Camera/CameraComponent.h"
#include "Components/DirectionalLightComponent.h"
#include "Components/SkyAtmosphereComponent.h"
#include "Components/SkyLightComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/DirectionalLight.h"
#include "Engine/SkyLight.h"
#include "Engine/StaticMesh.h"
#include "Engine/StaticMeshActor.h"
#include "Engine/World.h"
#include "GameFramework/PlayerController.h"
#include "HAL/PlatformMemory.h"
#include "Kismet/GameplayStatics.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "Misc/CommandLine.h"
#include "Misc/Parse.h"
#include "Misc/Paths.h"
#include "UnrealClient.h"

DEFINE_LOG_CATEGORY_STATIC(LogBb, Log, All);

using BbFrameConv::ToUE;

namespace
{
	// The sim's reference park (bb_physics::ground::Field::default): 330 / 400 / 330 ft, 8 ft wall.
	const double FT = 0.3048;
	const double BASE = 27.432;
	double FenceAt(double Theta)
	{
		const double S = FMath::Clamp(Theta / (PI / 4.0), -1.0, 1.0);
		return 400.0 * FT + (330.0 - 400.0) * FT * S * S;
	}
	const FLinearColor Grass(0.05f, 0.22f, 0.05f);
	const FLinearColor Dirt(0.35f, 0.2f, 0.1f);
	const FLinearColor Chalk(0.9f, 0.9f, 0.9f);
	const FLinearColor WallColor(0.02f, 0.08f, 0.2f);
	const int32 MaxActors = BB_MAX_ACTORS;
}

ABbDirector::ABbDirector()
{
	PrimaryActorTick.bCanEverTick = true;
	RootComponent = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
}

AStaticMeshActor* ABbDirector::Shape(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FLinearColor& Color, const FRotator& Rotation)
{
	AStaticMeshActor* A = GetWorld()->SpawnActor<AStaticMeshActor>(Location, Rotation);
	UStaticMeshComponent* C = A->GetStaticMeshComponent();
	C->SetMobility(EComponentMobility::Movable);
	C->SetStaticMesh(Mesh);
	C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	A->SetActorScale3D(Scale);
	if (BaseMaterial)
	{
		UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(BaseMaterial, A);
		M->SetVectorParameterValue(TEXT("Color"), Color);
		C->SetMaterial(0, M);
	}
	return A;
}

void ABbDirector::BuildLights()
{
	UWorld* W = GetWorld();
	// The template map may bring its own sky and sun; only add what is missing.
	TArray<AActor*> Found;
	UGameplayStatics::GetAllActorsOfClass(W, ADirectionalLight::StaticClass(), Found);
	if (Found.Num() == 0)
	{
		ADirectionalLight* Sun = W->SpawnActor<ADirectionalLight>(FVector(0, 0, 5000), FRotator(-50.f, 35.f, 0.f));
		Sun->GetLightComponent()->SetMobility(EComponentMobility::Movable);
		Sun->GetLightComponent()->SetIntensity(6.f);
		if (UDirectionalLightComponent* D = Cast<UDirectionalLightComponent>(Sun->GetLightComponent()))
		{
			D->SetAtmosphereSunLight(true);
			D->DynamicShadowDistanceMovableLight = 30000.f;
		}
	}
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ASkyLight::StaticClass(), Found);
	if (Found.Num() == 0)
	{
		AActor* Atmosphere = W->SpawnActor<AActor>(AActor::StaticClass());
		USkyAtmosphereComponent* Sky = NewObject<USkyAtmosphereComponent>(Atmosphere);
		Atmosphere->SetRootComponent(Sky);
		Sky->RegisterComponent();
		ASkyLight* SkyLight = W->SpawnActor<ASkyLight>();
		SkyLight->GetLightComponent()->SetMobility(EComponentMobility::Movable);
		SkyLight->GetLightComponent()->bRealTimeCapture = true;
		SkyLight->GetLightComponent()->SetIntensity(1.2f);
		SkyLight->GetLightComponent()->RecaptureSky();
	}
}

void ABbDirector::BuildField()
{
	// Engine primitives are 100 cm across (the plane is 100 × 100 cm).
	// Layers from the bottom, each a few centimetres above the last so none of them fight.
	Shape(Plane, ToUE(0, 60, -0.20), FVector(400, 400, 1), Grass);
	// Infield dirt: a flat disc of the sim's dirt radius (95 ft).
	Shape(Cylinder, ToUE(0, 0, -0.15), FVector(2 * 95 * FT, 2 * 95 * FT, 0.1), Dirt);
	// Grass inside the base paths, then the mound and the home-plate circle.
	Shape(Cube, ToUE(0, BASE * 0.7071, -0.10), FVector(BASE * 0.82, BASE * 0.82, 0.1), Grass, FRotator(0, 45, 0));
	Shape(Cylinder, ToUE(0, 18.44, -0.05), FVector(5.5, 5.5, 0.15), Dirt);
	Shape(Cylinder, ToUE(0, 0, -0.06), FVector(8.0, 8.0, 0.1), Dirt);
	// Bases and home plate.
	const double Bx[4] = { 0, BASE * 0.7071, 0, -BASE * 0.7071 };
	const double By[4] = { 0, BASE * 0.7071, BASE * 1.4142, BASE * 0.7071 };
	for (int32 I = 0; I < 4; ++I)
	{
		Shape(Cube, ToUE(Bx[I], By[I], 0.04), FVector(0.4, 0.4, 0.08), Chalk, FRotator(0, 45, 0));
	}
	Shape(Cube, ToUE(0, 18.44, 0.27), FVector(0.15, 0.6, 0.03), Chalk);
	// Foul lines to the fence.
	const double LineLen = 330.0 * FT;
	for (int32 Side = -1; Side <= 1; Side += 2)
	{
		const double Mx = Side * LineLen * 0.5 * 0.7071;
		const double My = LineLen * 0.5 * 0.7071;
		Shape(Cube, ToUE(Mx, My, 0.01), FVector(LineLen, 0.1, 0.01), Chalk, FRotator(0, Side * 45.f, 0));
		// Foul poles.
		Shape(Cylinder, ToUE(Side * LineLen * 0.7071, LineLen * 0.7071, 6.0), FVector(0.3, 0.3, 12.0), FLinearColor(0.9f, 0.8f, 0.1f));
	}
	// Outfield wall: short straight panels along the fence curve.
	const int32 Panels = 48;
	for (int32 I = 0; I < Panels; ++I)
	{
		const double T0 = -PI / 4 + (PI / 2) * I / Panels;
		const double T1 = -PI / 4 + (PI / 2) * (I + 1) / Panels;
		const FVector2D A(FenceAt(T0) * FMath::Sin(T0), FenceAt(T0) * FMath::Cos(T0));
		const FVector2D B(FenceAt(T1) * FMath::Sin(T1), FenceAt(T1) * FMath::Cos(T1));
		const FVector2D Mid = (A + B) * 0.5;
		const double Len = (B - A).Size();
		// Panel long axis: the cube's X, turned to run from A to B (in Unreal's frame).
		const FVector Dir = ToUE(B.X - A.X, B.Y - A.Y, 0).GetSafeNormal();
		Shape(Cube, ToUE(Mid.X, Mid.Y, 4.0 * FT), FVector(Len + 0.05, 0.3, 8.0 * FT), WallColor, Dir.Rotation());
	}
}

void ABbDirector::BeginPlay()
{
	Super::BeginPlay();
	Cube = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Cube.Cube"));
	Sphere = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Sphere.Sphere"));
	Cylinder = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Cylinder.Cylinder"));
	Plane = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Plane.Plane"));
	BaseMaterial = LoadObject<UMaterialInterface>(nullptr, TEXT("/Engine/BasicShapes/BasicShapeMaterial.BasicShapeMaterial"));

	// Whatever the template map contains (a floor, a player start) is not part of the park.
	TArray<AActor*> Existing;
	UGameplayStatics::GetAllActorsOfClass(GetWorld(), AStaticMeshActor::StaticClass(), Existing);
	for (AActor* A : Existing)
	{
		A->Destroy();
	}

	BuildLights();
	BuildField();

	// Players: a body and a head each, hidden until a frame uses them.
	for (int32 I = 0; I < MaxActors; ++I)
	{
		AStaticMeshActor* Body = Shape(Cylinder, FVector::ZeroVector, FVector(0.5, 0.5, 1.5), FLinearColor::White);
		AStaticMeshActor* Head = Shape(Sphere, FVector::ZeroVector, FVector(0.28), FLinearColor(0.8f, 0.6f, 0.45f));
		Body->SetActorHiddenInGame(true);
		Head->SetActorHiddenInGame(true);
		Bodies.Add(Body);
		Heads.Add(Head);
	}
	// The ball is drawn three times its real size so it can be seen from the stands.
	Ball = Shape(Sphere, FVector::ZeroVector, FVector(0.22), FLinearColor(1.f, 1.f, 0.9f));

	Camera = GetWorld()->SpawnActor<ACameraActor>(ToUE(0, -22, 14), FRotator::ZeroRotator);
	Camera->GetCameraComponent()->SetFieldOfView(60.f);
	Camera->GetCameraComponent()->bConstrainAspectRatio = false;
	// Fixed exposure, a little under the default: daylight on flat colors.
	Camera->GetCameraComponent()->PostProcessSettings.bOverride_AutoExposureBias = true;
	Camera->GetCameraComponent()->PostProcessSettings.AutoExposureBias = -0.7f;
	CamLook = ToUE(0, 22, 0);
	if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
	{
		PC->SetViewTarget(Camera);
	}

	const TCHAR* Cmd = FCommandLine::Get();
	FString Lib = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT("../sim/target/release/libbb_ffi.so")));
	FParse::Value(Cmd, TEXT("BbSimLib="), Lib);
	uint64 Seed = 7;
	FParse::Value(Cmd, TEXT("BbSeed="), Seed);
	FParse::Value(Cmd, TEXT("BbSpeed="), Speed);
	FParse::Value(Cmd, TEXT("BbShots="), ShotsLeft);
	FParse::Value(Cmd, TEXT("BbShotEvery="), ShotEvery);
	FParse::Value(Cmd, TEXT("BbBenchSeconds="), BenchSeconds);

	Sim = MakeUnique<FBbSim>();
	if (!Sim->Start(Lib, Seed))
	{
		Sim.Reset();
		bGameOver = true;
		return;
	}
	UE_LOG(LogBb, Display, TEXT("Sim loaded from %s, seed %llu"), *Lib, Seed);
	BeginPlayback(Sim->Next());
	RequestNext();
}

void ABbDirector::RequestNext()
{
	if (!Sim || bPending)
	{
		return;
	}
	FBbSim* S = Sim.Get();
	// The sim never runs on the game thread while a play is on screen (PLAN §3.4).
	Pending = Async(EAsyncExecution::ThreadPool, [S]() { return S->Next(); });
	bPending = true;
}

void ABbDirector::BeginPlayback(FBbPlay&& Play)
{
	if (Current.Blob)
	{
		Sim->Release(Current);
	}
	Current = MoveTemp(Play);
	PlayTime = 0.f;
	bBallInPlay = false;
	if (!Current.bValid)
	{
		bGameOver = true;
		UE_LOG(LogBb, Display, TEXT("Game over after %d pitches"), PlaysShown);
		return;
	}
	++PlaysShown;
	// A ball in play gets a moment at the end; a taken pitch moves on quickly.
	HoldAfter = Current.Info.exit_velocity_mps > 0.f ? 1.5f : 0.6f;
}

void ABbDirector::UpdateCamera(const BbFrame& Frame, float DeltaSeconds)
{
	const FVector BallUE = ToUE(Frame.ball);
	// Once the ball is hit past the mound side of the plate, follow it; otherwise frame the
	// pitcher-to-plate line from behind home.
	const bool bFollow = bBallInPlay;
	const FVector Target = bFollow ? FVector(BallUE.X, BallUE.Y, FMath::Min(BallUE.Z, 1500.0) * 0.5) : ToUE(0, 20, 1);
	CamLook = FMath::VInterpTo(CamLook, Target, DeltaSeconds, bFollow ? 4.f : 2.f);
	const FVector Eye = Camera->GetActorLocation();
	Camera->SetActorRotation((CamLook - Eye).Rotation());
	const float WantFov = bFollow ? 42.f : 58.f;
	UCameraComponent* C = Camera->GetCameraComponent();
	C->SetFieldOfView(FMath::FInterpTo(C->FieldOfView, WantFov, DeltaSeconds, 2.f));
}

void ABbDirector::Tick(float DeltaSeconds)
{
	Super::Tick(DeltaSeconds);
	Wall += DeltaSeconds;
	++Frames;
	if (Wall > 5.0)
	{
		WorstFrame = FMath::Max(WorstFrame, (double)DeltaSeconds);
	}

	if (Current.bValid && Sim)
	{
		PlayTime += DeltaSeconds * Speed;
		const float T = FMath::Min(PlayTime, Current.Info.duration_s);
		BbFrame Frame;
		if (Sim->Sample(Current, T, Frame))
		{
			Ball->SetActorLocation(ToUE(Frame.ball));
			for (int32 I = 0; I < MaxActors; ++I)
			{
				const bool bShow = I < (int32)Frame.n_actors;
				Bodies[I]->SetActorHiddenInGame(!bShow);
				Heads[I]->SetActorHiddenInGame(!bShow);
				if (!bShow)
				{
					continue;
				}
				const BbActor& A = Frame.actors[I];
				const FVector Foot = ToUE(A.pos);
				Bodies[I]->SetActorLocation(Foot + FVector(0, 0, 75));
				Bodies[I]->SetActorRotation(BbFrameConv::YawToUE(A.yaw));
				Heads[I]->SetActorLocation(Foot + FVector(0, 0, 165));
				// Team colors: home white, away grey; fielders a shade darker than batters.
				const bool bHome = (A.agent >> 8) != 0;
				const bool bFielder = (A.agent & 0x80) != 0;
				const FLinearColor Col = bHome ? (bFielder ? FLinearColor(0.85f, 0.85f, 0.9f) : FLinearColor(1.f, 1.f, 1.f)) : (bFielder ? FLinearColor(0.15f, 0.2f, 0.5f) : FLinearColor(0.25f, 0.3f, 0.7f));
				if (UMaterialInstanceDynamic* M = Cast<UMaterialInstanceDynamic>(Bodies[I]->GetStaticMeshComponent()->GetMaterial(0)))
				{
					M->SetVectorParameterValue(TEXT("Color"), Col);
				}
			}
			// In play: the ball has been hit and is heading out (contact telemetry present
			// and the ball is in front of the plate after the pitch has arrived).
			if (Current.Info.exit_velocity_mps > 0.f && T > 0.35f && Frame.ball[1] > 1.5f)
			{
				bBallInPlay = true;
			}
			UpdateCamera(Frame, DeltaSeconds);
		}
		if (ABbHUD* Hud = GetWorld()->GetFirstPlayerController() ? Cast<ABbHUD>(GetWorld()->GetFirstPlayerController()->GetHUD()) : nullptr)
		{
			const BbPlayInfo& In = Current.Info;
			if (PlayTime >= In.duration_s)
			{
				Hud->Scoreboard = FString::Printf(TEXT("AWAY %d   HOME %d    %s %d    %d out    %d-%d"), In.away_runs, In.home_runs, In.top ? TEXT("Top") : TEXT("Bot"), In.inning, In.outs, In.balls, In.strikes);
			}
			const float Mph = In.release_speed_mps * 2.23694f;
			Hud->PitchLine = In.exit_velocity_mps > 0.f
				? FString::Printf(TEXT("Pitch %.0f mph   Exit %.0f mph at %.0f deg"), Mph, In.exit_velocity_mps * 2.23694f, FMath::RadiansToDegrees(In.launch_angle_rad))
				: FString::Printf(TEXT("Pitch %.0f mph"), Mph);
		}
		if (PlayTime >= Current.Info.duration_s + HoldAfter && bPending && Pending.IsReady())
		{
			FBbPlay Next = Pending.Get();
			bPending = false;
			BeginPlayback(MoveTemp(Next));
			if (!bGameOver)
			{
				RequestNext();
			}
		}
	}

	// Unattended checks: screenshots at intervals, or a timed frame-rate measurement.
	if (ShotsLeft > 0 && Wall >= NextShot)
	{
		const FString Name = FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), FString::Printf(TEXT("shot_%02d.png"), ShotIndex++));
		FScreenshotRequest::RequestScreenshot(Name, true, false);
		NextShot += ShotEvery;
		if (--ShotsLeft == 0)
		{
			BenchSeconds = FMath::Max(BenchSeconds, (float)Wall + 2.f);
		}
	}
	if ((BenchSeconds > 0.f && Wall >= BenchSeconds) || (bGameOver && (ShotsLeft > 0 || BenchSeconds > 0.f)))
	{
		Finish();
	}
}

void ABbDirector::Finish()
{
	const FPlatformMemoryStats Mem = FPlatformMemory::GetStats();
	UE_LOG(LogBb, Display, TEXT("BB_RESULT plays=%d frames=%d seconds=%.1f avg_fps=%.1f worst_frame_ms=%.1f used_physical_mb=%llu peak_physical_mb=%llu"),
		PlaysShown, Frames, Wall, Frames / FMath::Max(Wall, 0.001), WorstFrame * 1000.0, Mem.UsedPhysical >> 20, Mem.PeakUsedPhysical >> 20);
	BenchSeconds = 0.f;
	ShotsLeft = 0;
	FPlatformMisc::RequestExit(false);
}

void ABbDirector::EndPlay(const EEndPlayReason::Type Reason)
{
	// Let the worker finish before the library goes away.
	if (bPending)
	{
		FBbPlay P = Pending.Get();
		if (Sim)
		{
			Sim->Release(P);
		}
		bPending = false;
	}
	if (Sim && Current.Blob)
	{
		Sim->Release(Current);
	}
	Sim.Reset();
	Super::EndPlay(Reason);
}
