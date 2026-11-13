#include "BbDirector.h"
#include "BbFrame.h"
#include "BbGameMode.h"
#include "Animation/AnimSequence.h"
#include "Animation/SkeletalMeshActor.h"
#include "Async/Async.h"
#include "Camera/CameraActor.h"
#include "Camera/CameraComponent.h"
#include "Components/DirectionalLightComponent.h"
#include "Components/SkyAtmosphereComponent.h"
#include "Components/SkyLightComponent.h"
#include "Components/SkeletalMeshComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/SkeletalMesh.h"
#include "Engine/DirectionalLight.h"
#include "Engine/ExponentialHeightFog.h"
#include "Engine/SkyLight.h"
#include "Engine/StaticMesh.h"
#include "Engine/StaticMeshActor.h"
#include "Engine/World.h"
#include "GameFramework/PlayerController.h"
#include "InputCoreTypes.h"
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
	/** Where an Unreal-frame figure at Foot looks to face the plate. */
	float YawToPlate(const FVector& Foot)
	{
		return (ToUE(0, 0.3, 0) - Foot).Rotation().Yaw;
	}
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
	// No distance haze: the pitch camera looks down 140 m of field through a long lens.
	UGameplayStatics::GetAllActorsOfClass(W, AExponentialHeightFog::StaticClass(), Found);
	for (AActor* A : Found)
	{
		A->Destroy();
	}
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ADirectionalLight::StaticClass(), Found);
	// An early-afternoon sun over the third-base side lights both camera angles.
	for (AActor* A : Found)
	{
		A->SetActorRotation(FRotator(-55.f, 100.f, 0.f));
	}
	if (Found.Num() == 0)
	{
		ADirectionalLight* Sun = W->SpawnActor<ADirectionalLight>(FVector(0, 0, 5000), FRotator(-55.f, 100.f, 0.f));
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
	// Ground layers from the bottom, a centimetre or two apart so none of them fight; they
	// cast no shadows, so their edges do not show as dark rims.
	auto Flat = [this](UStaticMesh* Mesh, const FVector& At, const FVector& Scale, const FLinearColor& Color, const FRotator& Rot = FRotator::ZeroRotator)
	{
		Shape(Mesh, At, Scale, Color, Rot)->GetStaticMeshComponent()->SetCastShadow(false);
	};
	Flat(Plane, ToUE(0, 60, -0.06), FVector(400, 400, 1), Grass);
	// Infield dirt: a flat disc of the sim's dirt radius (95 ft).
	Flat(Cylinder, ToUE(0, 0, -0.045), FVector(2 * 95 * FT, 2 * 95 * FT, 0.02), Dirt);
	// Grass inside the base paths, then the home-plate circle and the mound.
	Flat(Cube, ToUE(0, BASE * 0.7071, -0.03), FVector(BASE * 0.82, BASE * 0.82, 0.02), Grass, FRotator(0, 45, 0));
	Flat(Cylinder, ToUE(0, 0, -0.015), FVector(8.0, 8.0, 0.02), Dirt);
	Flat(Cylinder, ToUE(0, 18.44, -0.005), FVector(5.5, 5.5, 0.02), Dirt);
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

void ABbDirector::BuildStands()
{
	// Seating as plain blocks, so the park reads as a park: a bank beyond the outfield wall,
	// a grandstand down each foul line, and a curve behind home. None of it is in the sim.
	const FLinearColor Seats(0.2f, 0.22f, 0.28f);
	const FLinearColor Concrete(0.45f, 0.45f, 0.47f);
	auto Block = [this](const FVector2D& A, const FVector2D& B, double Depth, double Height, const FLinearColor& Color)
	{
		const FVector2D Mid = (A + B) * 0.5;
		const FVector Dir = ToUE(B.X - A.X, B.Y - A.Y, 0).GetSafeNormal();
		Shape(Cube, ToUE(Mid.X, Mid.Y, Height * 0.5), FVector((B - A).Size() + 0.2, Depth, Height), Color, Dir.Rotation())->GetStaticMeshComponent()->SetCastShadow(false);
	};
	const int32 N = 24;
	for (int32 I = 0; I < N; ++I)
	{
		const double T0 = -PI / 4 + (PI / 2) * I / N;
		const double T1 = -PI / 4 + (PI / 2) * (I + 1) / N;
		// Two tiers stepping up and back from the wall.
		for (int32 Tier = 0; Tier < 2; ++Tier)
		{
			const double Out = 5.0 + 7.0 * Tier;
			const FVector2D A((FenceAt(T0) + Out) * FMath::Sin(T0), (FenceAt(T0) + Out) * FMath::Cos(T0));
			const FVector2D B((FenceAt(T1) + Out) * FMath::Sin(T1), (FenceAt(T1) + Out) * FMath::Cos(T1));
			Block(A, B, 7.0, 5.0 + 4.5 * Tier, Tier == 0 ? Seats : Concrete);
		}
	}
	// Foul-line grandstands: parallel to each line, 20 m outside it, from behind the plate
	// area to the pole.
	for (int32 Side = -1; Side <= 1; Side += 2)
	{
		const FVector2D Along(Side * 0.7071, 0.7071);
		const FVector2D Outward(Side * 0.7071, -0.7071);
		for (int32 Tier = 0; Tier < 2; ++Tier)
		{
			const FVector2D Off = Outward * (20.0 + 8.0 * Tier);
			Block(Along * 12.0 + Off, Along * (330.0 * FT) + Off, 8.0, 5.0 + 5.0 * Tier, Tier == 0 ? Seats : Concrete);
		}
	}
	// Behind home: a curve of seats joining the two grandstands, and the backstop wall.
	const int32 M = 10;
	for (int32 I = 0; I < M; ++I)
	{
		const double T0 = PI * 0.75 + (PI * 0.5) * I / M;
		const double T1 = PI * 0.75 + (PI * 0.5) * (I + 1) / M;
		Block(FVector2D(24.0 * FMath::Sin(T0), 24.0 * FMath::Cos(T0)), FVector2D(24.0 * FMath::Sin(T1), 24.0 * FMath::Cos(T1)), 6.0, 5.0, Seats);
		Block(FVector2D(18.0 * FMath::Sin(T0), 18.0 * FMath::Cos(T0)), FVector2D(18.0 * FMath::Sin(T1), 18.0 * FMath::Cos(T1)), 0.3, 1.2, WallColor);
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
	BuildStands();

	// Players. Human figures if the mannequin assets are in Content (client/run.sh copies
	// them from the engine's templates; they are not in the repository).
	FigureMesh = LoadObject<USkeletalMesh>(nullptr, TEXT("/Game/Characters/Mannequins/Meshes/SKM_Manny_Simple.SKM_Manny_Simple"));
	AnimIdle = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/MM_Idle.MM_Idle"));
	AnimWalk = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Fwd.MF_Unarmed_Walk_Fwd"));
	AnimJog = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jog/MF_Unarmed_Jog_Fwd.MF_Unarmed_Jog_Fwd"));
	AnimPitch = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_ChargedAttack.MM_ChargedAttack"));
	AnimThrow = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_02.MM_Attack_02"));
	AnimCrouch = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Land.MM_Land"));
	AnimStance = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_01.MM_Attack_01"));
	AnimWalkBack = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Bwd.MF_Unarmed_Walk_Bwd"));
	AnimWalkLeft = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Left.MF_Unarmed_Walk_Left"));
	AnimWalkRight = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Right.MF_Unarmed_Walk_Right"));
	const bool bFigures = FigureMesh && AnimIdle && AnimWalk && AnimJog;
	if (!bFigures)
	{
		UE_LOG(LogBb, Warning, TEXT("Mannequin assets not found under /Game/Characters/Mannequins: drawing players as cylinders. Run client/run.sh once to copy them."));
	}
	// Figures for the frame's actors, then for players walking off after their last pitch.
	const int32 Pool = MaxActors + MaxExiting;
	FigureState.SetNum(Pool);
	Exits.SetNum(MaxExiting);
	for (int32 I = 0; bFigures && I < Pool; ++I)
	{
		ASkeletalMeshActor* F = GetWorld()->SpawnActor<ASkeletalMeshActor>(FVector::ZeroVector, FRotator::ZeroRotator);
		USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
		C->SetMobility(EComponentMobility::Movable);
		C->SetSkeletalMeshAsset(FigureMesh);
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		C->SetAnimationMode(EAnimationMode::AnimationSingleNode);
		// Animate even when a figure is small or briefly off screen: the camera cuts often.
		C->VisibilityBasedAnimTickOption = EVisibilityBasedAnimTickOption::AlwaysTickPoseAndRefreshBones;
		F->SetActorHiddenInGame(true);
		Figures.Add(F);
	}
	// Cylinder stand-ins: a body and a head each, hidden until a frame uses them.
	for (int32 I = 0; I < Pool; ++I)
	{
		AStaticMeshActor* Body = Shape(Cylinder, FVector::ZeroVector, FVector(0.5, 0.5, 1.5), FLinearColor::White);
		AStaticMeshActor* Head = Shape(Sphere, FVector::ZeroVector, FVector(0.28), FLinearColor(0.8f, 0.6f, 0.45f));
		Body->SetActorHiddenInGame(true);
		Head->SetActorHiddenInGame(true);
		Bodies.Add(Body);
		Heads.Add(Head);
	}
	// The ball is drawn twice its real size (7.4 cm) so it can be seen from the stands.
	Ball = Shape(Sphere, FVector::ZeroVector, FVector(BallDrawnM), FLinearColor(1.f, 1.f, 0.9f));
	// The ball's shadow as a viewer reads it: a dark disc on the ground straight under the
	// ball (the sun's own shadow of so small a ball is lost at a distance and falls well
	// to the side of a high fly). It shrinks as the ball climbs, so height reads too.
	BallShadow = Shape(Cylinder, FVector::ZeroVector, FVector(BallShadowM, BallShadowM, 0.002), FLinearColor(0.02f, 0.03f, 0.02f));
	BallShadow->GetStaticMeshComponent()->SetCastShadow(false);
	// The bat: a cylinder stretched between the knob and the tip the sim reports, drawn a
	// little thicker than real so the swing reads from the stands.
	Bat = Shape(Cylinder, FVector::ZeroVector, FVector(0.09, 0.09, 0.86), FLinearColor(0.75f, 0.55f, 0.3f));
	// Nothing on the field until the first pitch is ready.
	Ball->SetActorHiddenInGame(true);
	BallShadow->SetActorHiddenInGame(true);
	Bat->SetActorHiddenInGame(true);

	Camera = GetWorld()->SpawnActor<ACameraActor>(ToUE(-9, 136, 16), FRotator::ZeroRotator);
	ShownCam = 0;
	Camera->GetCameraComponent()->SetFieldOfView(8.5f);
	Camera->GetCameraComponent()->bConstrainAspectRatio = false;
	// Fixed exposure, a little under the default: daylight on flat colors.
	Camera->GetCameraComponent()->PostProcessSettings.bOverride_AutoExposureBias = true;
	Camera->GetCameraComponent()->PostProcessSettings.AutoExposureBias = -0.7f;
	CamLook = ToUE(0, 6, 1.0);
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
	FParse::Value(Cmd, TEXT("BbCamera="), CameraMode);
	FParse::Value(Cmd, TEXT("BbShots="), ShotsLeft);
	FParse::Value(Cmd, TEXT("BbShotEvery="), ShotEvery);
	FParse::Value(Cmd, TEXT("BbShotStart="), NextShot);
	FParse::Value(Cmd, TEXT("BbBenchSeconds="), BenchSeconds);
	bStagingLog = FParse::Param(Cmd, TEXT("BbStagingLog"));

	// Clip preview (-BbAnimTest): a row of figures each looping one stock clip slowly, for
	// choosing which clip stands in for which baseball motion. No game is played.
	if (FParse::Param(Cmd, TEXT("BbAnimTest")) && Figures.Num() >= 8)
	{
		const TCHAR* Clips[8] = {
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_01.MM_Attack_01"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_02.MM_Attack_02"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_03.MM_Attack_03"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_ChargedAttack.MM_ChargedAttack"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Jump.MM_Jump"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Dash.MM_Dash"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Land.MM_Land"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Fall_Loop.MM_Fall_Loop"),
		};
		for (int32 I = 0; I < 8; ++I)
		{
			UAnimSequence* Clip = LoadObject<UAnimSequence>(nullptr, Clips[I]);
			ASkeletalMeshActor* F = Figures[I];
			F->SetActorHiddenInGame(false);
			F->SetActorLocation(ToUE(-7.0 + 2.0 * I, 12.0, 0.0));
			// Three-quarter view: turned a little off facing the camera.
			F->SetActorRotation(FRotator(0.f, 180.f - 90.f + 35.f, 0.f));
			if (Clip)
			{
				UE_LOG(LogBb, Display, TEXT("clip %d %s length %.2f s"), I, Clips[I], Clip->GetPlayLength());
				F->GetSkeletalMeshComponent()->PlayAnimation(Clip, true);
				F->GetSkeletalMeshComponent()->SetPlayRate(0.25f);
			}
		}
		Camera->SetActorLocation(ToUE(0, -1, 2.0));
		Camera->SetActorRotation((ToUE(0, 12, 1.0) - ToUE(0, -1, 2.0)).Rotation());
		Camera->GetCameraComponent()->SetFieldOfView(62.f);
		return;
	}

	Sim = MakeUnique<FBbSim>();
	if (!Sim->Start(Lib, Seed))
	{
		Sim.Reset();
		bGameOver = true;
		return;
	}
	UE_LOG(LogBb, Display, TEXT("Sim loaded from %s, seed %llu"), *Lib, Seed);
	// The first pitch, like every other, resolves on a worker; Tick starts it when it is ready.
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
	// The board as this play starts: as the last one left it (top of the first, before the
	// first pitch). The board stays on the outgoing half inning until here.
	if (Current.bValid)
	{
		Before = Current.Info;
	}
	else
	{
		FMemory::Memzero(Before);
		Before.inning = 1;
		Before.top = 1;
	}
	// The last play's throws: was each one's clip drawn?
	for (const FThrowNote& N : Throws)
	{
		if (Figures.Num() == 0)
		{
			break;
		}
		N.FirstClip >= 0.f ? ++ThrowsShown : ++ThrowsMissed;
		if (bStagingLog)
		{
			UE_LOG(LogBb, Display, TEXT("BB_THROW play=%u agent=0x%03x release=%.3f staged=0 first_clip=%.3f%s"), Current.Info.play_index, N.Agent, N.ReleaseAt, N.FirstClip, N.FirstClip >= 0.f ? TEXT("") : TEXT(" MISSED"));
		}
	}
	Throws.Reset();
	ThrowsSampledTo = -1.f;
	if (Current.Blob)
	{
		Sim->Release(Current);
	}
	Current = MoveTemp(Play);
	// Each pitch opens with the wind-up: play time runs from −PreRoll to the release at 0.
	PlayTime = -PreRoll;
	bBallInPlay = false;
	CallsLogged = 0;
	if (!Current.bValid)
	{
		bGameOver = true;
		UE_LOG(LogBb, Display, TEXT("Game over after %d pitches"), PlaysShown);
		return;
	}
	++PlaysShown;
	// Anyone still walking off who is in this pitch after all (only when no reset was staged).
	BbFrame Start;
	if (Sim->Sample(Current, 0.f, Start))
	{
		PurgeExits(Start);
	}
	// A ball in play gets a moment at the end; a taken pitch moves on quickly.
	HoldAfter = Current.Info.exit_velocity_mps > 0.f ? 2.5f : 0.7f;
}

void ABbDirector::NoteThrows(float From, float To)
{
	// Every frame from just after the last one sampled up to To, at least every 0.1 s: a
	// fielder holding the ball with a throw to make says when he lets go (release_in_s).
	if (To <= From)
	{
		return;
	}
	for (float S = FMath::Max(From + 0.1f, 0.f);; S += 0.1f)
	{
		const float At = FMath::Min(S, To);
		BbFrame F;
		if (Sim->Sample(Current, At, F) && F.holder >= 0 && F.release_in_s >= 0.f)
		{
			const uint16 Agent = (uint16)F.holder;
			const float ReleaseAt = At + F.release_in_s;
			if (!Throws.ContainsByPredicate([&](const FThrowNote& N) { return N.Agent == Agent && FMath::Abs(N.ReleaseAt - ReleaseAt) < 0.05f; }))
			{
				// He turns toward where the ball goes: from his feet at the release to the
				// ball a moment into its flight.
				FThrowNote N;
				N.Agent = Agent;
				N.ReleaseAt = ReleaseAt;
				BbFrame AtRelease, InFlight;
				FVector Foot = FVector::ZeroVector;
				if (Sim->Sample(Current, ReleaseAt, AtRelease))
				{
					for (uint32 I = 0; I < AtRelease.n_actors; ++I)
					{
						if (AtRelease.actors[I].agent == Agent)
						{
							Foot = ToUE(AtRelease.actors[I].pos);
						}
					}
				}
				N.Yaw = Sim->Sample(Current, ReleaseAt + 0.35f, InFlight) ? (ToUE(InFlight.ball) - Foot).Rotation().Yaw : YawToPlate(Foot);
				Throws.Add(N);
			}
		}
		if (At >= To)
		{
			break;
		}
	}
}

FVector2D ABbDirector::Dugout(uint16 Agent)
{
	// Stand-in dugouts in the front of the foul-line grandstands (BuildStands), 18 m along
	// the line and just inside the stand's face: home on the first-base side, away on the
	// third-base side. A player walking off disappears into the stand, not in the open.
	const double Side = (Agent >> 8) != 0 ? 1.0 : -1.0;
	const double Along = 17.7, Out = 18.5;
	return FVector2D(Side * 0.7071 * (Along + Out), 0.7071 * (Along - Out));
}

void ABbDirector::CarryFigure(const TArray<FFigureState>& Old, int32 Slot, uint16 Agent)
{
	for (int32 J = 0; J < Old.Num(); ++J)
	{
		if (Old[J].Agent != Agent)
		{
			continue;
		}
		if (J != Slot)
		{
			// Another figure takes him over where the last one left him. Its tint (Team)
			// belongs to the figure, so it stays; its clip restarts.
			FFigureState& St = FigureState[Slot];
			St.Agent = Agent;
			St.Pos = Old[J].Pos;
			St.Yaw = Old[J].Yaw;
			St.SpeedCm = Old[J].SpeedCm;
			St.Gait = -1;
			St.Manual = nullptr;
		}
		return;
	}
}

bool ABbDirector::StartReset(const FBbPlay& Next)
{
	BbFrame From, To;
	if (!Sim->Sample(Current, Current.Info.duration_s, From) || !Sim->Sample(Next, 0.f, To) || To.n_actors == 0)
	{
		return false;
	}
	auto Find = [](const BbFrame& F, uint16 Agent) -> const BbActor*
	{
		for (uint32 I = 0; I < F.n_actors; ++I)
		{
			if (F.actors[I].agent == Agent)
			{
				return &F.actors[I];
			}
		}
		return nullptr;
	};
	// Fielders carry 0x80 plus their position in the low byte: 1 the pitcher, 2 the catcher.
	auto AtPosition = [](const BbFrame& F, int32 Position) -> const BbActor*
	{
		for (uint32 I = 0; I < F.n_actors; ++I)
		{
			if ((F.actors[I].agent & 0xFF) == (0x80 | Position))
			{
				return &F.actors[I];
			}
		}
		return nullptr;
	};
	const BbActor* WasPitcher = AtPosition(From, 1);
	const BbActor* Pitcher = AtPosition(To, 1);
	const bool bSideChange = WasPitcher && Pitcher && (WasPitcher->agent >> 8) != (Pitcher->agent >> 8);
	auto OnDeck = [](uint16 Agent) { return FVector2D((Agent >> 8) != 0 ? 10.0 : -10.0, -3.0); };
	// Figures follow their player into his slot for the reset (the next pitch's order).
	const TArray<FFigureState> Old = FigureState;

	// Everyone in the next pitch first, in its order, so the hand-over to the play is seamless.
	Movers.Reset();
	for (uint32 I = 0; I < To.n_actors; ++I)
	{
		const BbActor& A = To.actors[I];
		FResetMover M;
		M.Agent = A.agent;
		M.To = FVector2D(A.pos[0], A.pos[1]);
		const int32 Walking = Exits.IndexOfByPredicate([&A](const FExitSlot& X) { return X.bActive && X.Move.Agent == A.agent; });
		bool bTurns = false;
		if (const BbActor* Was = Find(From, A.agent))
		{
			M.From = FVector2D(Was->pos[0], Was->pos[1]);
			bTurns = true;
		}
		else if (Walking != INDEX_NONE)
		{
			// Still on his way off, and in the next pitch after all: he comes back from there.
			M.From = ExitAt(Exits[Walking].Move);
			EndExit(Walking, TEXT("rejoined"));
			bTurns = true;
		}
		else
		{
			// New on the field: a fielder from his dugout, a batter from the on-deck circle.
			// A fielder with far to go is picked up already on his way out.
			M.From = (A.agent & 0x80) != 0 ? Dugout(A.agent) : OnDeck(A.agent);
			const double Dist = (M.To - M.From).Size();
			if (Dist > ResetEnterMaxM)
			{
				M.From = M.To + (M.From - M.To) * (ResetEnterMaxM / Dist);
			}
		}
		const float Dist = (M.To - M.From).Size();
		M.SpeedMps = Dist < ResetWalkWithinM ? ResetWalkMps : ResetJogMps;
		// He quickens rather than keep the next pitch waiting, but only up to a jog: a
		// sprint (or faster than legs go) back to his spot looks wrong.
		if (TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2) > ResetMaxSeconds)
		{
			M.SpeedMps = FMath::Max(M.SpeedMps, SpeedFor(Dist, ResetMaxSeconds, ResetAccelMps2, ResetHurryMps));
		}
		// Someone already on the field with a real way to go turns toward it before he sets
		// off; a short move is a shuffle and needs no turn.
		M.Depart = bTurns && Dist >= ShuffleMaxM ? ResetTurnS : 0.f;
		Movers.Add(M);
		CarryFigure(Old, I, A.agent);
	}
	float Longest = 0.f;
	for (const FResetMover& M : Movers)
	{
		Longest = FMath::Max(Longest, M.Depart + TravelSeconds((M.To - M.From).Size(), M.SpeedMps, ResetAccelMps2));
	}
	// Then whoever is leaving (the retired batter, a runner who scored, the side going in):
	// they walk to their dugout in figures of their own, through the reset and on into the
	// next pitch if they need to.
	int32 Leaving = 0;
	for (uint32 I = 0; I < From.n_actors; ++I)
	{
		const BbActor& A = From.actors[I];
		if (!Find(To, A.agent))
		{
			AddExit(Old, A.agent, FVector2D(A.pos[0], A.pos[1]));
			++Leaving;
		}
	}
	ResetDuration = FMath::Max(Longest, 0.6f);
	if (bSideChange)
	{
		// The next half inning does not start with the last one's fielders still out there.
		for (const FExitSlot& X : Exits)
		{
			if (X.bActive)
			{
				const float Left = X.Move.Depart + TravelSeconds((X.Move.To - X.Move.From).Size(), X.Move.SpeedMps, ResetAccelMps2) - ExitClock;
				ResetDuration = FMath::Max(ResetDuration, Left);
			}
		}
	}

	// The ball: whoever ended the play with it throws it to the pitcher so that it reaches
	// him as he gets back to the mound. If nobody has it (a home run, a foul into the seats)
	// the catcher throws a new one. At a change of sides nothing is thrown.
	ResetThrower = -1;
	ResetThrowFirstClip = -1.f;
	ResetBallFrom = FVector(From.ball[0], From.ball[1], From.ball[2]);
	ResetBallTo = Pitcher ? FVector(Pitcher->pos[0], Pitcher->pos[1], 1.4) : FVector(0.0, 18.44, 1.4);
	if (!bSideChange && Pitcher)
	{
		int32 Thrower = -1;
		for (int32 I = 0; I < Movers.Num(); ++I)
		{
			if (From.holder == (int32)Movers[I].Agent && Find(From, Movers[I].Agent))
			{
				Thrower = I;
			}
		}
		if (Thrower < 0)
		{
			for (int32 I = 0; I < Movers.Num(); ++I)
			{
				if ((Movers[I].Agent & 0xFF) == 0x82)
				{
					Thrower = I;
					ResetBallFrom = FVector(Movers[I].From.X, Movers[I].From.Y, 1.2);
				}
			}
		}
		if (Thrower >= 0 && Movers[Thrower].Agent != Pitcher->agent)
		{
			FResetMover& M = Movers[Thrower];
			const FResetMover* PitcherMove = Movers.FindByPredicate([Pitcher](const FResetMover& X) { return X.Agent == Pitcher->agent; });
			const float PitcherArrives = PitcherMove ? PitcherMove->Depart + TravelSeconds((PitcherMove->To - PitcherMove->From).Size(), PitcherMove->SpeedMps, ResetAccelMps2) : 0.f;
			const float Dist = (ResetBallTo - FVector(M.From.X, M.From.Y, 1.2)).Size();
			ResetThrower = Thrower;
			ResetFlight = FMath::Clamp(Dist / ResetThrowMps, 0.35f, 2.5f);
			ResetThrowAt = FMath::Max(0.55f, PitcherArrives - ResetFlight);
			// He stands for the throw and its follow-through (turning to go as it ends), then
			// makes up the time.
			const float Walk = (M.To - M.From).Size();
			M.Depart = ResetThrowAt + (ThrowClipS - ThrowReleaseS) + (Walk >= ShuffleMaxM ? ResetTurnS : 0.f);
			M.SpeedMps = FMath::Max(M.SpeedMps, SpeedFor(Walk, 3.f, ResetAccelMps2, ResetHurryMps));
			ResetDuration = FMath::Max3(ResetDuration, ResetThrowAt + ResetFlight + 0.3f, M.Depart + TravelSeconds(Walk, M.SpeedMps, ResetAccelMps2));
		}
	}
	if (bStagingLog)
	{
		UE_LOG(LogBb, Display, TEXT("BB_RESET play=%u side_change=%d staying=%d leaving=%d duration=%.2f throw_at=%.2f"),
			Next.Info.play_index, bSideChange ? 1 : 0, Movers.Num(), Leaving, ResetDuration, ResetThrower >= 0 ? ResetThrowAt : -1.f);
	}
	return true;
}

float ABbDirector::AddExit(const TArray<FFigureState>& Old, uint16 Agent, const FVector2D& From)
{
	FResetMover M;
	M.Agent = Agent;
	M.From = From;
	M.To = Dugout(Agent);
	const float Dist = (M.To - M.From).Size();
	M.SpeedMps = Dist < ResetWalkWithinM ? ResetWalkMps : ResetJogMps;
	if (TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2) > ResetMaxSeconds)
	{
		M.SpeedMps = FMath::Max(M.SpeedMps, SpeedFor(Dist, ResetMaxSeconds, ResetAccelMps2, ResetHurryMps));
	}
	M.Depart = ExitClock + (Dist >= ShuffleMaxM ? ResetTurnS : 0.f);
	int32 Slot = Exits.IndexOfByPredicate([](const FExitSlot& X) { return !X.bActive; });
	if (Slot == INDEX_NONE)
	{
		// Out of figures (a side change needs at most 16 of 24): the one nearest his dugout
		// goes in now.
		float Least = TNumericLimits<float>::Max();
		for (int32 X = 0; X < Exits.Num(); ++X)
		{
			const float Left = (Exits[X].Move.To - ExitAt(Exits[X].Move)).Size();
			if (Left < Least)
			{
				Least = Left;
				Slot = X;
			}
		}
		EndExit(Slot, TEXT("overflow"));
	}
	Exits[Slot].bActive = true;
	Exits[Slot].Move = M;
	CarryFigure(Old, MaxActors + Slot, Agent);
	return M.Depart - ExitClock + TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2);
}

FVector2D ABbDirector::ExitAt(const FResetMover& M) const
{
	const FVector2D Path = M.To - M.From;
	const double Dist = Path.Size();
	if (Dist <= 1e-3)
	{
		return M.To;
	}
	return M.From + Path * (TravelGone(Dist, M.SpeedMps, ResetAccelMps2, ExitClock - M.Depart) / Dist);
}

void ABbDirector::EndExit(int32 Index, const TCHAR* Reason)
{
	FExitSlot& X = Exits[Index];
	if (!X.bActive)
	{
		return;
	}
	const float Left = (X.Move.To - ExitAt(X.Move)).Size();
	if (FCString::Strcmp(Reason, TEXT("arrived")) == 0)
	{
		++ExitsArrived;
	}
	else if (FCString::Strcmp(Reason, TEXT("rejoined")) == 0)
	{
		++ExitsRejoined;
	}
	else if (Left > 0.05f)
	{
		++ExitsCut;
	}
	if (bStagingLog)
	{
		UE_LOG(LogBb, Display, TEXT("BB_EXIT play=%u agent=0x%03x %s left=%.2f m after %.2f s"), Current.Info.play_index, X.Move.Agent, Reason, Left, ExitClock - X.Move.Depart);
	}
	X.bActive = false;
}

void ABbDirector::PurgeExits(const BbFrame& Frame)
{
	for (int32 X = 0; X < Exits.Num(); ++X)
	{
		for (uint32 I = 0; Exits[X].bActive && I < Frame.n_actors; ++I)
		{
			if (Frame.actors[I].agent == Exits[X].Move.Agent)
			{
				EndExit(X, TEXT("rejoined"));
			}
		}
	}
}

ABbDirector::FPoseInput ABbDirector::StagedInput(const FResetMover& M, float T) const
{
	FPoseInput In;
	In.bStaged = true;
	const FVector2D Path = M.To - M.From;
	const float Dist = Path.Size();
	if (Dist < 1e-3f)
	{
		return In;
	}
	In.Dir = Path / Dist;
	const float S = T - M.Depart;
	const float Total = TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2);
	In.bMoving = S >= 0.f && S < Total;
	In.PlannedMps = In.bMoving ? TravelSpeed(Dist, M.SpeedMps, ResetAccelMps2, S) : 0.f;
	In.bFacePath = Dist >= ShuffleMaxM && S >= -ResetTurnS && S < Total;
	return In;
}

float ABbDirector::TravelSpeed(float Dist, float V, float A, float T)
{
	const float Total = TravelSeconds(Dist, V, A);
	if (T <= 0.f || T >= Total)
	{
		return 0.f;
	}
	const float Peak = FMath::Min(V, FMath::Sqrt(A * Dist));
	const float Ramp = Peak / A;
	if (T < Ramp)
	{
		return A * T;
	}
	if (T > Total - Ramp)
	{
		return A * (Total - T);
	}
	return Peak;
}

float ABbDirector::TravelSeconds(float Dist, float V, float A)
{
	if (Dist <= 0.f || V <= 0.f || A <= 0.f)
	{
		return 0.f;
	}
	// Too short to reach V: speed up halfway, slow down the rest.
	if (Dist < V * V / A)
	{
		return 2.f * FMath::Sqrt(Dist / A);
	}
	return Dist / V + V / A;
}

float ABbDirector::TravelGone(float Dist, float V, float A, float T)
{
	const float Total = TravelSeconds(Dist, V, A);
	if (T <= 0.f || Total <= 0.f)
	{
		return 0.f;
	}
	if (T >= Total)
	{
		return Dist;
	}
	const float Peak = FMath::Min(V, FMath::Sqrt(A * Dist)); // top speed actually reached
	const float Ramp = Peak / A;
	if (T < Ramp)
	{
		return 0.5f * A * T * T;
	}
	if (T > Total - Ramp)
	{
		const float Left = Total - T;
		return Dist - 0.5f * A * Left * Left;
	}
	return 0.5f * Peak * Ramp + Peak * (T - Ramp);
}

float ABbDirector::SpeedFor(float Dist, float Seconds, float A, float Cap)
{
	// Dist / V + V / A = Seconds; the smaller root is the gentler pace. No root means even
	// speeding up and slowing down the whole way is not enough.
	const float Disc = A * A * Seconds * Seconds - 4.f * A * Dist;
	if (Disc < 0.f)
	{
		return Cap;
	}
	return FMath::Min(Cap, 0.5f * (A * Seconds - FMath::Sqrt(Disc)));
}

void ABbDirector::ResetFrame(float T, BbFrame& Out, bool& bBallShown) const
{
	FMemory::Memzero(Out);
	Out.n_actors = Movers.Num();
	Out.holder = -1;
	Out.release_in_s = -1.f;
	for (int32 I = 0; I < Movers.Num(); ++I)
	{
		const FResetMover& M = Movers[I];
		const FVector2D Path = M.To - M.From;
		const double Dist = Path.Size();
		const double Gone = TravelGone(Dist, M.SpeedMps, ResetAccelMps2, T - M.Depart);
		const FVector2D P = Dist > 1e-3 ? M.From + Path * (Gone / Dist) : M.To;
		Out.actors[I].agent = M.Agent;
		Out.actors[I].pos[0] = P.X;
		Out.actors[I].pos[1] = P.Y;
	}
	FVector B = ResetBallTo;
	bBallShown = false;
	if (ResetThrower >= 0)
	{
		// The thrower stands still until the ball is gone, holding it as the sim does: 1.2 m
		// above his feet.
		const FVector Hand(Movers[ResetThrower].From.X, Movers[ResetThrower].From.Y, 1.2);
		const float U = (T - ResetThrowAt) / ResetFlight;
		if (U < 0.f)
		{
			B = FMath::Lerp(ResetBallFrom, Hand, FMath::Clamp(T / 0.2f, 0.f, 1.f));
			bBallShown = true;
			Out.holder = Movers[ResetThrower].Agent;
			Out.release_in_s = ResetThrowAt - T;
		}
		else if (U < 1.f)
		{
			// A lob under gravity: the arc's height follows from its time in the air.
			B = FMath::Lerp(Hand, ResetBallTo, U) + FVector(0.0, 0.0, 4.9 * ResetFlight * ResetFlight * U * (1.f - U));
			bBallShown = true;
		}
	}
	Out.ball[0] = B.X;
	Out.ball[1] = B.Y;
	Out.ball[2] = B.Z;
}

void ABbDirector::PoseFigure(int32 Slot, const BbActor& A, const BbFrame& Frame, const FPoseInput& In, float SimDt, float DeltaSeconds)
{
	ASkeletalMeshActor* F = Figures[Slot];
	USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
	FFigureState& St = FigureState[Slot];
	const FVector Foot = ToUE(A.pos);
	const float PathYaw = ToUE(In.Dir.X, In.Dir.Y, 0.0).Rotation().Yaw;
	// Speed from how far he moved in sim time. A new agent in this slot, or a jump of
	// more than a sprinter could cover (a new play), is a placement, not a run.
	if (St.Agent != A.agent)
	{
		St.Agent = A.agent;
		St.SpeedCm = 0.f;
		St.Gait = -1;
		St.Yaw = In.bFacePath ? PathYaw : YawToPlate(Foot);
	}
	else if (SimDt > 1e-5f)
	{
		const float Raw = (Foot - St.Pos).Size2D() / SimDt;
		St.SpeedCm = Raw > 1400.f ? 0.f : FMath::FInterpTo(St.SpeedCm, Raw, DeltaSeconds, 12.f);
	}
	// Between pitches the client's own plan says how fast he is going, wherever the frames
	// happened to fall.
	if (In.bStaged)
	{
		St.SpeedCm = In.PlannedMps * 100.f;
	}
	// A short staged move is a shuffle: back-step or sidestep, still facing the plate.
	const bool bShuffleClips = AnimWalkBack && AnimWalkLeft && AnimWalkRight;
	const bool bShuffle = In.bStaged && In.bMoving && !In.bFacePath && bShuffleClips;
	// Facing: where he is going if he is moving, otherwise the plate (everyone watches the
	// pitch; the batter stands side-on to the pitcher). Between pitches it comes from the
	// planned path, so he turns before he sets off rather than after.
	const FVector Move = Foot - St.Pos;
	float WantYaw = St.Yaw;
	if (In.bStaged)
	{
		WantYaw = In.bFacePath || (In.bMoving && !bShuffleClips) ? PathYaw : YawToPlate(Foot);
	}
	else if (St.SpeedCm > 90.f && Move.SizeSquared2D() > 1e-4)
	{
		WantYaw = Move.Rotation().Yaw;
	}
	else if (St.SpeedCm < 40.f)
	{
		WantYaw = YawToPlate(Foot);
	}
	// Baseball motions, driven by play time so they hold their place when paused or sped up.
	// A player walking off makes none.
	const bool bMotions = !In.bExiting;
	const bool bFielder = (A.agent & 0x80) != 0;
	const int32 Position = A.agent & 0x7F;
	const bool bStill = St.SpeedCm < 40.f;
	const UAnimSequence* Manual = nullptr;
	float ClipTime = 0.f;
	if (bMotions && bFielder && Position == 1 && !In.bStaged && PlayTime < 0.8f && AnimPitch)
	{
		// The delivery: the clip's strike (1.0 s in) lands on the release (play time 0).
		Manual = AnimPitch;
		ClipTime = PlayTime + 1.0f;
		WantYaw = YawToPlate(Foot);
	}
	else if (bMotions && bFielder && AnimThrow && In.ThrowClip >= 0.f && In.ThrowClip <= ThrowClipS)
	{
		// A throw: the clip's release (ThrowReleaseS in) lands on the known release time, and
		// its phase follows from that time every frame, follow-through included. He turns
		// toward where the ball is going. Possession comes from the sim's record (or the
		// staging's plan), never from how near the ball is.
		Manual = AnimThrow;
		ClipTime = In.ThrowClip;
		WantYaw = In.ThrowYaw;
	}
	if (bMotions && !Manual && bFielder && Position == 2 && bStill && !In.bStaged && PlayTime < 0.6f && AnimCrouch)
	{
		// The catcher squats to receive the pitch.
		Manual = AnimCrouch;
		ClipTime = 0.1f;
	}
	if (bMotions && !bFielder && Frame.has_bat != 0 && AnimStance)
	{
		// The batter: whoever is at the bat's knob. Hands up in a stance; as the bat comes
		// around, his body turns with it.
		const FVector Knob = ToUE(Frame.bat_knob);
		if ((FVector(Knob.X, Knob.Y, 0) - FVector(Foot.X, Foot.Y, 0)).Size() < 160.f && bStill)
		{
			Manual = AnimStance;
			ClipTime = 0.f;
			const FVector Along = ToUE(Frame.bat_tip) - Knob;
			if (Along.Size2D() > 45.f)
			{
				// Hips lead the barrel: face between the bat's direction and the pitcher.
				WantYaw = (Along.GetSafeNormal2D() + ToUE(0, 1, 0).GetSafeNormal() * 0.6).Rotation().Yaw;
			}
		}
	}
	// Turns ease on play time, so a pause holds them and fast playback keeps up.
	St.Yaw = FMath::RInterpTo(FRotator(0, St.Yaw, 0), FRotator(0, WantYaw, 0), SimDt, Manual ? 18.f : TurnRate).Yaw;
	St.Pos = Foot;
	F->SetActorLocation(Foot);
	// The mannequin's mesh faces +Y in its own frame.
	F->SetActorRotation(FRotator(0.f, St.Yaw - 90.f, 0.f));

	if (Manual)
	{
		if (St.Manual != Manual)
		{
			St.Manual = Manual;
			St.Gait = -1;
			C->PlayAnimation(const_cast<UAnimSequence*>(Manual), false);
		}
		C->SetPlayRate(0.f);
		C->SetPosition(FMath::Clamp(ClipTime, 0.f, Manual->GetPlayLength() - 0.02f), false);
	}
	else
	{
		St.Manual = nullptr;
	}
	// The gait: from the plan between pitches (a step for even the shortest move, idle once
	// there), from measured speed during a play.
	int32 Gait = 0;
	if (In.bStaged)
	{
		if (!In.bMoving)
		{
			Gait = 0;
		}
		else if (bShuffle)
		{
			// Which way the path runs from where he faces (Unreal yaw turns clockwise seen
			// from above: positive is to his right).
			const float Rel = FRotator::NormalizeAxis(PathYaw - St.Yaw);
			Gait = FMath::Abs(Rel) < 45.f ? 1 : (FMath::Abs(Rel) > 135.f ? 3 : (Rel > 0.f ? 5 : 4));
		}
		else
		{
			Gait = In.PlannedMps < 2.5f ? 1 : 2;
		}
	}
	else
	{
		Gait = St.SpeedCm < 40.f ? 0 : (St.SpeedCm < 250.f ? 1 : 2);
	}
	if (!Manual && Gait != St.Gait)
	{
		UAnimSequence* Clips[6] = { AnimIdle, AnimWalk, AnimJog, AnimWalkBack, AnimWalkLeft, AnimWalkRight };
		St.Gait = Gait;
		C->PlayAnimation(Clips[Gait], true);
	}
	// The walk clips cover about 2.3 m/s and the jog about 5 m/s; play faster for a sprint,
	// slower for a step.
	if (!Manual)
	{
		const float Floor = In.bStaged ? 0.4f : 0.6f;
		const float Rate = Gait == 0 ? 1.f : FMath::Clamp(St.SpeedCm / (Gait == 2 ? 500.f : 230.f), Floor, 1.9f);
		C->SetPlayRate(SimDt <= 1e-5f ? 0.f : Rate * (SimDt / FMath::Max(DeltaSeconds, 1e-5f)));
	}

	// Uniform colors: home white, away blue; fielders a shade apart from batters and runners.
	const bool bHome = (A.agent >> 8) != 0;
	const int32 Team = (bHome ? 2 : 0) + (bFielder ? 1 : 0);
	if (Team != St.Team)
	{
		St.Team = Team;
		const FLinearColor Col = bHome ? (bFielder ? FLinearColor(0.8f, 0.8f, 0.85f) : FLinearColor(0.95f, 0.95f, 0.95f)) : (bFielder ? FLinearColor(0.08f, 0.12f, 0.4f) : FLinearColor(0.15f, 0.2f, 0.6f));
		for (int32 M = 0; M < C->GetNumMaterials(); ++M)
		{
			// The mannequin's own material, with its painted panels tinted to the uniform.
			UMaterialInstanceDynamic* Mat = Cast<UMaterialInstanceDynamic>(C->GetMaterial(M));
			if (!Mat)
			{
				Mat = C->CreateDynamicMaterialInstance(M);
			}
			if (Mat)
			{
				Mat->SetVectorParameterValue(TEXT("Paint Tint"), Col);
			}
		}
	}
}

void ABbDirector::UpdateCamera(const BbFrame& Frame, float DeltaSeconds)
{
	UCameraComponent* C = Camera->GetCameraComponent();
	const FVector BallUE = ToUE(Frame.ball);
	// Mode 0 cuts like a broadcast: the pitch from beyond center field through a long lens,
	// then the play from high behind home, following the ball. Mode 1 stays high behind
	// home; mode 2 looks straight down on the whole field.
	if (CameraMode == 2)
	{
		Camera->SetActorLocation(ToUE(0, 55, 230));
		Camera->SetActorRotation(FRotator(-90.f, 0.f, 0.f));
		C->SetFieldOfView(50.f);
		ShownCam = 2;
		return;
	}
	const bool bFollow = bBallInPlay || CameraMode == 1;
	const int32 Want = bFollow ? 1 : 0;
	if (Want != ShownCam)
	{
		ShownCam = Want;
		Camera->SetActorLocation(bFollow ? ToUE(0, -34, 19) : ToUE(-9, 136, 16));
		CamLook = bFollow ? (bBallInPlay ? FVector(BallUE.X, BallUE.Y, 0.0) : ToUE(0, 16, 0)) : ToUE(0, 6, 1.0);
		C->SetFieldOfView(bFollow ? 48.f : 8.5f);
	}
	if (bFollow)
	{
		const FVector Target = bBallInPlay ? FVector(BallUE.X, BallUE.Y, FMath::Min(BallUE.Z, 1500.0) * 0.5) : ToUE(0, 16, 0);
		CamLook = FMath::VInterpTo(CamLook, Target, DeltaSeconds, 4.f);
		C->SetFieldOfView(FMath::FInterpTo(C->FieldOfView, bBallInPlay ? 42.f : 55.f, DeltaSeconds, 2.f));
	}
	Camera->SetActorRotation((CamLook - Camera->GetActorLocation()).Rotation());
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

	// Keys: Space pauses, - and = change speed, C changes camera, Esc or Q quits.
	if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
	{
		if (PC->WasInputKeyJustPressed(EKeys::SpaceBar))
		{
			bPaused = !bPaused;
		}
		if (PC->WasInputKeyJustPressed(EKeys::Equals))
		{
			Speed = FMath::Min(Speed * 1.5f, 16.f);
		}
		if (PC->WasInputKeyJustPressed(EKeys::Hyphen))
		{
			Speed = FMath::Max(Speed / 1.5f, 0.1f);
		}
		if (PC->WasInputKeyJustPressed(EKeys::C))
		{
			CameraMode = (CameraMode + 1) % 3;
		}
		if (PC->WasInputKeyJustPressed(EKeys::Escape) || PC->WasInputKeyJustPressed(EKeys::Q))
		{
			FPlatformMisc::RequestExit(false);
		}
	}

	// The first pitch resolves on a worker like every other (PLAN §3.4): until it is ready
	// the park stands empty.
	if (!Current.bValid && !bGameOver && Sim && bPending && Pending.IsReady())
	{
		FBbPlay First = Pending.Get();
		bPending = false;
		BeginPlayback(MoveTemp(First));
		if (!bGameOver)
		{
			RequestNext();
		}
	}
	else if (!Current.bValid && !bGameOver && Sim)
	{
		if (ABbHUD* Hud = GetWorld()->GetFirstPlayerController() ? Cast<ABbHUD>(GetWorld()->GetFirstPlayerController()->GetHUD()) : nullptr)
		{
			Hud->Scoreboard = TEXT("Warming up");
		}
	}

	if (Current.bValid && Sim)
	{
		const float SimDt = bPaused ? 0.f : DeltaSeconds * Speed;
		PlayTime += SimDt;
		ExitClock += SimDt;
		const float T = FMath::Clamp(PlayTime, 0.f, Current.Info.duration_s);
		BbFrame Frame;
		bool bHave = false;
		// During the wind-up the ball is in the pitcher's glove.
		bool bBallShown = PlayTime >= 0.f;
		if (bReset)
		{
			ResetTime += SimDt;
			ResetFrame(ResetTime, Frame, bBallShown);
			bHave = true;
		}
		else
		{
			bHave = Sim->Sample(Current, T, Frame);
			// Throws coming up, from every frame since the last one drawn.
			NoteThrows(ThrowsSampledTo, T);
			ThrowsSampledTo = FMath::Max(ThrowsSampledTo, T);
		}
		// Players who have reached their dugout are gone into it.
		for (int32 X = 0; X < Exits.Num(); ++X)
		{
			const FResetMover& M = Exits[X].Move;
			if (Exits[X].bActive && ExitClock - M.Depart >= TravelSeconds((M.To - M.From).Size(), M.SpeedMps, ResetAccelMps2))
			{
				EndExit(X, TEXT("arrived"));
			}
		}
		if (bHave)
		{
			Ball->SetActorLocation(ToUE(Frame.ball));
			Ball->SetActorHiddenInGame(!bBallShown);
			{
				// Above every ground layer (the mound's top is at 5 mm), below anything else.
				const float H = FMath::Max(0.f, Frame.ball[2]);
				const float Size = BallShadowM * FMath::Lerp(1.f, 0.55f, FMath::Clamp(H / 30.f, 0.f, 1.f));
				BallShadow->SetActorLocation(ToUE(Frame.ball[0], Frame.ball[1], 0.012));
				BallShadow->SetActorScale3D(FVector(Size, Size, 0.002));
				BallShadow->SetActorHiddenInGame(!bBallShown);
			}
			Bat->SetActorHiddenInGame(Frame.has_bat == 0);
			if (Frame.has_bat != 0)
			{
				const FVector Knob = ToUE(Frame.bat_knob);
				const FVector Tip = ToUE(Frame.bat_tip);
				Bat->SetActorLocation((Knob + Tip) * 0.5);
				// The cylinder's long axis is its local Z.
				Bat->SetActorRotation(FRotationMatrix::MakeFromZ(Tip - Knob).Rotator());
			}
			// Slots below MaxActors show the frame's actors; the rest, players walking off.
			const int32 Pool = Bodies.Num();
			const bool bFigure = Figures.Num() == Pool;
			for (int32 I = 0; I < Pool; ++I)
			{
				BbActor A{};
				FPoseInput In;
				bool bShow = false;
				if (I < MaxActors)
				{
					bShow = I < (int32)Frame.n_actors;
					if (bShow)
					{
						A = Frame.actors[I];
					}
					if (bShow && bReset && I < Movers.Num())
					{
						In = StagedInput(Movers[I], ResetTime);
						if (I == ResetThrower)
						{
							// The return throw's clip phase from its known release, every frame.
							In.ThrowClip = ResetTime - (ResetThrowAt - ThrowReleaseS);
							In.ThrowYaw = (ToUE(ResetBallTo.X, ResetBallTo.Y, 0.0) - ToUE(Movers[I].From.X, Movers[I].From.Y, 0.0)).Rotation().Yaw;
							if (bFigure && ResetThrowFirstClip < 0.f && In.ThrowClip >= 0.f && In.ThrowClip <= ThrowClipS)
							{
								ResetThrowFirstClip = In.ThrowClip;
							}
						}
					}
					else if (bShow && !bReset)
					{
						for (FThrowNote& N : Throws)
						{
							const float Clip = PlayTime - (N.ReleaseAt - ThrowReleaseS);
							if (N.Agent == A.agent && Clip >= 0.f && Clip <= ThrowClipS)
							{
								In.ThrowClip = Clip;
								In.ThrowYaw = N.Yaw;
								if (bFigure && N.FirstClip < 0.f)
								{
									N.FirstClip = Clip;
								}
							}
						}
					}
				}
				else if (Exits[I - MaxActors].bActive)
				{
					const FResetMover& M = Exits[I - MaxActors].Move;
					const FVector2D P = ExitAt(M);
					bShow = true;
					A.agent = M.Agent;
					A.pos[0] = P.X;
					A.pos[1] = P.Y;
					In = StagedInput(M, ExitClock);
					In.bExiting = true;
				}
				Bodies[I]->SetActorHiddenInGame(!bShow || bFigure);
				Heads[I]->SetActorHiddenInGame(!bShow || bFigure);
				if (bFigure)
				{
					Figures[I]->SetActorHiddenInGame(!bShow);
					if (bShow)
					{
						PoseFigure(I, A, Frame, In, SimDt, DeltaSeconds);
					}
					else
					{
						FigureState[I].Agent = 0xFFFF;
					}
					continue;
				}
				if (!bShow)
				{
					continue;
				}
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
			// The call appears when a viewer would know it: at the umpire's call if the pitch
			// was not hit, at the end of the play if it was (the sim says when: call_s), and
			// never before the umpire has signalled the last out (strike three).
			const float LastCall = In.n_calls > 0 ? In.calls[In.n_calls - 1].t_s : 0.f;
			const bool bKnown = PlayTime >= FMath::Max(In.call_s, LastCall);
			Hud->Call = bKnown ? FString(UTF8_TO_TCHAR(In.summary)) : FString();
			// Outs, though, are called by the umpire as they happen (the sim times each
			// signal: at once on a close play, within a second on a routine one).
			int32 Called = 0;
			while (Called < (int32)In.n_calls && PlayTime >= In.calls[Called].t_s)
			{
				++Called;
			}
			Hud->Umpire.Reset();
			if (Called > 0 && PlayTime < In.calls[Called - 1].t_s + UmpireShowS)
			{
				const BbCall& C = In.calls[Called - 1];
				static const TCHAR* Where[5] = { TEXT(""), TEXT(" at first"), TEXT(" at second"), TEXT(" at third"), TEXT(" at home") };
				// Base 0 is the batter retired at the plate: a caught ball, or strike three.
				const TCHAR* At = C.base == 0 ? (In.exit_velocity_mps > 0.f ? TEXT(", caught") : TEXT("")) : Where[FMath::Min<int32>(C.base, 4)];
				// Emphasis for a bang-bang play in the field, not for strike three.
				Hud->bUmpireClose = C.margin_s <= CloseCallS && !(C.base == 0 && In.exit_velocity_mps <= 0.f);
				Hud->Umpire = FString::Printf(TEXT("OUT%s%s"), Hud->bUmpireClose ? TEXT("!") : TEXT(""), At);
				if (Called > CallsLogged)
				{
					// For unattended checks: when each call went up, to aim screenshots at it.
					UE_LOG(LogBb, Display, TEXT("BB_CALL wall=%.2f play=%u t=%.2f margin=%.2f %s"), Wall, In.play_index, PlayTime, C.margin_s, *Hud->Umpire);
					CallsLogged = Called;
				}
			}
			static const TCHAR* CamNames[3] = { TEXT("broadcast"), TEXT("behind home"), TEXT("overhead") };
			Hud->Footer = FString::Printf(TEXT("Space: %s    - / =: speed %.2gx    C: camera (%s)    Esc or Q: quit"), bPaused ? TEXT("paused") : TEXT("pause"), Speed, CamNames[CameraMode]);
			// The outs on the board are the ones called so far. Runs, count and bases change
			// when the play is known. A play that ends the half inning leaves it on the board,
			// at three outs, until the next pitch (BeginPlayback).
			{
				const BbPlayInfo& Src = bKnown ? In : Before;
				const bool bHalfOver = In.inning != Before.inning || In.top != Before.top;
				const BbPlayInfo& Half = bHalfOver ? Before : Src;
				const int32 Outs = FMath::Min<int32>(Before.outs + Called, 3);
				Hud->Bases = Src.bases;
				Hud->Scoreboard = FString::Printf(TEXT("AWAY %d   HOME %d    %s %d    %d out    %d-%d"), Src.away_runs, Src.home_runs, Half.top ? TEXT("Top") : TEXT("Bot"), Half.inning, Outs, bHalfOver && bKnown ? 0 : Src.balls, bHalfOver && bKnown ? 0 : Src.strikes);
			}
			const float Mph = In.release_speed_mps * 2.23694f;
			// Contact measurements only once the bat has met the ball: before that they would
			// say he is going to swing and hit it.
			Hud->PitchLine = In.exit_velocity_mps > 0.f && In.contact_s >= 0.f && PlayTime >= In.contact_s
				? FString::Printf(TEXT("Pitch %.0f mph   Exit %.0f mph at %.0f deg"), Mph, In.exit_velocity_mps * 2.23694f, FMath::RadiansToDegrees(In.launch_angle_rad))
				: FString::Printf(TEXT("Pitch %.0f mph"), Mph);
		}
		if (bReset)
		{
			if (ResetTime >= ResetDuration)
			{
				if (ResetThrower >= 0 && Figures.Num() > 0)
				{
					ResetThrowFirstClip >= 0.f ? ++ThrowsShown : ++ThrowsMissed;
					if (bStagingLog)
					{
						UE_LOG(LogBb, Display, TEXT("BB_THROW play=%u agent=0x%03x release=%.3f staged=1 first_clip=%.3f%s"), NextPlay.Info.play_index, Movers[ResetThrower].Agent, ResetThrowAt, ResetThrowFirstClip, ResetThrowFirstClip >= 0.f ? TEXT("") : TEXT(" MISSED"));
					}
				}
				bReset = false;
				BeginPlayback(MoveTemp(NextPlay));
				NextPlay = FBbPlay();
			}
		}
		else if (PlayTime >= Current.Info.duration_s + HoldAfter && bPending && Pending.IsReady())
		{
			FBbPlay Next = Pending.Get();
			bPending = false;
			if (Next.bValid && StartReset(Next))
			{
				// The next pitch waits while everyone gets back into place.
				NextPlay = MoveTemp(Next);
				bReset = true;
				ResetTime = 0.f;
				RequestNext();
			}
			else
			{
				BeginPlayback(MoveTemp(Next));
				if (!bGameOver)
				{
					RequestNext();
				}
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
	if (bStagingLog)
	{
		// exits_cut counts players removed short of their dugout (should be 0); throws_missed
		// throws whose clip was never drawn.
		int32 Walking = 0;
		for (const FExitSlot& X : Exits)
		{
			Walking += X.bActive ? 1 : 0;
		}
		UE_LOG(LogBb, Display, TEXT("BB_STAGING speed=%.2f exits_arrived=%d exits_rejoined=%d exits_cut=%d exits_walking=%d throws_shown=%d throws_missed=%d"),
			Speed, ExitsArrived, ExitsRejoined, ExitsCut, Walking, ThrowsShown, ThrowsMissed);
	}
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
	if (Sim && NextPlay.Blob)
	{
		Sim->Release(NextPlay);
	}
	Sim.Reset();
	Super::EndPlay(Reason);
}
