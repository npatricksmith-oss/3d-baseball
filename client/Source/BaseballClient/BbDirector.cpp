#include "BbDirector.h"
#include "BbFigure.h"
#include "BbFrame.h"
#include "BbPark.h"
#include "BbPose.h"
#include "BbGameMode.h"
#include "BbSavePlay.h"
#include "Animation/AnimSequence.h"
#include "Engine/GameViewportClient.h"
#include "Framework/Application/SlateApplication.h"
#include "Styling/CoreStyle.h"
#include "Widgets/Input/SButton.h"
#include "Widgets/Input/SMultiLineEditableTextBox.h"
#include "Widgets/Layout/SBorder.h"
#include "Widgets/SOverlay.h"
#include "Widgets/Text/STextBlock.h"
#include "Animation/SkeletalMeshActor.h"
#include "Animation/Skeleton.h"
#include "Async/Async.h"
#include "Camera/CameraActor.h"
#include "Camera/CameraComponent.h"
#include "Camera/PlayerCameraManager.h"
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
#include "Materials/Material.h"
#include "Materials/MaterialExpressionConstant2Vector.h"
#include "Materials/MaterialExpressionDistance.h"
#include "Materials/MaterialExpressionDivide.h"
#include "Materials/MaterialExpressionMultiply.h"
#include "Materials/MaterialExpressionOneMinus.h"
#include "Materials/MaterialExpressionSaturate.h"
#include "Materials/MaterialExpressionTextureCoordinate.h"
#include "Materials/MaterialExpressionVectorParameter.h"
#include "Misc/CommandLine.h"
#include "Misc/Parse.h"
#include "Misc/Paths.h"
#include "UnrealClient.h"

DEFINE_LOG_CATEGORY_STATIC(LogBb, Log, All);

using BbFrameConv::ToUE;

namespace
{
	const int32 MaxActors = BB_MAX_ACTORS;
	/** Where an Unreal-frame figure at Foot looks to face the plate. */
	float YawToPlate(const FVector& Foot)
	{
		return (ToUE(0, 0.3, 0) - Foot).Rotation().Yaw;
	}

	/** Starts a clip on a figure from its beginning (the figure's animation instance stays
	 *  the one that can lay a coded pose over the clip, BbPose.h). */
	void PlayClip(USkeletalMeshComponent* C, UAnimSequence* Clip, bool bLoop)
	{
		C->SetAnimation(Clip);
		C->Play(bLoop);
	}

	/** Where the catcher takes a pitch, from the rear point of the plate, m (the sim ends a
	 *  taken pitch there), and where he holds his mitt for a target. */
	/** An umpire calls a runner safe this long after he is at the bag; a throw taken at first
	 *  within this long of the batter's getting there is a play on him. */
	const float SafeCallS = 0.3f, SafePlayS = 1.2f;
	const float MittPlaneY = -1.2f;
	const float MittTargetZ = 0.55f;

	/** A bone's place in the clip's component space at Time, cm (its parents composed). */
	FVector ClipBoneAt(const UAnimSequence* Clip, const FReferenceSkeleton& Ref, int32 Bone, double Time)
	{
		FTransform T = FTransform::Identity;
		for (int32 B = Bone; B != INDEX_NONE; B = Ref.GetParentIndex(B))
		{
			FTransform Local;
			Clip->GetBoneTransform(Local, FSkeletonPoseBoneIndex(B), FAnimExtractContext(Time), false);
			T = T * Local;
		}
		return T.GetLocation();
	}

	/**
	 * The ground speed a walk or run clip is animated for, cm/s, measured from the clip itself
	 * (-BbClipSpeeds). The mannequin's clips carry their travel in the root bone, so that is
	 * the speed: how far the root goes in one loop over the loop's length. As a check, Detail
	 * also gives how fast the planted feet pass back under the root (the same speed when the
	 * feet hold the ground) and how much a planted foot itself slides in the clip. A foot is
	 * planted while it is within 2 cm of its lowest point.
	 */
	float ClipGroundSpeed(const UAnimSequence* Clip, FString& Detail)
	{
		const USkeleton* Skel = Clip ? Clip->GetSkeleton() : nullptr;
		if (!Skel)
		{
			return 0.f;
		}
		const FReferenceSkeleton& Ref = Skel->GetReferenceSkeleton();
		const double Len = Clip->GetPlayLength();
		const int32 N = 240;
		const double Dt = Len / N;
		const float RootCmps = (ClipBoneAt(Clip, Ref, 0, Len) - ClipBoneAt(Clip, Ref, 0, 0.0)).Size2D() / Len;
		double PassSum = 0.0, SlideSum = 0.0;
		int32 Count = 0;
		for (const TCHAR* Name : { TEXT("ball_l"), TEXT("ball_r") })
		{
			const int32 Bone = Ref.FindBoneIndex(FName(Name));
			if (Bone == INDEX_NONE)
			{
				continue;
			}
			TArray<FVector> Foot, Root;
			double Low = TNumericLimits<double>::Max();
			for (int32 I = 0; I <= N; ++I)
			{
				Foot.Add(ClipBoneAt(Clip, Ref, Bone, I * Dt));
				Root.Add(ClipBoneAt(Clip, Ref, 0, I * Dt));
				Low = FMath::Min(Low, Foot.Last().Z);
			}
			for (int32 I = 0; I < N; ++I)
			{
				if (Foot[I].Z < Low + 2.0 && Foot[I + 1].Z < Low + 2.0)
				{
					PassSum += ((Foot[I + 1] - Root[I + 1]) - (Foot[I] - Root[I])).Size2D() / Dt;
					SlideSum += (Foot[I + 1] - Foot[I]).Size2D() / Dt;
					++Count;
				}
			}
		}
		Detail = FString::Printf(TEXT("length=%.3f s root_motion=%d planted_foot_passes_root=%.2f m/s planted_foot_slides=%.2f m/s"),
			Len, Clip->HasRootMotion() ? 1 : 0, Count ? PassSum / Count / 100.0 : 0.0, Count ? SlideSum / Count / 100.0 : 0.0);
		return RootCmps;
	}
}

namespace
{
	/** A dark disc on the engine's 1 m plane, solid in the middle and fading to nothing over the
	 *  outer SoftFrac of its radius: the ball's shadow, soft-edged (owner, 2026-11-16). Built in
	 *  code like the park's materials; null outside the editor build, where the hard disc stays. */
	UMaterialInterface* SoftDiscMaterial(float SoftFrac)
	{
#if WITH_EDITOR
		UMaterial* M = NewObject<UMaterial>(GetTransientPackage(), NAME_None, RF_Transient);
		M->BlendMode = BLEND_Translucent;
		M->SetShadingModel(MSM_Unlit);
		UMaterialExpressionVectorParameter* Color = NewObject<UMaterialExpressionVectorParameter>(M);
		Color->ParameterName = TEXT("Color");
		Color->DefaultValue = FLinearColor(0.02f, 0.03f, 0.02f);
		UMaterialExpressionTextureCoordinate* Uv = NewObject<UMaterialExpressionTextureCoordinate>(M);
		UMaterialExpressionConstant2Vector* Mid = NewObject<UMaterialExpressionConstant2Vector>(M);
		Mid->R = 0.5f;
		Mid->G = 0.5f;
		// Distance from the middle as a share of the radius: 0 at the centre, 1 at the rim.
		UMaterialExpressionDistance* Dist = NewObject<UMaterialExpressionDistance>(M);
		Dist->A.Connect(0, Uv);
		Dist->B.Connect(0, Mid);
		UMaterialExpressionMultiply* Share = NewObject<UMaterialExpressionMultiply>(M);
		Share->A.Connect(0, Dist);
		Share->ConstB = 2.f;
		// Opacity (1 − share) / SoftFrac, held to 0..1: solid inside, a linear fade at the rim.
		UMaterialExpressionOneMinus* Inward = NewObject<UMaterialExpressionOneMinus>(M);
		Inward->Input.Connect(0, Share);
		UMaterialExpressionDivide* Ramp = NewObject<UMaterialExpressionDivide>(M);
		Ramp->A.Connect(0, Inward);
		Ramp->ConstB = FMath::Max(SoftFrac, 1e-3f);
		UMaterialExpressionSaturate* Opacity = NewObject<UMaterialExpressionSaturate>(M);
		Opacity->Input.Connect(0, Ramp);
		for (UMaterialExpression* E : { (UMaterialExpression*)Color, (UMaterialExpression*)Uv, (UMaterialExpression*)Mid, (UMaterialExpression*)Dist, (UMaterialExpression*)Share, (UMaterialExpression*)Inward, (UMaterialExpression*)Ramp, (UMaterialExpression*)Opacity })
		{
			M->GetExpressionCollection().AddExpression(E);
		}
		M->GetEditorOnlyData()->EmissiveColor.Connect(0, Color);
		M->GetEditorOnlyData()->Opacity.Connect(0, Opacity);
		M->PostEditChange();
		return M;
#else
		return nullptr;
#endif
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

	// The park (BbPark): the sun and sky now, the field and stands once the sim has said what
	// park the game is in.
	Park = GetWorld()->SpawnActor<ABbPark>();
	Park->BuildLights();

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
	AnimDive = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_WallJump.MM_WallJump"));
	AnimWalkBack = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Bwd.MF_Unarmed_Walk_Bwd"));
	AnimWalkLeft = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Left.MF_Unarmed_Walk_Left"));
	AnimWalkRight = LoadObject<UAnimSequence>(nullptr, TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Walk/MF_Unarmed_Walk_Right.MF_Unarmed_Walk_Right"));
	const bool bFigures = FigureMesh && AnimIdle && AnimWalk && AnimJog;
	if (bFigures && !FParse::Param(FCommandLine::Get(), TEXT("BbStockMotions")))
	{
		Skel.Init(FigureMesh);
	}
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
		ASkeletalMeshActor* F = GetWorld()->SpawnActor<ABbFigure>(FVector::ZeroVector, FRotator::ZeroRotator);
		USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
		C->SetMobility(EComponentMobility::Movable);
		C->SetSkeletalMeshAsset(FigureMesh);
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		// One clip at a time, with a coded pose laid over it when a motion calls for one.
		C->SetAnimInstanceClass(UBbPoseAnim::StaticClass());
		// The director poses the figures; they work their bones out after it has.
		C->AddTickPrerequisiteActor(this);
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
	// The ball larger than life (BallDrawnScale; owner, 2026-11-16).
	Ball = Shape(Sphere, FVector::ZeroVector, FVector(BallDrawnM), FLinearColor(1.f, 1.f, 0.9f));
	// The ball's shadow as a viewer reads it: a dark disc on the ground straight under the
	// ball (the sun's own shadow of so small a ball is lost at a distance and falls well
	// to the side of a high fly), its rim faded (BallShadowSoftFrac). Its size follows the
	// ball's height, so height reads too.
	if (UMaterialInterface* Soft = SoftDiscMaterial(BallShadowSoftFrac))
	{
		BallShadow = Shape(Plane, FVector::ZeroVector, FVector(BallShadowM, BallShadowM, 1.0), FLinearColor(0.02f, 0.03f, 0.02f));
		BallShadow->GetStaticMeshComponent()->SetMaterial(0, Soft);
	}
	else
	{
		BallShadow = Shape(Cylinder, FVector::ZeroVector, FVector(BallShadowM, BallShadowM, 0.002), FLinearColor(0.02f, 0.03f, 0.02f));
	}
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
	// Exposure and the grade are the park's, for every camera (ABbPark::BuildLights).
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
	// The game is started here, before anything is drawn, because the park comes from it.
	TUniquePtr<FBbSim> Started = MakeUnique<FBbSim>();
	if (!Started->Start(Lib, Seed))
	{
		bGameOver = true;
		return;
	}
	UE_LOG(LogBb, Display, TEXT("Sim loaded from %s, seed %llu"), *Lib, Seed);
	Park->Build(Started->Park());
	// The figures are dressed with the park's paints, so they are built after it.
	bMannequin = FParse::Param(FCommandLine::Get(), TEXT("BbMannequin"));
	if (bFigures && !bMannequin)
	{
		FigureShop = NewObject<UBbFigureShop>(this);
		if (FigureShop->Init(FigureMesh, Park))
		{
			for (ASkeletalMeshActor* F : Figures)
			{
				CastChecked<ABbFigure>(F)->Build(FigureShop);
			}
			// The turned bat in place of the cylinder: its knob at its origin.
			Bat->GetStaticMeshComponent()->SetStaticMesh(FigureShop->BatMesh());
			Bat->GetStaticMeshComponent()->SetMaterial(0, FigureShop->BatPaint());
			bTurnedBat = true;
		}
		else
		{
			FigureShop = nullptr;
		}
	}
	FParse::Value(Cmd, TEXT("BbSpeed="), Speed);
	FParse::Value(Cmd, TEXT("BbCamera="), CameraMode);
	FParse::Value(Cmd, TEXT("BbShots="), ShotsLeft);
	FParse::Value(Cmd, TEXT("BbRushAt="), RushAt);
	FParse::Value(Cmd, TEXT("BbReplayAt="), ReplayAt);
	FParse::Value(Cmd, TEXT("BbSaveAt="), SaveAt);
	SeedShown = Seed;
	FParse::Value(Cmd, TEXT("BbShotEvery="), ShotEvery);
	FParse::Value(Cmd, TEXT("BbShotStart="), NextShot);
	FParse::Value(Cmd, TEXT("BbBenchSeconds="), BenchSeconds);
	FParse::Value(Cmd, TEXT("BbDiveShots="), DiveShotsLeft);
	FParse::Value(Cmd, TEXT("BbSlideShots="), SlideShotsLeft);
	FParse::Value(Cmd, TEXT("BbBoxShots="), BoxShotAtPlay);
	FParse::Value(Cmd, TEXT("BbBoxHold="), BoxHoldS);
	FParse::Value(Cmd, TEXT("BbParkShotsAt="), ParkShotsAt);
	ParkShot = FParse::Param(Cmd, TEXT("BbParkShots")) ? 0 : -1;
	{
		int32 Cut = 1;
		FParse::Value(Cmd, TEXT("BbSideCut="), Cut);
		bSideCut = Cut != 0;
	}
	bStagingLog = FParse::Param(Cmd, TEXT("BbStagingLog"));
	bMotionShots = FParse::Param(Cmd, TEXT("BbMotionShots"));
	if (FParse::Param(Cmd, TEXT("BbClipSpeeds")))
	{
		// The pace each gait clip is animated for, to check the staging speeds against.
		const UAnimSequence* Gaits[5] = { AnimWalk, AnimJog, AnimWalkBack, AnimWalkLeft, AnimWalkRight };
		for (const UAnimSequence* Clip : Gaits)
		{
			FString Detail;
			const float Cmps = ClipGroundSpeed(Clip, Detail);
			UE_LOG(LogBb, Display, TEXT("BB_CLIP %s ground_speed=%.2f m/s %s"), Clip ? *Clip->GetName() : TEXT("(missing)"), Cmps / 100.f, *Detail);
		}
	}

	// Dive preview (-BbAnimTest=dive): candidate clips, each held at a fraction of its length,
	// on figures tipped forward as a dive lays them out (BbAnimTilt degrees, default 80), seen
	// side-on diving toward screen right, for choosing the dive's stand-in. No game is played.
	FString AnimTest;
	if (FParse::Value(Cmd, TEXT("BbAnimTest="), AnimTest) && AnimTest == TEXT("dive") && Figures.Num() >= 8)
	{
		const TCHAR* Clips[8] = {
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Fall_Loop.MM_Fall_Loop"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Jump.MM_Jump"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Dash.MM_Dash"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_WallJump.MM_WallJump"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_02.MM_Attack_02"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_Attack_03.MM_Attack_03"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Jump/MM_Land.MM_Land"),
			TEXT("/Game/Characters/Mannequins/Anims/Unarmed/Attack/MM_ChargedAttack.MM_ChargedAttack"),
		};
		float Frac = 0.5f;
		float Tilt = 80.f;
		FParse::Value(Cmd, TEXT("BbAnimFrac="), Frac);
		FParse::Value(Cmd, TEXT("BbAnimTilt="), Tilt);
		float Lift = 0.f;
		FParse::Value(Cmd, TEXT("BbAnimLift="), Lift);
		for (int32 I = 0; I < 8; ++I)
		{
			UAnimSequence* Clip = LoadObject<UAnimSequence>(nullptr, Clips[I]);
			ASkeletalMeshActor* F = Figures[I];
			F->SetActorHiddenInGame(false);
			// Facing sim +X (screen right from behind home), Unreal yaw 90.
			PlaceDiver(F, ToUE(-7.0 + 2.2 * I, 8.0, 0.0), 90.f, Tilt, Lift);
			if (Clip)
			{
				UE_LOG(LogBb, Display, TEXT("BB_ANIM %d %s length %.2f s, held at %.2f s"), I, Clips[I], Clip->GetPlayLength(), Frac * Clip->GetPlayLength());
				USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
				PlayClip(C, Clip, false);
				C->SetPlayRate(0.f);
				C->SetPosition(Frac * Clip->GetPlayLength(), false);
			}
		}
		Camera->SetActorLocation(ToUE(0, 1, 3.0));
		Camera->SetActorRotation((ToUE(0, 8, 0.3) - ToUE(0, 1, 3.0)).Rotation());
		Camera->GetCameraComponent()->SetFieldOfView(90.f);
		return;
	}
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
				PlayClip(F->GetSkeletalMeshComponent(), Clip, true);
				F->GetSkeletalMeshComponent()->SetPlayRate(0.25f);
			}
		}
		Camera->SetActorLocation(ToUE(0, -1, 2.0));
		Camera->SetActorRotation((ToUE(0, 12, 1.0) - ToUE(0, -1, 2.0)).Rotation());
		Camera->GetCameraComponent()->SetFieldOfView(62.f);
		return;
	}

	// The people who are not playing: umpires, base coaches, the benches (player-visuals §12).
	if (FigureShop && Skel.Ok() && !FParse::Param(Cmd, TEXT("BbNoCrew")))
	{
		Crew = NewObject<UBbCrew>(this);
		Crew->Init(this, Park, FigureShop, &Skel, FigureMesh, AnimIdle, *Started);
		UE_LOG(LogBb, Display, TEXT("BB_CREW %d figures besides the players"), Crew->Num());
	}
	if (Crew && FParse::Param(Cmd, TEXT("BbCrewShots")))
	{
		CrewShot = 0;
		return;
	}
	// Figure check (-BbFigureShots): both nines in a row, front and back, then one player
	// turned through eight angles, plain and in a mirror. No game is played.
	if (FParse::Param(Cmd, TEXT("BbFigureShots")) && Figures.Num() >= 9)
	{
		for (uint16 Side = 0; Side < 2; ++Side)
		{
			for (uint16 Order = 1; Order <= 9; ++Order)
			{
				const BbPlayer* P = Started->Player((uint16)(Side << 8 | Order));
				ShotPlayers.Add(P ? *P : BbPlayer{});
			}
		}
		FigureShot = 0;
		return;
	}
	// (The checks above return without a game: they need only the park.)
	Sim = MoveTemp(Started);
	// The first pitch, like every other, resolves on a worker; Tick starts it when it is ready.
	RequestNext();
}

float ABbDirector::SideCutFade()
{
	if (!bSideCutPlanned)
	{
		return 0.f;
	}
	const float BackAt = ResetDuration - SideCutTailS;
	if (!bSideCutJumped)
	{
		const float Out = FMath::Clamp((ResetTime - SideCutLeadS) / SideCutFadeS, 0.f, 1.f);
		if (Out < 1.f)
		{
			return Out;
		}
		// Black: skip the middle of the jog. The movers (on ResetTime) and the players going
		// off (on ExitClock) jump together, so everyone is where the full changeover would
		// have him SideCutHoldS before the picture returns.
		const float Skip = FMath::Max(0.f, BackAt - SideCutHoldS - ResetTime);
		ResetTime += Skip;
		ExitClock += Skip;
		bSideCutJumped = true;
		if (bStagingLog)
		{
			UE_LOG(LogBb, Display, TEXT("BB_SIDECUT play=%u skipped=%.2f of %.2f s back_at=%.2f"), NextPlay.Info.play_index, Skip, ResetDuration, BackAt);
		}
		return 1.f;
	}
	return 1.f - FMath::Clamp((ResetTime - BackAt) / SideCutFadeS, 0.f, 1.f);
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
	// The next pitch is ready: a skip asked for with Enter ends here.
	if (bRush && RushClock > 0.f)
	{
		UE_LOG(LogBb, Display, TEXT("BB_RUSH off at=%.2f s"), RushClock);
	}
	bRush = false;
	bReplay = false;
	const BbPlayInfo BoardBefore = Before;
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
		// Kept for the replay keys; the oldest goes when there are too many.
		History.Add({ Current, BoardBefore });
		if (History.Num() > HistoryMax)
		{
			Sim->Release(History[0].Play);
			History.RemoveAt(0);
		}
	}
	Current = MoveTemp(Play);
	// Each pitch opens with the wind-up: play time runs from −PreRoll to the release at 0.
	PlayTime = -PreRoll;
	MotionShotNext = 0;
	NotePitch();
	bBallInPlay = false;
	bBallDead = false;
	bDeadCut = false;
	bHomeRunCut = false;
	bFoulOut = false;
	CallsLogged = 0;
	if (!Current.bValid)
	{
		bGameOver = true;
		UE_LOG(LogBb, Display, TEXT("Game over after %d pitches"), PlaysShown);
		if (Sim)
		{
			// The final box: every pitch, read by the sim from its log. The worker has nothing
			// left to do, so nothing contends for the game.
			RefreshBox(0xFFFFFFFFu);
			bBoxFinal = bBoxValid;
			BoxFinalTime = 0.f;
		}
		return;
	}
	++PlaysShown;
	// Anyone still walking off who is in this pitch after all (only when no reset was staged).
	BbFrame Start;
	if (SampleDrawn(Current, 0.f, Start))
	{
		PurgeExits(Start);
	}
	// A ball in play gets a moment at the end; a taken pitch moves on quickly.
	HoldAfter = Current.Info.exit_velocity_mps > 0.f ? 2.5f : 0.7f;
	// A home run: the timeline has the batter and runners trot round to home after the ball
	// clears the fence (bb_game::timeline); the view holds a moment after the last one scores.
	bHomeRun = Current.Info.dead_reason == 2;
	if (bHomeRun)
	{
		HoldAfter = HrHoldS;
	}
	// A foul into the seats: a moment on the field, then straight to everyone set for the next
	// pitch, with no walk back (owner's play, seed 3647 play 62).
	bFoulOut = Current.Info.dead_reason == 4;
	if (bFoulOut)
	{
		HoldAfter = FoulOutHoldS;
	}
}

void ABbDirector::ShowReplay(int32 Index)
{
	ReplayIdx = FMath::Clamp(Index, 0, History.Num());
	const bool bLive = ReplayIdx == History.Num();
	Current = bLive ? LivePlay : History[ReplayIdx].Play;
	Before = bLive ? LiveBefore : History[ReplayIdx].Before;
	// The pitch from its wind-up, as BeginPlayback starts one.
	Throws.Reset();
	ThrowsSampledTo = -1.f;
	PlayTime = -PreRoll;
	NotePitch();
	bBallInPlay = false;
	bBallDead = false;
	bDeadCut = false;
	bHomeRunCut = false;
	CallsLogged = 0;
	bRush = false;
	BbFrame Start;
	if (SampleDrawn(Current, 0.f, Start))
	{
		PurgeExits(Start);
	}
	bHomeRun = Current.Info.dead_reason == 2;
	bFoulOut = Current.Info.dead_reason == 4;
	HoldAfter = bHomeRun ? HrHoldS : bFoulOut ? FoulOutHoldS : Current.Info.exit_velocity_mps > 0.f ? 2.5f : 0.7f;
	UE_LOG(LogBb, Display, TEXT("BB_REPLAY show play=%u live=%d kept=%d"), Current.Info.play_index, bLive ? 1 : 0, History.Num());
}

void ABbDirector::ReplayBack()
{
	if (!Current.bValid || bGameOver || !Sim)
	{
		return;
	}
	bool bSame = PlayTime > -PreRoll + ReplayBackS;
	if (!bReplay)
	{
		// Leaving the live game: the pitch on screen is kept aside, and a walk back into
		// place under way is dropped (the next pitch stays in NextPlay and is staged again
		// when the replay comes back to the end of this one).
		LivePlay = Current;
		LiveBefore = Before;
		ReplayIdx = History.Num();
		bReplay = true;
		if (bReset)
		{
			bSame = true;
			bReset = false;
			bSideCutPlanned = false;
			if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
			{
				if (PC->PlayerCameraManager)
				{
					PC->PlayerCameraManager->SetManualCameraFade(0.f, FLinearColor::Black, false);
				}
				if (ABbHUD* CardHud = Cast<ABbHUD>(PC->GetHUD()))
				{
					CardHud->CardAlpha = 0.f;
					CardHud->Card.Reset();
				}
			}
		}
	}
	ShowReplay(bSame ? ReplayIdx : ReplayIdx - 1);
}

void ABbDirector::ReplayForward()
{
	if (!bReplay)
	{
		return;
	}
	if (ReplayIdx >= History.Num())
	{
		// Already on the live pitch: carry on from here.
		bReplay = false;
		UE_LOG(LogBb, Display, TEXT("BB_REPLAY end play=%u"), Current.Info.play_index);
		return;
	}
	ShowReplay(ReplayIdx + 1);
}

void ABbDirector::SyncCursor()
{
	APlayerController* PC = GetWorld()->GetFirstPlayerController();
	if (!PC || !GEngine || !GEngine->GameViewport)
	{
		return;
	}
	if (!SaveButtonLayer.IsValid())
	{
		// The replay's button, top right; it shows only while a replay runs.
		TWeakObjectPtr<ABbDirector> Self(this);
		SaveButtonLayer = SNew(SOverlay)
			+ SOverlay::Slot().HAlign(HAlign_Right).VAlign(VAlign_Top).Padding(FMargin(0.f, 24.f, 28.f, 0.f))
			[
				SNew(SButton)
				.IsFocusable(false)
				.ContentPadding(FMargin(14.f, 5.f))
				.Visibility_Lambda([Self]() { return Self.IsValid() && Self->bReplay && !Self->bSaveOpen ? EVisibility::Visible : EVisibility::Collapsed; })
				.OnClicked_Lambda([Self]() { if (Self.IsValid()) { Self->OpenSaveDialog(); } return FReply::Handled(); })
				[
					SNew(STextBlock).Text(FText::FromString(TEXT("Save this play  (S)"))).Font(FCoreStyle::GetDefaultFontStyle("Bold", 14))
				]
			];
		GEngine->GameViewport->AddViewportWidgetContent(SaveButtonLayer.ToSharedRef(), 50);
	}
	const int32 Want = bSaveOpen ? 2 : bReplay ? 1 : 0;
	if (Want == InputModeSet)
	{
		return;
	}
	InputModeSet = Want;
	PC->bShowMouseCursor = Want != 0;
	if (Want == 2 && SaveDialog.IsValid())
	{
		FInputModeUIOnly Mode;
		Mode.SetWidgetToFocus(SaveDialog->Box);
		Mode.SetLockMouseToViewportBehavior(EMouseLockMode::DoNotLock);
		PC->SetInputMode(Mode);
		FSlateApplication::Get().SetKeyboardFocus(SaveDialog->Box);
	}
	else if (Want == 1)
	{
		FInputModeGameAndUI Mode;
		Mode.SetHideCursorDuringCapture(false);
		Mode.SetLockMouseToViewportBehavior(EMouseLockMode::DoNotLock);
		PC->SetInputMode(Mode);
	}
	else
	{
		PC->SetInputMode(FInputModeGameOnly());
	}
}

void ABbDirector::OpenSaveDialog()
{
	if (bSaveOpen || !Current.bValid || !GEngine || !GEngine->GameViewport)
	{
		return;
	}
	bSaveOpen = true;
	SaveClose = 0;
	bPausedBeforeSave = bPaused;
	bPaused = true;
	TWeakObjectPtr<ABbDirector> Self(this);
	const FText Title = FText::FromString(FString::Printf(TEXT("[SEED %llu    PLAY %u]"), SeedShown, Current.Info.play_index));
	// A floating window, not over the middle of the field: it opens at the top right, under
	// the replay's button, where it was last left, and is dragged by any part that is not a
	// button or the text box. The field is not dimmed.
	SaveDialogLayer = SNew(SOverlay)
		+ SOverlay::Slot().HAlign(HAlign_Right).VAlign(VAlign_Top)
		.Padding(TAttribute<FMargin>::CreateLambda([Self]() { return Self.IsValid() && Self->SaveDialog.IsValid() ? Self->SaveDialog->Margin() : FMargin(0.f); }))
		[
			SAssignNew(SaveDialog, SBbSavePlayDialog)
			.Title(Title)
			.Offset(SaveDialogOffset)
			.OnCancel_Lambda([Self]() { if (Self.IsValid()) { Self->SaveClose = 1; } })
			.OnSave_Lambda([Self](const FString& Comment) { if (Self.IsValid()) { Self->SaveComment = Comment; Self->SaveClose = 2; } })
		];
	GEngine->GameViewport->AddViewportWidgetContent(SaveDialogLayer.ToSharedRef(), 100);
	SyncCursor();
	UE_LOG(LogBb, Display, TEXT("BB_SAVE open seed=%llu play=%u"), SeedShown, Current.Info.play_index);
}

void ABbDirector::CloseSaveDialog(bool bSave)
{
	if (!bSaveOpen)
	{
		return;
	}
	if (bSave)
	{
		BbSavePlay::FWhat What;
		What.Seed = SeedShown;
		What.Play = Current.Info.play_index;
		What.Inning = Before.inning;
		What.bTop = Before.top != 0;
		What.Outs = Before.outs;
		What.Balls = Before.balls;
		What.Strikes = Before.strikes;
		What.AwayRuns = Before.away_runs;
		What.HomeRuns = Before.home_runs;
		const FString Path = BbSavePlay::Write(What, SaveComment);
		SavedNote = Path.IsEmpty() ? FString(TEXT("Could not save the play")) : FString::Printf(TEXT("Saved: saved-plays/%s"), *FPaths::GetCleanFilename(Path));
		SavedUntil = Wall + 4.0;
		UE_LOG(LogBb, Display, TEXT("BB_SAVE %s seed=%llu play=%u path=%s"), Path.IsEmpty() ? TEXT("failed") : TEXT("saved"), SeedShown, What.Play, *Path);
	}
	else
	{
		UE_LOG(LogBb, Display, TEXT("BB_SAVE cancel"));
	}
	if (SaveDialog.IsValid())
	{
		SaveDialogOffset = SaveDialog->Offset;
	}
	if (GEngine && GEngine->GameViewport && SaveDialogLayer.IsValid())
	{
		GEngine->GameViewport->RemoveViewportWidgetContent(SaveDialogLayer.ToSharedRef());
	}
	SaveDialogLayer.Reset();
	SaveDialog.Reset();
	SaveComment.Reset();
	SaveClose = 0;
	bSaveOpen = false;
	bPaused = bPausedBeforeSave;
	SyncCursor();
}

FVector ABbDirector::BatterFrame(const float P[3], const FBatNote& N)
{
	// He is shown facing the pitcher with the plate on his right; a left-handed batter (the
	// first-base side's box, sim x > 0) is that in a mirror.
	return FVector((N.bLeftBox ? 1.0 : -1.0) * (P[0] - N.Spot.X) * 100.0, (P[1] - N.Spot.Y) * 100.0, P[2] * 100.0);
}

void ABbDirector::NoteBat(const FBbPlay& Play, FBatNote& Out) const
{
	Out = FBatNote();
	BbFrame F0;
	if (!Sim || !Play.bValid || !Sim->Sample(Play, 0.f, F0) || F0.has_bat == 0)
	{
		return;
	}
	// The batter: whoever of the side at bat stands by the bat.
	double Best = 1.6 * 1.6;
	for (uint32 I = 0; I < F0.n_actors; ++I)
	{
		const BbActor& A = F0.actors[I];
		const double D = FMath::Square(A.pos[0] - F0.bat_knob[0]) + FMath::Square(A.pos[1] - F0.bat_knob[1]);
		if ((A.agent & 0x80) == 0 && D < Best)
		{
			Best = D;
			Out.bBat = true;
			Out.Spot = FVector2D(A.pos[0], A.pos[1]);
		}
	}
	if (!Out.bBat)
	{
		return;
	}
	Out.bLeftBox = Out.Spot.X > 0.0;
	Out.StanceKnob = Out.EndKnob = ToUE(F0.bat_knob);
	Out.StanceTip = Out.EndTip = ToUE(F0.bat_tip);
	// The swing, if there is one: the sim's bat track every 5 ms. It ends when the bat has
	// stopped moving (the sim holds its last pose a moment, then has no bat at all).
	const float Until = FMath::Min(Play.Info.duration_s, 1.5f);
	float LeastDeg = 1e9f, LastMove = 0.f;
	BbFrame Prev = F0;
	for (float T = 0.005f; T <= Until; T += 0.005f)
	{
		BbFrame F;
		if (!Sim->Sample(Play, T, F) || F.has_bat == 0)
		{
			break;
		}
		const bool bMoved = FMemory::Memcmp(F.bat_knob, Prev.bat_knob, sizeof(F.bat_knob)) != 0 || FMemory::Memcmp(F.bat_tip, Prev.bat_tip, sizeof(F.bat_tip)) != 0;
		if (bMoved && !Out.bSwing)
		{
			Out.bSwing = true;
			Out.StartT = T - 0.005f;
		}
		if (bMoved)
		{
			LastMove = T;
			Out.EndKnob = ToUE(F.bat_knob);
			Out.EndTip = ToUE(F.bat_tip);
		}
		if (Out.bSwing)
		{
			const float Deg = (float)BbMotion::HandsDeg(BatterFrame(F.bat_knob, Out), BatterFrame(F.bat_tip, Out));
			if (Deg < LeastDeg)
			{
				LeastDeg = Deg;
				Out.LaunchT = T;
			}
		}
		Prev = F;
	}
	if (Out.bSwing)
	{
		Out.LaunchDeg = LeastDeg;
		Out.EndT = LastMove;
	}
}

void ABbDirector::NotePitch()
{
	PitchEndT = -1.f;
	CrouchUntil = 0.6f;
	NoteBat(Current, BatNote);
	MotionStrip = -1;
	if (bMotionShots && Current.bValid)
	{
		// The first three pitches for the pitcher and the catcher; then the first ball hit,
		// the first swing and miss, and the first pitch taken, for the batter.
		const int32 Index = (int32)Current.Info.play_index;
		const bool bHit = Current.Info.contact_s >= 0.f;
		if (Index >= 1 && Index <= 3)
		{
			MotionStrip = Index - 1;
		}
		else if (Index > 3 && BatNote.bSwing && bHit && !bStripDone[3])
		{
			MotionStrip = 3;
		}
		else if (Index > 3 && BatNote.bSwing && !bHit && !bStripDone[4])
		{
			MotionStrip = 4;
		}
		else if (Index > 3 && BatNote.bBat && !BatNote.bSwing && !bStripDone[5])
		{
			MotionStrip = 5;
		}
	}
	NoteCatches();
	NoteSignals();
	NoteSlides();
	if (Crew)
	{
		Crew->SetSignals(Signals);
	}
	if (bStagingLog)
	{
		static const TCHAR* const Kinds[5] = { TEXT("none"), TEXT("strike"), TEXT("out"), TEXT("safe"), TEXT("foul") };
		for (const FBbSignal& S : Signals)
		{
			UE_LOG(LogBb, Display, TEXT("BB_SIGNAL play=%u t=%.2f of %.2f umpire=%d %s at=(%.1f, %.1f) \"%s\""), Current.Info.play_index, S.T, Current.Info.duration_s, S.Umpire, Kinds[(int32)S.Kind], S.At.X, S.At.Y, UTF8_TO_TCHAR(Current.Info.summary));
		}
	}
	ReadyUntil = Current.bValid && Current.Info.contact_s >= 0.f ? Current.Info.contact_s + 0.3f : 0.8f;
	BbFrame Start;
	if (!Sim || !Current.bValid || !SampleDrawn(Current, 0.f, Start))
	{
		return;
	}
	PitchFrom = ToUE(Start.ball);
	const float Contact = Current.Info.contact_s;
	if (Contact >= 0.f)
	{
		// Hit: the mitt goes to where the pitch was heading, and he is up at once.
		CrouchUntil = Contact + 0.12f;
		const float Ta = FMath::Max(Contact - 0.03f, 0.f), Tb = FMath::Max(Contact - 0.004f, 0.f);
		BbFrame A, B;
		if (Tb > Ta && SampleDrawn(Current, Ta, A) && SampleDrawn(Current, Tb, B) && B.ball[1] < A.ball[1] - 1e-3f)
		{
			const float K = (MittPlaneY - B.ball[1]) / (B.ball[1] - A.ball[1]);
			PitchEndAt = ToUE(B.ball[0] + K * (B.ball[0] - A.ball[0]), MittPlaneY, FMath::Max(0.05f, B.ball[2] + K * (B.ball[2] - A.ball[2])));
			PitchEndT = Tb + K * (Tb - Ta);
		}
		return;
	}
	// Taken or missed: the ball stops in the mitt. When it first gets there:
	BbFrame Last;
	if (!SampleDrawn(Current, Current.Info.duration_s, Last) || FMath::Abs(Last.ball[1] - MittPlaneY) > 0.4f)
	{
		return;
	}
	const FVector End = ToUE(Last.ball);
	for (float T = 0.2f; T <= Current.Info.duration_s + 0.005f; T += 0.005f)
	{
		BbFrame F;
		if (SampleDrawn(Current, T, F) && (ToUE(F.ball) - End).SizeSquared() < 9.0)
		{
			PitchEndT = T;
			PitchEndAt = End;
			CrouchUntil = T + ReceiveHoldS;
			break;
		}
	}
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
		if (SampleDrawn(Current, At, F) && F.holder >= 0 && F.release_in_s >= 0.f)
		{
			AddThrow((uint16)F.holder, At + F.release_in_s);
		}
		if (At >= To)
		{
			break;
		}
	}
}

void ABbDirector::AddThrow(uint16 Agent, float ReleaseAt)
{
	if (Throws.ContainsByPredicate([&](const FThrowNote& N) { return N.Agent == Agent && FMath::Abs(N.ReleaseAt - ReleaseAt) < 0.05f; }))
	{
		return;
	}
	// He turns toward where the ball goes: from his feet at the release to the ball a moment
	// into its flight.
	FThrowNote N;
	N.Agent = Agent;
	N.ReleaseAt = ReleaseAt;
	BbFrame AtRelease, InFlight, Gone;
	FVector Foot = FVector::ZeroVector;
	if (SampleDrawn(Current, ReleaseAt, AtRelease))
	{
		for (uint32 I = 0; I < AtRelease.n_actors; ++I)
		{
			if (AtRelease.actors[I].agent == Agent)
			{
				Foot = ToUE(AtRelease.actors[I].pos);
			}
		}
	}
	N.Yaw = SampleDrawn(Current, ReleaseAt + 0.35f, InFlight) ? (ToUE(InFlight.ball) - Foot).Rotation().Yaw : YawToPlate(Foot);
	// Where the sim lets it go: the ball as its flight starts (two moments on, carried back).
	BbFrame Later;
	if (SampleDrawn(Current, ReleaseAt + 0.004f, Gone) && SampleDrawn(Current, ReleaseAt + 0.008f, Later) && Gone.holder < 0 && Later.holder < 0)
	{
		const FVector A = ToUE(Gone.ball), B = ToUE(Later.ball);
		N.From = A - (B - A);
		N.bFrom = (N.From - Foot).Size() < 350.0;
	}
	Throws.Add(N);
}

void ABbDirector::NoteCatches()
{
	// The whole pitch, a fiftieth of a second at a time: each moment the ball passes into a
	// fielder's hands (found to the millisecond), and each throw he then has to make.
	Catches.Reset();
	StripCatch = -1;
	StripThrow = -1;
	if (!Sim || !Current.bValid)
	{
		return;
	}
	const float End = Current.Info.duration_s;
	int32 Prev = -1, LastHolder = -1;
	float PrevT = 0.f;
	for (float T = 0.f;; T += 0.02f)
	{
		const float At = FMath::Min(T, End);
		BbFrame F;
		if (!SampleDrawn(Current, At, F))
		{
			break;
		}
		if (F.holder >= 0 && F.holder != Prev)
		{
			float Lo = PrevT, Hi = At;
			BbFrame M, Free = F;
			bool bFree = false;
			for (int32 K = 0; K < 6 && Hi - Lo > 0.0008f; ++K)
			{
				const float Mid = 0.5f * (Lo + Hi);
				if (SampleDrawn(Current, Mid, M) && M.holder == F.holder)
				{
					Hi = Mid;
				}
				else
				{
					Lo = Mid;
					Free = M;
					bFree = true;
				}
			}
			if (!bFree)
			{
				bFree = At > 0.f && SampleDrawn(Current, Lo, Free) && Free.holder != F.holder;
			}
			// Where the ball is as he takes it: the timeline has the ball at each step of the
			// sim, and in the step that ends with the catch it jumps to his holding point. So
			// the two steps before, carried on one more.
			BbFrame One, Two;
			if (bFree && Free.holder < 0 && Lo > 2.f * SimTickS && SampleDrawn(Current, Lo - SimTickS, One) && SampleDrawn(Current, Lo - 2.f * SimTickS, Two) && One.holder < 0 && Two.holder < 0)
			{
				FCatchNote N;
				N.Agent = (uint16)F.holder;
				N.T = Hi;
				N.Prev = ToUE(One.ball);
				N.At = N.Prev + (N.Prev - ToUE(Two.ball));
				N.At.Z = FMath::Max(N.At.Z, 4.0);
				N.bThrown = LastHolder >= 0;
				Catches.Add(N);
			}
			if (F.release_in_s >= 0.f)
			{
				AddThrow((uint16)F.holder, At + F.release_in_s);
			}
		}
		if (F.holder >= 0)
		{
			LastHolder = F.holder;
		}
		Prev = F.holder;
		PrevT = At;
		if (At >= End)
		{
			break;
		}
	}
	if (bMotionShots && MotionStrip < 0 && Current.Info.play_index > 3)
	{
		// The strips of the field: a batted ball taken and thrown by the same man (not the
		// pitcher or the catcher); then, another pitch, a throw taken.
		for (int32 C = 0; C < Catches.Num() && MotionStrip < 0; ++C)
		{
			const FCatchNote& N = Catches[C];
			if ((N.Agent & 0x7F) < 3)
			{
				continue;
			}
			if (!N.bThrown && !bStripDone[6])
			{
				for (int32 W = 0; W < Throws.Num(); ++W)
				{
					if (Throws[W].Agent == N.Agent && Throws[W].bFrom && Throws[W].ReleaseAt > N.T && Throws[W].ReleaseAt < N.T + 3.f)
					{
						MotionStrip = 6;
						StripCatch = C;
						StripThrow = W;
						break;
					}
				}
			}
			else if (N.bThrown && bStripDone[6] && !bStripDone[7])
			{
				MotionStrip = 7;
				StripCatch = C;
			}
		}
	}
}

FVector2D ABbDirector::Dugout(uint16 Agent) const
{
	// The dugouts are the sim's openings in the stands' wall (physics-spec §9.5): home on the
	// first-base side, away on the third-base side. A player walking off goes in at the middle
	// of the opening and disappears under the roof.
	return Park->DugoutDoor((Agent >> 8) != 0 ? 1.0 : -1.0);
}

bool ABbDirector::SampleDrawn(const FBbPlay& Play, float T, BbFrame& Out) const
{
	if (!Sim->Sample(Play, T, Out))
	{
		return false;
	}
	// The sim's ground is level; the drawn field has the mound's height (park-visuals §2).
	// The ball is never drawn under the drawn ground: a ball in the air is where the sim has
	// it. (Players are stood on the drawn ground where they are posed, in Tick.)
	const float Under = (float)Park->GroundZ(Out.ball[0], Out.ball[1]);
	if (Under > 0.f)
	{
		Out.ball[2] = FMath::Max(Out.ball[2], Under + 0.037f);
	}
	return true;
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
			St.Motion = Old[J].Motion;
			St.MotionW = Old[J].MotionW;
			St.MotionT = Old[J].MotionT;
			St.Off = Old[J].Off;
		}
		return;
	}
}

bool ABbDirector::StartReset(const FBbPlay& Next)
{
	BbFrame From, To;
	if (!SampleDrawn(Current, Current.Info.duration_s, From) || !SampleDrawn(Next, 0.f, To) || To.n_actors == 0)
	{
		return false;
	}
	NoteBat(Next, NextBatNote);
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
		// At a change of sides the fielders taking the field jog all the way from their dugout.
		const bool bTakesField = bSideChange && (A.agent & 0x80) != 0;
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
			// A fielder coming on mid-inning with far to go is picked up already on his way
			// out; at a change of sides he comes out of the dugout.
			M.From = (A.agent & 0x80) != 0 ? Dugout(A.agent) : OnDeck(A.agent);
			const double Dist = (M.To - M.From).Size();
			if (Dist > ResetEnterMaxM && !bTakesField)
			{
				M.From = M.To + (M.From - M.To) * (ResetEnterMaxM / Dist);
			}
		}
		const float Dist = (M.To - M.From).Size();
		M.SpeedMps = Dist < ResetWalkWithinM ? ResetWalkMps : ResetJogMps;
		if (bTakesField)
		{
			M.SpeedMps = SideChangeJogMps;
		}
		// He quickens rather than keep the next pitch waiting, but only up to a jog: a
		// sprint (or faster than legs go) back to his spot looks wrong.
		else if (TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2) > ResetMaxSeconds)
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
	// next pitch if they need to. At a change of sides they all jog off.
	int32 Leaving = 0;
	for (uint32 I = 0; I < From.n_actors; ++I)
	{
		const BbActor& A = From.actors[I];
		if (!Find(To, A.agent))
		{
			AddExit(Old, A.agent, FVector2D(A.pos[0], A.pos[1]), bSideChange);
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
	ResetBallTo = Pitcher ? FVector(Pitcher->pos[0], Pitcher->pos[1], 1.4) : FVector(0.0, Sim->Park().rubber_m, 1.4);
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
			// He lets it go as the sim's fielders do: 1.8 m up, off to his throwing side.
			const FVector2D Line = (FVector2D(ResetBallTo.X, ResetBallTo.Y) - M.From).GetSafeNormal();
			const double ArmSide = Sim->ThrowsLeft(M.Agent) ? -0.35 : 0.35;
			ResetHand = FVector(M.From.X + Line.Y * ArmSide, M.From.Y - Line.X * ArmSide, 1.8);
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
	// Worth cutting only if there is a middle to skip.
	bSideCutPlanned = bSideCut && bSideChange && ResetDuration > SideCutLeadS + 2.f * SideCutFadeS + SideCutHoldS + SideCutTailS + 2.f;
	bSideCutJumped = false;
	if (bStagingLog)
	{
		UE_LOG(LogBb, Display, TEXT("BB_RESET play=%u side_change=%d staying=%d leaving=%d duration=%.2f throw_at=%.2f"),
			Next.Info.play_index, bSideChange ? 1 : 0, Movers.Num(), Leaving, ResetDuration, ResetThrower >= 0 ? ResetThrowAt : -1.f);
	}
	return true;
}

float ABbDirector::AddExit(const TArray<FFigureState>& Old, uint16 Agent, const FVector2D& From, bool bJog)
{
	FResetMover M;
	M.Agent = Agent;
	M.From = From;
	M.To = Dugout(Agent);
	const float Dist = (M.To - M.From).Size();
	M.SpeedMps = Dist < ResetWalkWithinM ? ResetWalkMps : ResetJogMps;
	if (bJog)
	{
		M.SpeedMps = SideChangeJogMps;
	}
	else if (TravelSeconds(Dist, M.SpeedMps, ResetAccelMps2) > ResetMaxSeconds)
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
		// The thrower stands still until the ball is gone, and lets it go beside his head.
		const FVector Hand = ResetHand;
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

void ABbDirector::NoteSignals()
{
	// What the umpires signal on this pitch, from the sim's own record of it: a strike (the
	// plate umpire), a foul ball (whoever's line it is), each out (the umpire at that base; a
	// caught fly, the umpire nearest it). "Safe" is added by NoteSlides.
	Signals.Reset();
	if (!Sim || !Current.bValid)
	{
		return;
	}
	using BbMotion::ESignal;
	const BbPlayInfo& Info = Current.Info;
	const FString Words(UTF8_TO_TCHAR(Info.summary));
	const double L = 27.432 / UE_SQRT_2;
	const FVector2D Bags[5] = { FVector2D(0.0, 0.0), FVector2D(L, L), FVector2D(0.0, 2.0 * L), FVector2D(-L, L), FVector2D(0.0, 0.0) };
	auto Near = [](const FVector2D& P, bool bLinesOnly)
	{
		const double R = P.Size();
		return R < 15.0 || P.Y < 0.0 ? 0 : P.X > 0.35 * R ? 1 : P.X < -0.35 * R ? 3 : bLinesOnly ? 0 : 2;
	};
	auto BallAt = [&](float T)
	{
		BbFrame F;
		return SampleDrawn(Current, FMath::Clamp(T, 0.f, Info.duration_s), F) ? FVector(F.ball[0], F.ball[1], F.ball[2]) : FVector::ZeroVector;
	};
	const bool bContact = Info.contact_s >= 0.f;
	if (Words.Contains(TEXT("trike")))
	{
		Signals.Add(FBbSignal{ bContact ? Info.contact_s + 0.5f : Info.call_s, 0, ESignal::Strike, FVector2D(0.0, 18.0) });
	}
	else if (bContact && Words.EndsWith(TEXT("Foul")))
	{
		// Called when it comes down (or leaves the field).
		float T = Info.dead_s >= 0.f ? Info.dead_s : Info.duration_s;
		for (float At = Info.contact_s + 0.1f; At < T; At += 0.05f)
		{
			if (BallAt(At).Z < 0.12)
			{
				T = At;
				break;
			}
		}
		const FVector B = BallAt(T);
		Signals.Add(FBbSignal{ FMath::Min(T + 0.15f, Info.duration_s), Near(FVector2D(B.X, B.Y), true), ESignal::Foul, FVector2D(B.X, B.Y) });
	}
	for (uint32 C = 0; C < Info.n_calls; ++C)
	{
		const BbCall& Call = Info.calls[C];
		if (Call.base == 0)
		{
			// A strikeout is the strike's signal; a caught fly is called by whoever is nearest.
			if (bContact)
			{
				const FVector B = BallAt(Call.t_s);
				Signals.Add(FBbSignal{ Call.t_s, Near(FVector2D(B.X, B.Y), false), ESignal::Out, FVector2D(B.X, B.Y) });
			}
		}
		else if (Call.base <= 4)
		{
			Signals.Add(FBbSignal{ Call.t_s, Call.base == 4 ? 0 : (int32)Call.base, ESignal::Out, Bags[Call.base] });
		}
	}
}

void ABbDirector::ArrangeCrewShot(int32 Step)
{
	// No game: the crew alone, told what to do. The plate umpire set and calling a strike; the
	// first-base umpire calling out, safe and foul, and set; the coach at first; the benches;
	// everyone from behind home.
	using BbMotion::ESignal;
	FBbCrewTick T;
	T.Batting = 0;
	T.BatterX = -0.8;
	T.bSet = Step == 0 || Step == 5 || Step == 6;
	T.Force = Step == 1 ? ESignal::Strike : Step == 2 ? ESignal::Out : Step == 3 ? ESignal::Safe : Step == 4 ? ESignal::Foul : ESignal::None;
	CrewShotTick = T;
	FVector Eye, Look;
	float Fov = 40.f;
	if (Step <= 1)
	{
		const FVector2D U = Crew->UmpireAt(0, T);
		Eye = ToUE(U.X + 4.6, U.Y + 3.6, 1.5);
		Look = ToUE(U.X, U.Y, 1.05);
	}
	else if (Step <= 5)
	{
		const FVector2D U = Crew->UmpireAt(1, T);
		Eye = ToUE(U.X * 0.78, U.Y * 0.78 - 1.5, 1.4);
		Look = ToUE(U.X, U.Y, 1.0);
	}
	else if (Step == 6)
	{
		const FVector2D U = UBbCrew::CoachAt(1.0);
		Eye = ToUE(U.X * 0.72, U.Y * 0.72, 1.4);
		Look = ToUE(U.X, U.Y, 1.0);
	}
	else if (Step <= 8)
	{
		FVector2D From, Along, Face;
		double Length = 0.0, SeatZ = 0.0, FloorZ = 0.0;
		Park->Bench(Step == 7 ? 1.0 : -1.0, From, Along, Face, Length, SeatZ, FloorZ);
		const FVector2D Mid = From + Along * (Length * 0.5), At = Mid + Face * 10.0;
		Eye = ToUE(At.X, At.Y, 1.7);
		Look = ToUE(Mid.X, Mid.Y, 0.0);
		Fov = 70.f;
	}
	else
	{
		Eye = ToUE(0.0, -16.0, 9.0);
		Look = ToUE(0.0, 16.0, 0.0);
		Fov = 75.f;
	}
	Camera->SetActorLocationAndRotation(Eye, (Look - Eye).Rotation());
	Camera->GetCameraComponent()->SetFieldOfView(Fov);
	ShownCam = -1;
}

void ABbDirector::NoteSlides()
{
	// The whole pitch again (after NoteCatches: the ball's takers are known). The timeline has
	// everyone's place every tenth of a second, so a runner who runs into a base and stops
	// shows as full speed up to one of those moments and still from it.
	Slides.Reset();
	ExtraShots.Reset();
	ExtraShotNext = 0;
	ShotDiver = -1;
	if (!Sim || !Current.bValid || Current.Info.contact_s < 0.f)
	{
		return;
	}
	const double L = 27.432 / UE_SQRT_2;
	const FVector2D Bags[3] = { FVector2D(0.0, 2.0 * L), FVector2D(-L, L), FVector2D(0.0, 0.0) };
	const float End = Current.Info.duration_s, Step = 0.02f;
	TArray<BbFrame> Seen;
	for (float T = 0.f; T < End + Step; T += Step)
	{
		BbFrame F;
		if (!SampleDrawn(Current, FMath::Min(T, End), F))
		{
			break;
		}
		Seen.Add(F);
	}
	auto Find = [&](int32 K, uint16 Agent, FVector2D& Out)
	{
		const BbFrame& F = Seen[FMath::Clamp(K, 0, Seen.Num() - 1)];
		for (uint32 I = 0; I < F.n_actors; ++I)
		{
			if (F.actors[I].agent == Agent)
			{
				Out = FVector2D(F.actors[I].pos[0], F.actors[I].pos[1]);
				return true;
			}
		}
		return false;
	};
	for (int32 K = 15; K < Seen.Num(); ++K)
	{
		const BbFrame& F = Seen[K];
		for (uint32 I = 0; I < F.n_actors; ++I)
		{
			const uint16 Agent = F.actors[I].agent;
			if ((Agent & 0x80) != 0)
			{
				continue;
			}
			const FVector2D P(F.actors[I].pos[0], F.actors[I].pos[1]);
			for (int32 B = 0; B < 3; ++B)
			{
				FVector2D Was, Prior, Earlier, Later;
				// Just there, having come at a run, and stopping (not rounding it).
				if ((P - Bags[B]).Size() > 0.04 || !Find(K - 1, Agent, Was) || (Was - Bags[B]).Size() <= 0.04
					|| !Find(K - 5, Agent, Prior) || !Find(K - 15, Agent, Earlier)
					|| (Find(K + 10, Agent, Later) && (Later - Bags[B]).Size() > 0.1))
				{
					continue;
				}
				const float T = K * Step;
				const double Mps = (Prior - Earlier).Size() / (10 * Step);
				// Is a play being made on him there?
				bool bPlay = false;
				for (const FCatchNote& N : Catches)
				{
					bPlay |= FMath::Abs(N.T - T) < SlidePlayS && (FVector2D(N.At.Y, N.At.X) / 100.0 - Bags[B]).Size() < SlidePlayM;
				}
				for (uint32 C = 0; C < Current.Info.n_calls; ++C)
				{
					bPlay |= Current.Info.calls[C].runner == Agent && Current.Info.calls[C].base == B + 2;
				}
				FVector2D Holder;
				bPlay |= F.holder >= 0 && Find(K, (uint16)F.holder, Holder) && (Holder - Bags[B]).Size() < SlidePlayM;
				const bool bSlide = bPlay && Mps >= SlideMinMps;
				// A play on him and no out: the umpire there has him safe.
				bool bOutThere = false;
				for (uint32 C = 0; C < Current.Info.n_calls; ++C)
				{
					bOutThere |= Current.Info.calls[C].runner == Agent && Current.Info.calls[C].base == B + 2;
				}
				if (bPlay && !bOutThere)
				{
					Signals.Add(FBbSignal{ T + SafeCallS, B == 0 ? 2 : B == 1 ? 3 : 0, BbMotion::ESignal::Safe, Bags[B] });
				}
				if (bStagingLog)
				{
					UE_LOG(LogBb, Display, TEXT("BB_SLIDE play=%u agent=0x%03x base=%d t=%.2f of %.2f s in=%.1f m/s play_on_him=%d slides=%d"), Current.Info.play_index, Agent, B + 2, T, End, Mps, bPlay ? 1 : 0, bSlide ? 1 : 0);
				}
				if (bSlide)
				{
					FSlideNote N;
					N.Agent = Agent;
					N.T = T;
					N.Base = B + 2;
					N.Yaw = ToUE(Prior.X - Earlier.X, Prior.Y - Earlier.Y, 0.0).Rotation().Yaw;
					N.At = ToUE(Bags[B].X, Bags[B].Y, 0.0);
					Slides.Add(N);
				}
			}
		}
	}
	// The batter at first: a throw taken at the bag about when he gets there, and no out.
	{
		const FVector2D First(L, L);
		const uint16 Batter = Current.Info.batter;
		bool bOutThere = false;
		for (uint32 C = 0; C < Current.Info.n_calls; ++C)
		{
			bOutThere |= Current.Info.calls[C].base == 1;
		}
		FVector2D P;
		for (int32 K = 0; !bOutThere && K < Seen.Num(); ++K)
		{
			if (Find(K, Batter, P) && (P - First).Size() < 0.5)
			{
				const float T = K * Step;
				bool bPlay = false;
				for (const FCatchNote& N : Catches)
				{
					bPlay |= FMath::Abs(N.T - T) < SafePlayS && (FVector2D(N.At.Y, N.At.X) / 100.0 - First).Size() < 3.0;
				}
				if (bPlay)
				{
					Signals.Add(FBbSignal{ T + SafeCallS, 1, BbMotion::ESignal::Safe, First });
				}
				break;
			}
		}
	}
	// What -BbSlideShots and -BbDiveShots photograph of this pitch: its first slide going
	// down, sliding, at the bag, sitting up and getting up; its first dive driving off, in the
	// air, down, on hands and knees and getting up.
	const bool bWantSlides = SlideShotsLeft > 0, bWantDives = DiveShotsLeft > 0;
	if (bWantSlides && Slides.Num() > 0)
	{
		const float Marks[5] = { -0.3f, -0.15f, 0.f, 0.4f, 0.8f };
		for (int32 M = 0; M < 5; ++M)
		{
			ExtraShots.Emplace(Slides[0].T + Marks[M], FString::Printf(TEXT("slide_%02d_%d.png"), SlideShotIndex, M + 1));
		}
		ShotSlide = 0;
		++SlideShotIndex;
		--SlideShotsLeft;
	}
	else
	{
		ShotSlide = -1;
	}
	for (int32 K = 0; bWantDives && K < Seen.Num() && ShotDiver < 0; ++K)
	{
		if (Seen[K].n_dives > 0)
		{
			const BbDive& Dv = Seen[K].dives[0];
			const float T0 = K * Step - Dv.since_s, Up = T0 + Dv.since_s + Dv.up_in_s;
			const float Marks[5] = { T0 - 0.13f, T0, T0 + 0.25f, Up - 0.6f, Up - 0.3f };
			for (int32 M = 0; M < 5; ++M)
			{
				ExtraShots.Emplace(Marks[M], FString::Printf(TEXT("dive_%02d_%d.png"), DiveShotIndex, M + 1));
			}
			ShotDiver = Dv.agent;
			ShotDiveFrom = T0;
			ShotDiveYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
			++DiveShotIndex;
			--DiveShotsLeft;
		}
	}
	ExtraShots.Sort([](const TPair<float, FString>& A, const TPair<float, FString>& B) { return A.Key < B.Key; });
}

void ABbDirector::PlaceDiver(ASkeletalMeshActor* F, const FVector& Foot, float Yaw, float TiltDeg, float LiftCm) const
{
	// The mannequin's mesh faces +Y in its own frame. A tilt tips him forward about the axis
	// across his body, pivoting at the feet; tipped, his middle (not his feet) is kept over
	// the sim's position. LiftCm raises (or, negative, lowers) him.
	const FQuat Facing(FRotator(0.f, Yaw - 90.f, 0.f));
	const float S = FMath::Sin(FMath::DegreesToRadians(FMath::Abs(TiltDeg)));
	const FQuat Tip(FVector(1.f, 0.f, 0.f), FMath::DegreesToRadians(DiveTiltSign * TiltDeg));
	const FVector Ahead = FRotator(0.f, Yaw, 0.f).Vector();
	F->SetActorLocationAndRotation(Foot - Ahead * (DiveBodyBackCm * S) + FVector(0, 0, LiftCm), Facing * Tip);
}

void ABbDirector::PlaceBat(const FVector& Knob, const FVector& Tip)
{
	// Either shape's long axis is its local Z: the cylinder's middle is its origin, the
	// turned bat's knob is.
	Bat->SetActorRotation(FRotationMatrix::MakeFromZ(Tip - Knob).Rotator());
	if (bTurnedBat)
	{
		Bat->SetActorLocation(Knob);
		Bat->SetActorScale3D(FVector(BbLook::BatThick, BbLook::BatThick, (Tip - Knob).Size() / UBbFigureShop::BatCm));
	}
	else
	{
		Bat->SetActorLocation((Knob + Tip) * 0.5);
	}
}

void ABbDirector::ArrangeFigureShot(int32 Step)
{
	// A row along the line from second base to centre field, on the outfield grass. The sun
	// is over the third-base side, so the row faces that way to be photographed from the
	// front and turns its back to the same camera for the numbers.
	// The last picture is the turn again with every figure shown in a mirror, as a
	// left-hander's motions are: the lettering must still read.
	// Then the kit (step 2): the home nine as they take the field, from the front and (every
	// one in a mirror, which must leave each glove on the same hand) from behind; and eight
	// visitors in batting helmets turned through eight angles, a bat stood beside the first.
	const bool bKitField = Step == 6 || Step == 7, bKitBat = Step == 8;
	const bool bTurn = Step == 4 || Step == 5 || bKitBat;
	const int32 Count = bTurn ? 8 : 9;
	const double Gap = bTurn ? 1.05 : 0.95, Y0 = 50.0 - Gap * (Count - 1) * 0.5;
	for (int32 I = 0; I < Figures.Num(); ++I)
	{
		ASkeletalMeshActor* F = Figures[I];
		F->SetActorHiddenInGame(I >= Count);
		if (I >= Count)
		{
			continue;
		}
		const BbPlayer& P = ShotPlayers[bKitBat ? I : bKitField ? 9 + I : bTurn ? 9 : (Step % 2) * 9 + I];
		if (ABbFigure* Dressed = FigureShop ? Cast<ABbFigure>(F) : nullptr)
		{
			Dressed->Dress(&P, P.side);
			Dressed->SetMirror(Step == 5 || Step == 7);
			Dressed->SetRole(bKitBat ? EBbRole::Batter : !bKitField ? EBbRole::Plain : P.position == 2 ? EBbRole::Catcher : EBbRole::Fielder);
		}
		const double Y = Y0 + Gap * I;
		const float Yaw = bTurn ? -90.f + 45.f * I : (Step == 2 || Step == 3 || Step == 7 ? 90.f : -90.f);
		PlaceDiver(F, ToUE(0.0, Y, Park->GroundZ(0.0, Y)), Yaw, 0.f, 0.f);
		USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
		PlayClip(C, AnimIdle, true);
	}
	Bat->SetActorHiddenInGame(!bKitBat);
	if (bKitBat)
	{
		const double Y = Y0 + Gap * 0.5, G = Park->GroundZ(-0.9, Y);
		PlaceBat(ToUE(-0.9, Y, G + 0.87), ToUE(-0.9, Y, G + 0.01));
	}
	const FVector Eye = ToUE(-8.6, 50.0, 1.35);
	Camera->SetActorLocationAndRotation(Eye, (ToUE(0.0, 50.0, 0.95) - Eye).Rotation());
	Camera->GetCameraComponent()->SetFieldOfView(54.f);
	ShownCam = -1;
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
		St.Motion = 0;
		St.MotionW = 0.f;
		St.Off = FVector2D::ZeroVector;
		St.Ready = 0.f;
		St.Carry = 0.f;
		St.Lower = 1.f;
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
	// The stand-in clips are right-handed. A left-hander's delivery, throw or stance is the
	// same clip in a mirror: the figure is flipped across his own facing (owner backlog A1).
	const bool bLefty = Sim && Sim->ThrowsLeft(A.agent);
	// The coded motions (BbPose.h): the pitcher's delivery and the catcher's crouch. Each
	// begins in the last moments of the reset (he comes set, the catcher squats), so the
	// pitch opens on it.
	UBbPoseAnim* Coded = Skel.Ok() ? Cast<UBbPoseAnim>(C->GetAnimInstance()) : nullptr;
	if (Coded && Coded->PelvisBone == INDEX_NONE)
	{
		Coded->UpperBones = Skel.UpperBones();
		Coded->PelvisBone = Skel.PelvisBone();
	}
	// The coded motions are right-handed, so a left-handed fielder is shown in a mirror the
	// whole time he is out there (his glove then sits on the hand the viewer expects).
	bool bMirror = Coded && bFielder && bLefty;
	int32 Motion = 0;
	float MotionT = St.MotionT;
	const bool bSetUp = bReset && In.bStaged && !In.bMoving && ResetDuration - ResetTime < SetLeadS;
	// A fielder away from the battery is down and set as the pitch is thrown.
	const bool bReadyNow = bFielder && Position >= 3 && bStill && (bSetUp || (!bReset && PlayTime < ReadyUntil));
	St.Ready = FMath::FInterpConstantTo(St.Ready, bReadyNow ? 1.f : 0.f, SimDt, 1.f / 0.35f);
	St.Carry = FMath::FInterpConstantTo(St.Carry, In.bHolds ? 1.f : 0.f, SimDt, 1.f / 0.2f);
	// A man settling under a ball or stepping to a throw, short of a run, squares up to it:
	// he faces the ball and his feet are the pose's. (At a run he takes it going away.)
	const bool bSettle = In.bCatch && In.CatchDt < 0.15f && St.SpeedCm < SettleMaxCmps;
	St.Lower = FMath::FInterpConstantTo(St.Lower, (St.SpeedCm < 60.f || bSettle) && !(In.bStaged && In.bMoving) ? 1.f : 0.f, SimDt, 1.f / 0.25f);
	if (Coded && bMotions && bFielder && In.DiveSince < 0.f)
	{
		if (Position == 1 && !bReset && PlayTime < BbMotion::DeliveryEndS)
		{
			Motion = 1;
			MotionT = PlayTime;
		}
		else if (Position == 1 && bSetUp && St.Off.IsNearlyZero(2.0))
		{
			Motion = 1;
			MotionT = BbMotion::DeliveryStartS;
		}
		else if (Position == 2 && bStill && (bSetUp || (!bReset && PlayTime < CrouchUntil)))
		{
			Motion = 2;
		}
		// A throw to make, a ball to take, the ball in his hands, or the ready position (the
		// catcher's crouch takes the pitch itself).
		if (In.bThrow || (Motion != 2 && In.bCatch) || (Motion == 0 && (In.bHolds || bReadyNow)))
		{
			Motion = 4;
		}
	}
	// A dive (player-visuals §11): as the sim has it, he leaves his feet toward the ball,
	// lies where he landed, and is back up when the sim says.
	if (Coded && bMotions && bFielder && In.DiveSince >= 0.f)
	{
		Motion = 5;
	}
	// The batter at the plate, from the moment he stands in: at ease between pitches, in his
	// stance as the pitcher comes set, and whatever the bat does from there.
	const FBatNote& Bn = bReset ? NextBatNote : BatNote;
	if (Coded && bMotions && !bFielder && Bn.bBat && bStill && (!bReset || (In.bStaged && !In.bMoving))
		&& FMath::Square(A.pos[0] - Bn.Spot.X) + FMath::Square(A.pos[1] - Bn.Spot.Y) < 0.04)
	{
		Motion = 3;
	}
	// A runner sliding into a base.
	if (Coded && bMotions && !bFielder && In.bSlide && !bReset)
	{
		Motion = 6;
		MotionT = In.SlideT;
	}
	// (A pitcher who dives on an early comebacker is diving, not finishing his delivery.)
	if (!Coded && bMotions && bFielder && Position == 1 && !In.bStaged && PlayTime < 0.8f && AnimPitch && In.DiveSince < 0.f)
	{
		// The delivery: the clip's strike (1.0 s in) lands on the release (play time 0).
		Manual = AnimPitch;
		ClipTime = PlayTime + 1.0f;
		WantYaw = YawToPlate(Foot);
		bMirror = bLefty;
	}
	else if (!Coded && bMotions && bFielder && AnimDive && AnimCrouch && In.DiveSince >= 0.f)
	{
		// A dive, as the sim has it: he leaves his feet toward the ball (the sim carries his
		// body there), lies where he landed, and is back up when the sim says. Stand-ins:
		// a held leap with an arm reaching overhead, which the layout (below) turns into a
		// reach for the ball; then, over the last DiveRiseS, a landing clip from its crouch
		// up to standing.
		if (In.DiveUpIn < DiveRiseS)
		{
			Manual = AnimCrouch;
			ClipTime = (1.f - In.DiveUpIn / DiveRiseS) * 0.9f * AnimCrouch->GetPlayLength();
		}
		else
		{
			Manual = AnimDive;
			ClipTime = DiveClipFrac * AnimDive->GetPlayLength();
		}
		if (In.bDiveDir)
		{
			WantYaw = In.DiveYaw;
		}
	}
	else if (!Coded && bMotions && bFielder && AnimThrow && In.ThrowClip >= 0.f && In.ThrowClip <= ThrowClipS)
	{
		// A throw: the clip's release (ThrowReleaseS in) lands on the known release time, and
		// its phase follows from that time every frame, follow-through included. He turns
		// toward where the ball is going. Possession comes from the sim's record (or the
		// staging's plan), never from how near the ball is.
		Manual = AnimThrow;
		ClipTime = In.ThrowClip;
		WantYaw = In.ThrowYaw;
		// The stand-in throw clip strikes with the left arm (owner, 2026-11-15: right-handed
		// infielders looked left-handed), so it is the right-hander who gets the mirror.
		bMirror = bThrowClipLeftArm ? !bLefty : bLefty;
	}
	if (!Coded && bMotions && !Manual && bFielder && Position == 2 && bStill && !In.bStaged && PlayTime < 0.6f && AnimCrouch)
	{
		// The catcher squats to receive the pitch.
		Manual = AnimCrouch;
		ClipTime = 0.1f;
	}
	if (!Coded && bMotions && !bFielder && Frame.has_bat != 0 && AnimStance)
	{
		// The batter: whoever is at the bat's knob. Hands up in a stance; as the bat comes
		// around, his body turns with it.
		const FVector Knob = ToUE(Frame.bat_knob);
		if ((FVector(Knob.X, Knob.Y, 0) - FVector(Foot.X, Foot.Y, 0)).Size() < 160.f && bStill)
		{
			Manual = AnimStance;
			ClipTime = 0.f;
			// A left-handed batter (in the first-base-side box, sim x > 0) is the right-handed
			// stance in a mirror.
			bMirror = A.pos[0] > 0.f;
			const FVector Along = ToUE(Frame.bat_tip) - Knob;
			if (Along.Size2D() > 45.f)
			{
				// Hips lead the barrel: face between the bat's direction and the pitcher.
				WantYaw = (Along.GetSafeNormal2D() + ToUE(0, 1, 0).GetSafeNormal() * 0.6).Rotation().Yaw;
			}
		}
	}
	// A coded motion: how much of it shows. It comes in and goes out over MotionBlendS (a
	// dive or a slide comes in faster: he is down in a third of a second). The motions are
	// right-handed, so a left-hander is shown in a mirror for as long as any of one shows.
	if (Manual || !Coded)
	{
		St.MotionW = 0.f;
		Motion = 0;
	}
	if (Motion != 0 && Motion != St.Motion)
	{
		// (A dive ends standing with the ball in his hands, as the fielder's pose has him.)
		if (!(St.Motion == 5 && Motion == 4))
		{
			St.MotionW = 0.f;
		}
		St.Motion = Motion;
	}
	St.MotionW = FMath::FInterpConstantTo(St.MotionW, Motion != 0 ? 1.f : 0.f, SimDt, 1.f / (Motion >= 5 ? 0.12f : MotionBlendS));
	St.MotionT = MotionT;
	const float MotionShown = St.MotionW * St.MotionW * (3.f - 2.f * St.MotionW);
	if (St.MotionW > 0.f)
	{
		// (The batter is posed facing the pitcher; which side he bats from is where he stands.)
		if (St.Motion == 5)
		{
			// He faces the way he dove.
			WantYaw = In.DiveSince >= 0.f && In.bDiveDir ? In.DiveYaw : St.Yaw;
		}
		else if (St.Motion == 6)
		{
			WantYaw = In.bSlide ? In.SlideYaw : St.Yaw;
		}
		else if (St.Motion != 4)
		{
			WantYaw = St.Motion == 3 ? ToUE(0.0, 1.0, 0.0).Rotation().Yaw : YawToPlate(Foot);
		}
		else if (In.bThrow)
		{
			// He turns to throw where the ball is going.
			WantYaw = In.ThrowYaw;
		}
		else if (bSettle && In.CatchDt < 0.f && (ToUE(Frame.ball) - Foot).SizeSquared2D() > 900.0)
		{
			WantYaw = (ToUE(Frame.ball) - Foot).Rotation().Yaw;
		}
		else if (bSettle)
		{
			WantYaw = St.Yaw;
		}
		bMirror = St.Motion == 3 ? Bn.bLeftBox : St.Motion == 6 ? false : bLefty;
	}
	// Turns ease on play time, so a pause holds them and fast playback keeps up.
	St.Yaw = FMath::RInterpTo(FRotator(0, St.Yaw, 0), FRotator(0, WantYaw, 0), SimDt, Manual || (St.Motion == 4 && In.bThrow) || (St.MotionW > 0.f && St.Motion >= 5) ? 18.f : TurnRate).Yaw;
	St.Pos = Foot;
	// A dive lays him out: tipped forward from the feet over DiveFallS, coming down onto the
	// turf as he lands (late, so he is in the air first), flat until he gets up.
	float Tilt = 0.f;
	float Lift = 0.f;
	if (Manual && Manual == AnimDive && In.DiveSince >= 0.f)
	{
		const float Down = FMath::Clamp(In.DiveSince / DiveFallS, 0.f, 1.f);
		Tilt = DiveTiltDeg * FMath::Sin(HALF_PI * Down);
		Lift = DiveDropCm * Down * Down;
	}
	else if (Manual && Manual == AnimCrouch && In.DiveSince >= 0.f)
	{
		// Getting up: the layout eases off over the first half of the rise, so he comes up
		// from where he lay rather than jumping to his feet.
		const float Left = FMath::Clamp((In.DiveUpIn / DiveRiseS - 0.5f) / 0.5f, 0.f, 1.f);
		const float Ease = Left * Left * (3.f - 2.f * Left);
		Tilt = DiveTiltDeg * Ease;
		Lift = DiveDropCm * Ease;
	}
	// The mannequin faces +Y in its own frame, so its left and right are its X.
	ABbFigure* Dressed = FigureShop ? Cast<ABbFigure>(F) : nullptr;
	if (Dressed)
	{
		// Who he is: his club's uniform, his number, his own height and build.
		Dressed->Dress(Sim ? Sim->Player(A.agent) : nullptr, A.agent >> 8);
		Dressed->SetMirror(bMirror);
		Dressed->SetRole(!bFielder ? EBbRole::Batter : Position == 2 ? EBbRole::Catcher : EBbRole::Fielder);
	}
	else
	{
		const FVector Scale = F->GetActorScale3D();
		const float WantX = FMath::Abs(Scale.X) * (bMirror ? -1.f : 1.f);
		if (Scale.X != WantX)
		{
			F->SetActorScale3D(FVector(WantX, Scale.Y, Scale.Z));
		}
	}
	if (bMirror)
	{
		++MirroredFrames;
	}
	// Where he is drawn. A motion's positions are measured from the sim's place for him
	// (Base); the figure itself may be drawn off it (St.Off): the delivery carries the
	// pitcher down the mound and leaves him there, the catcher sits back of his spot.
	const FQuat FacingQ(FRotator(0.f, St.Yaw - 90.f, 0.f));
	const FVector Scale = F->GetActorScale3D();
	const FTransform Base(FacingQ, Foot, Scale);
	auto OnGround = [&](const FVector2D& V) { return FVector2D(Base.TransformVector(FVector(V.X, V.Y, 0.0))); };
	if (St.Motion == 1 && Motion == 1 && !bReset)
	{
		St.Off = OnGround(BbMotion::DeliveryTravel(PlayTime));
	}
	else if (St.Motion == 2)
	{
		St.Off = OnGround(BbMotion::CrouchBack) * MotionShown;
	}
	else
	{
		// Left off his spot by a motion: gathered in as he runs, stepped back between pitches.
		// (A pitcher with a play to make is where the sim has him.)
		const float Rate = St.SpeedCm > 40.f || (St.Motion == 4 && Motion == 4) ? GatherCmps : bReset ? StepBackCmps : 0.f;
		St.Off = FMath::Vector2DInterpConstantTo(St.Off, FVector2D::ZeroVector, SimDt, Rate);
	}
	const bool bStepBack = bReset && St.SpeedCm <= 40.f && bFielder && Position == 1 && !St.Off.IsNearlyZero(1.0);
	if (St.MotionW <= 0.f && St.Off.IsNearlyZero(0.01))
	{
		St.Motion = 0;
	}
	FVector At = Foot + FVector(St.Off.X, St.Off.Y, 0.0);
	const double GroundAtFoot = Park->GroundZ(A.pos[0], A.pos[1]);
	At.Z += 100.0 * (Park->GroundZ(At.Y / 100.0, At.X / 100.0) - GroundAtFoot);
	PlaceDiver(F, At, St.Yaw, Tilt, Lift);
	St.bHand = false;
	if (Coded)
	{
		Coded->PoseWeight = 0.f;
		Coded->LowerWeight = 1.f;
		if (MotionShown > 0.f && St.Motion != 0)
		{
			const FTransform Own(FacingQ, At, Scale);
			// (The fielder's motion is laid out about the figure as it is drawn.)
			const double GroundAtOwn = Park->GroundZ(At.Y / 100.0, At.X / 100.0);
			auto GroundOwnFn = [&](double X, double Y)
			{
				const FVector Wd = Own.TransformPosition(FVector(X, Y, 0.0));
				return (Park->GroundZ(Wd.Y / 100.0, Wd.X / 100.0) - GroundAtOwn) * 100.0 / FMath::Abs(Scale.Z);
			};
			const FBbMotionGround GroundOwn{ GroundOwnFn };
			FVector Pocket(0, 0, 1);
			auto GroundFn = [&](double X, double Y)
			{
				const FVector Wd = Base.TransformPosition(FVector(X, Y, 0.0));
				return (Park->GroundZ(Wd.Y / 100.0, Wd.X / 100.0) - GroundAtFoot) * 100.0 / FMath::Abs(Scale.Z);
			};
			const FBbMotionGround Ground{ GroundFn };
			FBbReach Reach;
			FBbStick S;
			if (St.Motion == 1)
			{
				S = BbMotion::Delivery(St.MotionT, Skel, Ground);
				// The hand to the sim's release point as the ball goes.
				Reach.At[1] = Own.InverseTransformPosition(PitchFrom);
				Reach.Weight[1] = bReset ? 0.f : BbMotion::DeliveryReach(St.MotionT);
				Reach.Past[1] = BbMotion::BallInHandCm;
			}
			else if (St.Motion == 4)
			{
				// A fielder (player-visuals §10): ready, the glove to the ball where the sim
				// has it caught, the ball brought in to his hand, the throw with the hand on
				// the sim's release point as the ball goes.
				BbMotion::FFieldIn Fi;
				Fi.Ready = St.Ready;
				Fi.Carry = St.Carry;
				Fi.Lower = St.Lower;
				if (In.bCatch)
				{
					Fi.GloveAt = Own.InverseTransformPosition(In.CatchAt);
					Fi.Glove = FMath::SmoothStep(-GloveOutS, -GloveThereS, In.CatchDt) * (1.f - FMath::SmoothStep(GloveStayS, GloveInS, In.CatchDt));
				}
				if (In.bThrow)
				{
					Fi.ThrowT = In.ThrowT;
					Fi.Throw = FMath::SmoothStep(BbMotion::ThrowStartS, BbMotion::ThrowStartS + 0.15f, In.ThrowT) * (1.f - FMath::SmoothStep(BbMotion::ThrowEndS - 0.2f, BbMotion::ThrowEndS, In.ThrowT));
					if (In.bCatch && In.CatchDt > -0.05f)
					{
						// A quick one: the throw waits for the ball to be in the glove.
						Fi.Throw *= FMath::SmoothStep(0.04f, 0.2f, In.CatchDt);
					}
					if (In.bThrowFrom)
					{
						Reach.At[1] = Own.InverseTransformPosition(In.ThrowFrom);
						Reach.Weight[1] = BbMotion::ThrowReach(In.ThrowT) * Fi.Throw;
						Reach.Past[1] = BbMotion::BallInHandCm;
						Fi.ThrowSink = (float)FMath::Clamp(168.0 - Reach.At[1].Z, 0.0, 30.0);
					}
				}
				// His eyes: on the ball until he has it, then on where it is to go.
				Fi.Look = In.bHolds || In.bThrow || bReset ? FVector(0.0, 1500.0, 150.0) : Own.InverseTransformPosition(ToUE(Frame.ball));
				if (Coded->bClipPelvis)
				{
					Fi.ClipPelvis = Coded->ClipPelvis.GetLocation();
					Skel.PelvisFacing(Coded->ClipPelvis, Fi.ClipFwd, Fi.ClipUp);
				}
				else
				{
					Fi.Lower = 1.f;
				}
				// A low ball taken on the run: he gets down to it, legs and all.
				// One over his head he leaves his feet for.
				Fi.Lower = FMath::Max3(Fi.Lower, Fi.Glove * (1.f - FMath::SmoothStep(45.f, 95.f, (float)Fi.GloveAt.Z)), Fi.Glove * FMath::SmoothStep(0.f, 8.f, (float)BbMotion::JumpForCm(Fi.GloveAt.Z)));
				Coded->LowerWeight = Fi.Lower;
				S = BbMotion::Field(Fi, Skel, GroundOwn, Reach, Pocket);
			}
			else if (St.Motion == 5)
			{
				// A dive: the glove to the ball if he gets one. (Going out, he is on his feet.)
				FVector GloveAt(0.0, 95.0, 20.0);
				float Glove = 0.f;
				if (In.bCatch)
				{
					GloveAt = Own.InverseTransformPosition(In.CatchAt);
					// (The glove stays on the ball while the rest of him comes down behind it.)
					Glove = FMath::SmoothStep(-GloveOutS, -GloveThereS, In.CatchDt) * (1.f - FMath::SmoothStep(0.2f, GloveInS + 0.02f, In.CatchDt));
				}
				const bool bDown = In.DiveSince >= 0.f;
				S = BbMotion::Dive(bDown ? In.DiveSince : 9.f, bDown ? In.DiveUpIn : 0.f, GloveAt, Glove, Own.InverseTransformPosition(ToUE(Frame.ball)), Skel, GroundOwn, Reach, Pocket);
			}
			else if (St.Motion == 6)
			{
				S = BbMotion::Slide(St.MotionT, Skel, GroundOwn);
			}
			else if (St.Motion == 3)
			{
				// The batter. The bat is the sim's while the sim has it. Otherwise it is his
				// to carry, in his own frame:
				auto Put = [&](const FVector& K, const FVector& Axis, FVector& OutKnob, FVector& OutTip)
				{
					OutKnob = Base.TransformPosition(K);
					OutTip = OutKnob + Base.TransformVector(Axis).GetSafeNormal() * BatLenCm;
				};
				if (!bBatSim && !bReset && Bn.bSwing && PlayTime > Bn.EndT)
				{
					// back to his shoulder from where the swing left it. A full swing has it
					// round in front of him: it is unwound there, the barrel dropping past
					// the plate and coming up behind, not drawn back through his chest.
					const FVector K0 = Base.InverseTransformPosition(Bn.EndKnob);
					const FVector A0 = (Base.InverseTransformPosition(Bn.EndTip) - K0).GetSafeNormal();
					const FVector A3 = BbMotion::RestAxis.GetSafeNormal();
					const float U = FMath::SmoothStep(0.f, 1.f, (PlayTime - Bn.EndT) / BatReturnS);
					FVector K = FMath::Lerp(K0, BbMotion::RestKnob, (double)U), Ax = FMath::Lerp(A0, A3, (double)U);
					if (A0.X > 0.2)
					{
						const FVector A1(0.0, 1.0, -0.1), A2(-0.75, 0.25, -0.6);
						const double V = U * 3.0;
						Ax = V < 1.0 ? FMath::Lerp(A0, A1, V) : V < 2.0 ? FMath::Lerp(A1, A2, V - 1.0) : FMath::Lerp(A2, A3, V - 2.0);
						K.Z -= 8.0 * FMath::Sin(PI * U);
					}
					Put(K, Ax, BatKnob, BatTip);
				}
				else if (!bBatSim)
				{
					// at ease on his shoulder between pitches, up into the next pitch's
					// stance as the pitcher comes set.
					FVector WantKnob = Bn.StanceKnob, WantTip = Bn.StanceTip;
					if (bReset && ResetDuration - ResetTime >= SetLeadS)
					{
						Put(BbMotion::RestKnob, BbMotion::RestAxis, WantKnob, WantTip);
					}
					if (BatHolder != A.agent)
					{
						BatKnob = WantKnob;
						BatTip = WantTip;
					}
					else
					{
						BatKnob = FMath::VInterpTo(BatKnob, WantKnob, SimDt, 6.f);
						BatTip = FMath::VInterpTo(BatTip, WantTip, SimDt, 6.f);
						BatTip = BatKnob + (BatTip - BatKnob).GetSafeNormal() * BatLenCm;
					}
				}
				BatHolder = A.agent;
				bBatHeld = true;
				BbMotion::FSwingIn Sw;
				Sw.Knob = Own.InverseTransformPosition(BatKnob);
				Sw.Tip = Own.InverseTransformPosition(BatTip);
				Sw.Look = Own.InverseTransformPosition(BatLook);
				Sw.Swing = BatSwingW;
				Sw.Stride = BatStride;
				Sw.Rest = BatRest;
				Sw.TurnDeg = !bReset && Bn.bSwing && PlayTime >= Bn.LaunchT ? (float)BbMotion::HandsDeg(Sw.Knob, Sw.Tip) - Bn.LaunchDeg : 0.f;
				S = BbMotion::Swing(Sw, Skel, Ground, Reach);
			}
			else
			{
				// The mitt: held for a target, then to the ball where the sim has it caught.
				FVector Mitt = ToUE(0.0, MittPlaneY, MittTargetZ);
				if (!bReset && PitchEndT >= 0.f)
				{
					const float U = FMath::Clamp((PlayTime - (PitchEndT - ReceiveEarlyS - ReceiveMoveS)) / ReceiveMoveS, 0.f, 1.f);
					Mitt = FMath::Lerp(Mitt, PitchEndAt, (double)(U * U * (3.f - 2.f * U)));
				}
				S = BbMotion::Crouch(Own.InverseTransformPosition(Mitt), Skel, Ground, Reach);
			}
			// From the motion's frame to the figure's own, wherever it is drawn.
			for (FVector* P : { &S.Pelvis, &S.Ankle[0], &S.Ankle[1] })
			{
				if (St.Motion < 4)
				{
					*P = Own.InverseTransformPosition(Base.TransformPosition(*P));
				}
			}
			FVector Hold[2], Wrist[2];
			Skel.Solve(S, Reach, Coded->PoseLocal, Hold, Wrist);
			if (St.Motion == 4)
			{
				// The ball he has: in the glove's pocket as he takes it, then in his hand.
				const FVector InGlove = Wrist[0] + Pocket * BbMotion::BallInMittCm;
				if (In.bHolds && MotionShown >= 0.5f)
				{
					const float ToHand = In.bCatch && In.CatchDt >= 0.f ? FMath::SmoothStep(GloveStayS, GloveInS - 0.04f, In.CatchDt) : 1.f;
					St.bHand = true;
					St.Hand = Own.TransformPosition(FMath::Lerp(InGlove, Hold[1], (double)ToHand));
				}
				// -BbStagingLog: the glove's pocket from the ball as he takes it, and his hand
				// from the sim's release point as he lets it go (the play is held on each).
				if (bStagingLog && In.bCatch && In.CatchDt == 0.f)
				{
					UE_LOG(LogBb, Display, TEXT("BB_CATCH play=%u agent=0x%03x staged=%d left=%d ball=(%.2f, %.2f, %.2f) m from feet, glove_off=%.1f cm lower=%.2f shown=%.2f"), Current.Info.play_index, A.agent, bReset ? 1 : 0, bLefty ? 1 : 0,
						(In.CatchAt.Y - At.Y) / 100.0, (In.CatchAt.X - At.X) / 100.0, (In.CatchAt.Z - At.Z) / 100.0, (Own.TransformPosition(InGlove) - In.CatchAt).Size(), Coded->LowerWeight, MotionShown);
				}
				if (bStagingLog && In.bThrow && In.ThrowT == 0.f && In.bThrowFrom)
				{
					UE_LOG(LogBb, Display, TEXT("BB_LETGO play=%u agent=0x%03x staged=%d left=%d hand_off=%.1f cm lower=%.2f shown=%.2f point=(%.0f, %.0f, %.0f) cm his frame, weight=%.2f yaw_off=%.0f deg"), Current.Info.play_index, A.agent, bReset ? 1 : 0, bLefty ? 1 : 0,
						(Own.TransformPosition(Hold[1]) - In.ThrowFrom).Size(), St.Lower, MotionShown, Reach.At[1].X, Reach.At[1].Y, Reach.At[1].Z, Reach.Weight[1], FRotator::NormalizeAxis(St.Yaw - In.ThrowYaw));
				}
			}
			if (St.Motion == 5)
			{
				// The ball he dove for and got stays in his glove until he is up.
				const FVector InGlove = Wrist[0] + Pocket * BbMotion::BallInMittCm;
				if (In.bHolds && MotionShown >= 0.5f)
				{
					St.bHand = true;
					St.Hand = Own.TransformPosition(InGlove);
				}
				if (bStagingLog && In.bCatch && In.CatchDt == 0.f)
				{
					UE_LOG(LogBb, Display, TEXT("BB_CATCH play=%u agent=0x%03x dive=1 since=%.2f left=%d ball=(%.2f, %.2f, %.2f) m from him, glove_off=%.1f cm shown=%.2f"), Current.Info.play_index, A.agent, In.DiveSince, bLefty ? 1 : 0,
						Reach.At[0].X / 100.0, Reach.At[0].Y / 100.0, Reach.At[0].Z / 100.0, (Own.TransformPosition(InGlove) - In.CatchAt).Size(), MotionShown);
				}
			}
			// -BbStagingLog: how far the hand is from the sim's release point as the ball goes
			// (the play is held on that moment), and the mitt from the ball once it is caught.
			if (bStagingLog && St.Motion == 1 && !bReset && PlayTime == 0.f)
			{
				UE_LOG(LogBb, Display, TEXT("BB_RELEASE play=%u agent=0x%03x left=%d height=%.3f hand_off=%.1f cm shown=%.2f"), Current.Info.play_index, A.agent, bLefty ? 1 : 0,
					FMath::Abs(Scale.Z), (Own.TransformPosition(Hold[1]) - PitchFrom).Size(), MotionShown);
			}
			if (bStagingLog && St.Motion == 2 && !bReset && PitchEndT >= 0.f && Current.Info.contact_s < 0.f && PlayTime >= PitchEndT + 0.05f && PlayTime - SimDt < PitchEndT + 0.05f)
			{
				UE_LOG(LogBb, Display, TEXT("BB_RECEIVE play=%u agent=0x%03x left=%d ball=(%.2f, %.2f) wrist_off=%.1f cm shown=%.2f"), Current.Info.play_index, A.agent, bLefty ? 1 : 0,
					PitchEndAt.Y / 100.0, PitchEndAt.Z / 100.0, (Wrist[0] - Reach.At[0]).Size() * FMath::Abs(Scale.Z), MotionShown);
			}
			// And each hand from the handle as the bat meets the ball (0: on it).
			if (bStagingLog && St.Motion == 3 && !bReset && Current.Info.contact_s >= 0.f && PlayTime == Current.Info.contact_s)
			{
				UE_LOG(LogBb, Display, TEXT("BB_SWING play=%u agent=0x%03x left=%d height=%.3f low_off=%.1f high_off=%.1f cm launch=%.3f s turn=%.0f deg shown=%.2f"), Current.Info.play_index, A.agent, Bn.bLeftBox ? 1 : 0,
					FMath::Abs(Scale.Z), ((Wrist[0] - Reach.At[0]).Size() - Reach.Past[0]) * FMath::Abs(Scale.Z), ((Wrist[1] - Reach.At[1]).Size() - Reach.Past[1]) * FMath::Abs(Scale.Z),
					Bn.LaunchT, BbMotion::HandsDeg(Own.InverseTransformPosition(BatKnob), Own.InverseTransformPosition(BatTip)) - Bn.LaunchDeg, MotionShown);
			}
			Coded->PoseWeight = MotionShown;
			if (St.Motion == 1 && !bReset && PlayTime < 0.f && MotionShown >= 0.99f)
			{
				St.bHand = true;
				St.Hand = Own.TransformPosition(Hold[1]);
			}
		}
	}

	if (Manual)
	{
		if (St.Manual != Manual)
		{
			St.Manual = Manual;
			St.Gait = -1;
			PlayClip(C, const_cast<UAnimSequence*>(Manual), false);
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
	// A pitcher going back up to the rubber backs up the mound, watching the plate.
	if (bStepBack && Gait == 0 && AnimWalkBack)
	{
		Gait = 3;
	}
	if (!Manual && Gait != St.Gait)
	{
		UAnimSequence* Clips[6] = { AnimIdle, AnimWalk, AnimJog, AnimWalkBack, AnimWalkLeft, AnimWalkRight };
		St.Gait = Gait;
		PlayClip(C, Clips[Gait], true);
	}
	// The jog clip covers JogClipMps; the walk clips are played as covering about 2.3 m/s
	// (their root bones travel 3.0 m/s, but the walk was not retuned with the jog). Play
	// faster for a sprint, slower for a step.
	if (!Manual)
	{
		const float Floor = In.bStaged ? 0.4f : 0.6f;
		const float Rate = Gait == 0 ? 1.f : FMath::Clamp((bStepBack ? StepBackCmps : St.SpeedCm) / (Gait == 2 ? JogClipMps * 100.f : 230.f), Floor, 1.9f);
		C->SetPlayRate(SimDt <= 1e-5f ? 0.f : Rate * (SimDt / FMath::Max(DeltaSeconds, 1e-5f)));
	}

	// The bare mannequin (-BbMannequin): home white, away blue; fielders a shade apart from
	// batters and runners.
	const bool bHome = (A.agent >> 8) != 0;
	const int32 Team = (bHome ? 2 : 0) + (bFielder ? 1 : 0);
	if (!Dressed && Team != St.Team)
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
	// A home run, once the ball is over the wall (A16): from beyond center field looking in at
	// the diamond, for the trot, until the next pitch.
	if (bHomeRunCut && CameraMode == 0)
	{
		if (ShownCam != 3)
		{
			ShownCam = 3;
			Camera->SetActorLocation(ToUE(0, 125, 22));
			CamLook = ToUE(0, 21, 0);
			C->SetFieldOfView(HrViewFov);
		}
		Camera->SetActorRotation((CamLook - Camera->GetActorLocation()).Rotation());
		return;
	}
	const bool bFollow = bBallInPlay || CameraMode == 1;
	// Once the ball has left the field and is dead the camera stops following it: the high
	// view from behind home on the whole field (A10). Behind home (mode 1) a home run does
	// the same.
	const bool bGone = bBallDead || bHomeRunCut;
	const bool bOnBall = bBallInPlay && !bGone;
	const int32 Want = bFollow ? 1 : 0;
	if (Want != ShownCam)
	{
		ShownCam = Want;
		Camera->SetActorLocation(bFollow ? ToUE(0, -34, 19) : ToUE(-9, 136, 16));
		CamLook = bFollow ? (bOnBall ? FVector(BallUE.X, BallUE.Y, 0.0) : ToUE(0, 16, 0)) : ToUE(0, 6, 1.0);
		C->SetFieldOfView(bFollow ? 48.f : 8.5f);
	}
	if (bFollow && bGone && !bDeadCut)
	{
		// A cut, not a pan back from the seats.
		bDeadCut = true;
		CamLook = ToUE(0, 16, 0);
		C->SetFieldOfView(55.f);
	}
	if (bFollow)
	{
		const FVector Target = bOnBall ? FVector(BallUE.X, BallUE.Y, FMath::Min(BallUE.Z, 1500.0) * 0.5) : ToUE(0, 16, 0);
		CamLook = FMath::VInterpTo(CamLook, Target, DeltaSeconds, 4.f);
		C->SetFieldOfView(FMath::FInterpTo(C->FieldOfView, bOnBall ? 42.f : 55.f, DeltaSeconds, 2.f));
	}
	Camera->SetActorRotation((CamLook - Camera->GetActorLocation()).Rotation());
}

void ABbDirector::PlaceBallShadow(const BbFrame& Frame, bool bShown)
{
	// Just above the drawn ground (which rises at the mound), below anything else.
	const float GroundM = (float)Park->GroundZ(Frame.ball[0], Frame.ball[1]);
	const FVector At = ToUE(Frame.ball[0], Frame.ball[1], GroundM + 0.012);
	const FVector Look = At - Camera->GetActorLocation();
	const float DistM = Look.Size() / 100.f;
	// Width of the view at the shadow's distance, m: a far shadow is drawn larger so it keeps
	// about a fixed share of the screen (a long lens shows less, so it needs less). Slant
	// distance stands in for the camera-space depth, which is close enough near the aim.
	const float HalfFov = FMath::DegreesToRadians(Camera->GetCameraComponent()->FieldOfView * 0.5f);
	const float ViewM = 2.f * DistM * FMath::Tan(HalfFov);
	// Height reads from the size (owner, 2026-11-15): nothing under a ball on the ground or
	// rolling, a small shadow that closes up as the ball comes down to the grass, and a
	// larger one the higher the ball is.
	const float H = FMath::Max(0.f, Frame.ball[2] - GroundM);
	const float Near = FMath::Clamp((H - BallShadowNoneM) / (BallShadowFullM - BallShadowNoneM), 0.f, 1.f);
	const float High = FMath::Clamp((H - BallShadowFullM) / (BallShadowHighM - BallShadowFullM), 0.f, 1.f);
	const float Climb = Near * FMath::Lerp(1.f, BallShadowHighScale, High);
	bShown = bShown && Climb > 0.f;
	const float Across = FMath::Max(BallShadowM, ViewM * BallShadowViewShare) * Climb;
	// Seen from above at angle E, a disc shows sin E of its width in height. Lengthen it along
	// the line of sight by 1/sqrt(sin E), at most BallShadowMaxStretch.
	const float SinE = DistM > 0.f ? FMath::Clamp((float)(-Look.Z / Look.Size()), 0.f, 1.f) : 1.f;
	const float Along = Across * FMath::Min(BallShadowMaxStretch, 1.f / FMath::Sqrt(FMath::Max(SinE, 1e-4f)));
	// The cylinder's local X lies along the line of sight on the ground.
	const float Yaw = FMath::RadiansToDegrees(FMath::Atan2(Look.Y, Look.X));
	BallShadow->SetActorLocationAndRotation(At, FRotator(0.f, Yaw, 0.f));
	BallShadow->SetActorScale3D(FVector(Along, Across, BallShadow->GetActorScale3D().Z));
	BallShadow->SetActorHiddenInGame(!bShown);
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

	// Keys: Space pauses, - and = change speed, C changes camera, Tab holds the box score,
	// Esc or Q quits.
	bool bTabHeld = false;
	// The save dialog (BbSavePlay): its buttons close it here, a tick after their click.
	if (SaveClose != 0)
	{
		CloseSaveDialog(SaveClose == 2);
	}
	// -BbSaveAt=<s>: S, then Save a second later, for an unattended check.
	if (SaveAt >= 0.f && Current.bValid && Wall >= SaveAt + (SaveAuto == 1 ? 1.f : 0.f))
	{
		if (SaveAuto == 0)
		{
			OpenSaveDialog();
			++SaveAuto;
		}
		else
		{
			SaveComment = TEXT("Unattended check (-BbSaveAt): \"quotes\", a back\\slash\nand a second line.");
			CloseSaveDialog(true);
			SaveAt = -1.f;
		}
	}
	SyncCursor();
	APlayerController* KeysPC = bSaveOpen ? nullptr : GetWorld()->GetFirstPlayerController();
	if (APlayerController* PC = KeysPC)
	{
		// S: save the pitch on screen, with a comment (BbSavePlay).
		if (PC->WasInputKeyJustPressed(EKeys::S) && Current.bValid)
		{
			OpenSaveDialog();
		}
		if (PC->WasInputKeyJustPressed(EKeys::SpaceBar))
		{
			bPaused = !bPaused;
		}
		// Speed keys (owner, 2026-11-16): 1 to 5 jump to 1x, 2x, 4x, 6x and 10x; minus slows
		// down a quarter at a time (to 0.25x at least), equals speeds up the same way.
		static const FKey SpeedKeys[] = { EKeys::One, EKeys::Two, EKeys::Three, EKeys::Four, EKeys::Five };
		static const float SpeedJumps[] = { 1.f, 2.f, 4.f, 6.f, 10.f };
		for (int32 I = 0; I < UE_ARRAY_COUNT(SpeedKeys); ++I)
		{
			if (PC->WasInputKeyJustPressed(SpeedKeys[I]))
			{
				Speed = SpeedJumps[I];
			}
		}
		if (PC->WasInputKeyJustPressed(EKeys::Hyphen))
		{
			Speed = FMath::Max(0.25f, Speed - 0.25f);
		}
		if (PC->WasInputKeyJustPressed(EKeys::Equals))
		{
			Speed = FMath::Min(10.f, Speed + 0.25f);
		}
		// Enter: run at RushSpeed (or the chosen speed, if faster) until the next pitch is
		// ready to be thrown, then back to the chosen speed (owner, 2026-11-15). Again: stop.
		if (PC->WasInputKeyJustPressed(EKeys::Enter))
		{
			bRush = !bRush;
		}
		if (PC->WasInputKeyJustPressed(EKeys::R))
		{
			ReplayBack();
		}
		if (PC->WasInputKeyJustPressed(EKeys::F))
		{
			ReplayForward();
		}
		if (PC->WasInputKeyJustPressed(EKeys::C))
		{
			CameraMode = (CameraMode + 1) % 3;
		}
		if (PC->WasInputKeyJustPressed(EKeys::Escape) || PC->WasInputKeyJustPressed(EKeys::Q))
		{
			FPlatformMisc::RequestExit(false);
		}
		bTabHeld = PC->IsInputKeyDown(EKeys::Tab);
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
			Hud->Bug = FBbBugInfo();
			Hud->Call = TEXT("Warming up");
		}
	}

	if (Current.bValid && Sim)
	{
		// -BbReplayAt=<s>: R twice, then F six seconds later, for an unattended check.
		if (ReplayAt >= 0.f && (RushClock += RushAt >= 0.f ? 0.f : DeltaSeconds) >= ReplayAt + (ReplayAuto == 2 ? 6.f : 0.f))
		{
			if (ReplayAuto < 2)
			{
				ReplayBack();
				++ReplayAuto;
			}
			else
			{
				ReplayForward();
				ReplayAt = -1.f;
			}
		}
		// -BbRushAt=<s>: Enter, pressed for an unattended check that many seconds in.
		if (RushAt >= 0.f && (RushClock += DeltaSeconds) >= RushAt)
		{
			RushAt = -1.f;
			bRush = true;
			UE_LOG(LogBb, Display, TEXT("BB_RUSH on play=%u play_time=%.2f reset=%d"), Current.Info.play_index, PlayTime, bReset ? 1 : 0);
		}
		const float SimDt = bPaused ? 0.f : DeltaSeconds * (bRush ? FMath::Max(Speed, RushSpeed) : Speed);
		PlayTime += SimDt;
		ExitClock += SimDt;
		// The play is held on the moment of release for a frame when the motions are checked.
		if ((bStagingLog || bMotionShots) && !bReset && PlayTime >= 0.f && PlayTime - SimDt < 0.f)
		{
			PlayTime = 0.f;
		}
		// And on the moment of contact, for the hands' distance from the handle.
		if (bStagingLog && !bReset && Current.Info.contact_s >= 0.f && PlayTime >= Current.Info.contact_s && PlayTime - SimDt < Current.Info.contact_s)
		{
			PlayTime = Current.Info.contact_s;
		}
		// And on each catch and each throw's release, for the glove's and the hand's distances.
		if (bStagingLog && !bReset)
		{
			float HoldAt = -1.f;
			auto Crossed = [&](float At)
			{
				if (PlayTime >= At && PlayTime - SimDt < At && (HoldAt < 0.f || At < HoldAt))
				{
					HoldAt = At;
				}
			};
			for (const FCatchNote& N : Catches)
			{
				Crossed(N.T);
			}
			for (const FThrowNote& N : Throws)
			{
				Crossed(N.ReleaseAt);
			}
			if (HoldAt >= 0.f)
			{
				PlayTime = HoldAt;
			}
		}
		// -BbMotionShots: the first pitch from beside the pitcher, the second from the plate,
		// the third beside the catcher; then the batter: the first ball hit, the first swing
		// and miss, the first pitch taken. Each held on its named moments and photographed.
		if (bMotionShots && !bReset && Current.bValid && MotionStrip >= 0)
		{
			struct FMark { float T; const TCHAR* Name; };
			const float E = PitchEndT >= 0.f ? PitchEndT : 0.45f;
			const FBatNote& B = BatNote;
			const float C = Current.Info.contact_s >= 0.f ? Current.Info.contact_s : B.LaunchT + 0.15f;
			const FMark Swing[] = { { -0.5f, TEXT("stance") }, { 0.f, TEXT("stride") }, { B.LaunchT, TEXT("launch") }, { 0.5f * (B.LaunchT + C), TEXT("turn") }, { C, TEXT("contact") },
				{ C + 0.05f, TEXT("through") }, { B.EndT, TEXT("finish") }, { B.EndT + 0.4f * BatReturnS, TEXT("back") }, { B.EndT + BatReturnS + 0.1f, TEXT("rest") } };
			const FMark Take[] = { { 0.17f, TEXT("stride_down") }, { E + 0.15f, TEXT("take") } };
			const FMark Pitch[] = { { -1.3f, TEXT("set") }, { -0.84f, TEXT("lift") }, { -0.48f, TEXT("stride") }, { -0.16f, TEXT("plant") },
				{ -0.045f, TEXT("laid_back") }, { 0.f, TEXT("release") }, { 0.2f, TEXT("follow") }, { 0.6f, TEXT("finish") } };
			const FMark Catch[] = { { -0.6f, TEXT("target") }, { E - 0.12f, TEXT("reach") }, { E + 0.01f, TEXT("receive") }, { E + 0.5f, TEXT("hold") } };
			// (Not the first pitch: the picture is still settling. A ball hit has the batter
			// running once his swing is done, so its strip stops at the finish.)
			const int32 Which = MotionStrip;
			// The field: a ball taken and thrown; a throw taken.
			TArray<FMark, TInlineAllocator<10>> Field;
			if (Which >= 6 && Catches.IsValidIndex(StripCatch))
			{
				const float Tc = Catches[StripCatch].T;
				Field.Add({ 0.05f, TEXT("ready") });
				Field.Add({ Tc - 0.5f, TEXT("coming") });
				Field.Add({ Tc - 0.12f, TEXT("reach") });
				Field.Add({ Tc, TEXT("catch") });
				if (Which == 6 && Throws.IsValidIndex(StripThrow))
				{
					const float Tr = Throws[StripThrow].ReleaseAt;
					Field.Add({ Tc + 0.22f, TEXT("in") });
					Field.Add({ Tr - 0.27f, TEXT("step") });
					Field.Add({ Tr - 0.12f, TEXT("cocked") });
					Field.Add({ Tr - 0.04f, TEXT("laid_back") });
					Field.Add({ Tr, TEXT("release") });
					Field.Add({ Tr + 0.18f, TEXT("follow") });
				}
				else
				{
					Field.Add({ Tc + 0.3f, TEXT("in") });
				}
				for (int32 K = 1; K < Field.Num(); ++K)
				{
					Field[K].T = FMath::Max(Field[K].T, Field[K - 1].T + 0.01f);
				}
			}
			const FMark* Marks = Which >= 6 ? Field.GetData() : Which < 2 ? Pitch : Which == 2 ? Catch : Which < 5 ? Swing : Take;
			const int32 Count = Which >= 6 ? Field.Num() : Which < 2 ? UE_ARRAY_COUNT(Pitch) : Which == 2 ? UE_ARRAY_COUNT(Catch) : Which == 3 ? UE_ARRAY_COUNT(Swing) - 2 : Which == 4 ? UE_ARRAY_COUNT(Swing) : UE_ARRAY_COUNT(Take);
			// (No moment later than the pitch stays on screen.)
			const float At = MotionShotNext < Count ? FMath::Min(Marks[MotionShotNext].T, Current.Info.duration_s + HoldAfter - 0.05f) : 0.f;
			if (MotionShotNext < Count && PlayTime >= At)
			{
				PlayTime = At;
				const FString Name = FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), FString::Printf(TEXT("motion_%d_%d_%s.png"), Which + 1, MotionShotNext + 1, Marks[MotionShotNext].Name));
				FScreenshotRequest::RequestScreenshot(Name, false, false);
				if (++MotionShotNext == Count)
				{
					bStripDone[Which] = true;
					if (bStripDone[2] && bStripDone[3] && bStripDone[4] && bStripDone[5] && bStripDone[6] && bStripDone[7])
					{
						BenchSeconds = FMath::Max(BenchSeconds, (float)Wall + 2.f);
					}
				}
			}
		}
		// -BbSlideShots, -BbDiveShots: the play held on each of the slide's or the dive's
		// moments (NoteSlides lists them) and photographed.
		if (!bReset && Current.bValid && ExtraShotNext < ExtraShots.Num())
		{
			const float At = FMath::Min(ExtraShots[ExtraShotNext].Key, Current.Info.duration_s + HoldAfter - 0.05f);
			if (PlayTime >= At)
			{
				PlayTime = At;
				const FString Name = FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), ExtraShots[ExtraShotNext].Value);
				FScreenshotRequest::RequestScreenshot(Name, false, false);
				UE_LOG(LogBb, Display, TEXT("BB_EXTRA_SHOT %s play %u t %.2f"), *Name, Current.Info.play_index, At);
				if (++ExtraShotNext == ExtraShots.Num() && SlideShotsLeft <= 0 && DiveShotsLeft <= 0)
				{
					BenchSeconds = FMath::Max(BenchSeconds, (float)Wall + 2.f);
				}
			}
		}
		const float T = FMath::Clamp(PlayTime, 0.f, Current.Info.duration_s);
		BbFrame Frame;
		bool bHave = false;
		// During the wind-up the ball is in the pitcher's glove.
		bool bBallShown = PlayTime >= 0.f;
		if (bReset)
		{
			ResetTime += SimDt;
			const float Black = SideCutFade();
			if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
			{
				if (PC->PlayerCameraManager)
				{
					PC->PlayerCameraManager->SetManualCameraFade(Black, FLinearColor::Black, false);
				}
				if (ABbHUD* CardHud = Cast<ABbHUD>(PC->GetHUD()))
				{
					CardHud->CardAlpha = Black;
					if (Black > 0.f && CardHud->Card.IsEmpty())
					{
						// The half just played is the board before the last play.
						static const TCHAR* Ord[] = { TEXT("th"), TEXT("st"), TEXT("nd"), TEXT("rd") };
						const int32 N = Before.inning;
						const TCHAR* Suffix = (N % 100 >= 11 && N % 100 <= 13) || N % 10 > 3 ? Ord[0] : Ord[N % 10];
						CardHud->Card = FString::Printf(TEXT("%s of the %d%s"), Before.top ? TEXT("Middle") : TEXT("End"), N, Suffix);
						CardHud->CardRuns[0] = Current.Info.away_runs;
						CardHud->CardRuns[1] = Current.Info.home_runs;
					}
					else if (Black <= 0.f)
					{
						CardHud->Card.Reset();
					}
				}
			}
			ResetFrame(ResetTime, Frame, bBallShown);
			bHave = true;
		}
		else
		{
			bHave = SampleDrawn(Current, T, Frame);
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
		// A ball that left the field is dead (A10; rules-spec 5.8): a foul or a throw into the
		// seats or a dugout, or a fair ball bounding out. The camera stops following it and cuts
		// back to the field, and the ball is gone from view. A home run keeps the follow (A16).
		if (!bBallDead && Current.Info.dead_s >= 0.f && Current.Info.dead_reason != 2 && PlayTime >= Current.Info.dead_s)
		{
			bBallDead = true;
			UE_LOG(LogBb, Display, TEXT("BB_DEAD wall=%.2f play=%u t=%.2f reason=%u at=(%.1f, %.1f)"), Wall, Current.Info.play_index, Current.Info.dead_s, Current.Info.dead_reason, Current.Info.dead_at[0], Current.Info.dead_at[1]);
		}
		// A home run (A16): once the ball is over the wall the camera cuts to center field to
		// watch the trot, the ball is gone, and the card goes up.
		if (bHomeRun && !bHomeRunCut && PlayTime >= Current.Info.dead_s)
		{
			bHomeRunCut = true;
			const int32 Runs = Before.top ? Current.Info.away_runs - Before.away_runs : Current.Info.home_runs - Before.home_runs;
			static const TCHAR* Kinds[] = { TEXT("Home run"), TEXT("Solo home run"), TEXT("Two-run home run"), TEXT("Three-run home run"), TEXT("Grand slam") };
			HrKind = Kinds[FMath::Clamp(Runs, 0, 4)];
			HrName = Sim->FamilyName(Current.Info.batter).ToUpper();
			HrLine = FString::Printf(TEXT("%d ft     %d mph"), FMath::RoundToInt(Current.Info.projected_distance_m / 0.3048f), FMath::RoundToInt(Current.Info.exit_velocity_mps * 2.2369363f));
			UE_LOG(LogBb, Display, TEXT("BB_HR wall=%.2f play=%u t=%.2f trot_end=%.2f card=\"%s / %s / %s\""), Wall, Current.Info.play_index, Current.Info.dead_s, Current.Info.duration_s, *HrName, *HrLine, *HrKind);
		}
		if ((bBallDead || bHomeRunCut) && !bReset)
		{
			bBallShown = false;
		}
		if (bHave)
		{
			if (!Skel.Ok())
			{
				// Without the coded motions the bat is the sim's alone, in nobody's hands.
				Bat->SetActorHiddenInGame(Frame.has_bat == 0);
				if (Frame.has_bat != 0)
				{
					PlaceBat(ToUE(Frame.bat_knob), ToUE(Frame.bat_tip));
				}
			}
			// The bat (player-visuals §9). It is the sim's for as long as the sim's bat is in
			// his hands and moving or waiting to: through the stance and the swing. After the
			// swing, and between pitches, it is the batter's own to carry (PoseFigure).
			bBatHeld = false;
			bBatSim = !bReset && Frame.has_bat != 0 && (!BatNote.bSwing || PlayTime <= BatNote.EndT);
			if (bBatSim)
			{
				BatKnob = ToUE(Frame.bat_knob);
				BatTip = ToUE(Frame.bat_tip);
			}
			{
				// What the batter's body goes by besides the bat. In a pitch these follow from
				// play time alone, so a held or replayed moment shows the same pose.
				const bool bSwinging = !bReset && BatNote.bSwing && PlayTime >= BatNote.StartT && PlayTime <= BatNote.EndT;
				BatSwingW = bSwinging ? FMath::SmoothStep(BatNote.StartT, FMath::Max(BatNote.LaunchT, BatNote.StartT + 0.01f), PlayTime) : FMath::FInterpConstantTo(BatSwingW, 0.f, SimDt, 1.f / 0.45f);
				// He strides at every pitch, swing or no: up as the ball is let go, down 0.17 s on.
				BatStride = !bReset && PlayTime > -0.2f ? FMath::SmoothStep(-0.2f, 0.17f, PlayTime) : FMath::FInterpConstantTo(BatStride, 0.f, SimDt, 2.f);
				const bool bEase = bReset ? ResetDuration - ResetTime >= SetLeadS : BatNote.bSwing && PlayTime > BatNote.EndT + 0.3f;
				BatRest = FMath::FInterpConstantTo(BatRest, bEase ? 1.f : 0.f, SimDt, 2.f);
				// His eyes: on the pitcher, then on the ball (not all the way back into the mitt).
				const FVector See = bReset || PlayTime < 0.f || !bBallShown ? ToUE(0.0, 17.0, 1.8) : ToUE(Frame.ball[0], FMath::Max(Frame.ball[1], -0.5f), Frame.ball[2]);
				BatLook = BatLook.IsZero() ? See : FMath::VInterpTo(BatLook, See, SimDt, 12.f);
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
						if (ResetThrower >= 0 && I != ResetThrower && (A.agent & 0xFF) == 0x81)
						{
							// The pitcher takes the ball thrown back to him.
							const float Dt = ResetTime - (ResetThrowAt + ResetFlight);
							if (Dt >= -0.45f && Dt <= GloveInS + 0.03f)
							{
								In.bCatch = true;
								In.CatchDt = Dt;
								In.CatchAt = ToUE(ResetBallTo.X, ResetBallTo.Y, ResetBallTo.Z);
							}
						}
						if (I == ResetThrower)
						{
							const float Tt = ResetTime - ResetThrowAt;
							if (Tt >= BbMotion::ThrowStartS && Tt <= BbMotion::ThrowEndS)
							{
								In.bThrow = true;
								In.ThrowT = Tt;
								In.bThrowFrom = true;
								In.ThrowFrom = ToUE(ResetHand.X, ResetHand.Y, ResetHand.Z);
							}
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
						// The sim has a diving fielder take the ball at the moment he leaves his
						// feet and travel after it; a man laid out has left his feet already. So
						// the drawn dive runs DiveLeadS ahead of the sim's: he is in the air, the
						// glove on the ball, as the sim has it caught.
						for (uint32 D = 0; D < Frame.n_dives; ++D)
						{
							const BbDive& Dv = Frame.dives[D];
							if (Dv.agent == A.agent)
							{
								In.DiveSince = Dv.since_s + DiveLeadS;
								In.DiveUpIn = Dv.up_in_s;
								In.bDiveDir = Dv.dir[0] != 0.f || Dv.dir[1] != 0.f;
								In.DiveYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
							}
						}
						BbFrame Ahead;
						if (In.DiveSince < 0.f && bFigure && PlayTime + DiveLeadS <= Current.Info.duration_s && SampleDrawn(Current, PlayTime + DiveLeadS, Ahead))
						{
							for (uint32 D = 0; D < Ahead.n_dives; ++D)
							{
								const BbDive& Dv = Ahead.dives[D];
								if (Dv.agent == A.agent)
								{
									In.DiveSince = Dv.since_s;
									In.DiveUpIn = Dv.up_in_s + DiveLeadS;
									In.bDiveDir = Dv.dir[0] != 0.f || Dv.dir[1] != 0.f;
									In.DiveYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
								}
							}
						}
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
							const float Tt = PlayTime - N.ReleaseAt;
							if (N.Agent == A.agent && Tt >= BbMotion::ThrowStartS && Tt <= BbMotion::ThrowEndS)
							{
								In.bThrow = true;
								In.ThrowT = Tt;
								In.ThrowYaw = N.Yaw;
								In.bThrowFrom = N.bFrom;
								In.ThrowFrom = N.From;
							}
						}
						// The catch of his nearest in time.
						for (const FCatchNote& N : Catches)
						{
							const float Dt = PlayTime - N.T;
							if (N.Agent == A.agent && Dt >= -0.45f && Dt <= GloveInS + 0.03f && (!In.bCatch || FMath::Abs(Dt) < FMath::Abs(In.CatchDt)))
							{
								In.bCatch = true;
								In.CatchDt = Dt;
								In.CatchAt = N.At;
							}
						}
					}
					for (const FSlideNote& N : Slides)
					{
						const float Tt = PlayTime - N.T;
						if (bShow && !bReset && N.Agent == A.agent && Tt >= BbMotion::SlideStartS && Tt <= BbMotion::SlideEndS)
						{
							In.bSlide = true;
							In.SlideT = Tt;
							In.SlideYaw = N.Yaw;
						}
					}
					In.bHolds = bShow && Frame.holder == (int32)A.agent;
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
				// Everyone stands on the ground that is drawn, wherever the play or the reset
				// has put him: on the mound that is up to 10 in above the sim's level field.
				A.pos[2] += (float)Park->GroundZ(A.pos[0], A.pos[1]);
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
						FigureState[I].bHand = false;
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
			if (Crew)
			{
				// The people who are not playing: told where the play has got to, who is at bat
				// and behind the plate, and who is out on the field (so not on his bench).
				FBbCrewTick Ct;
				Ct.PlayTime = PlayTime;
				Ct.Dt = SimDt;
				Ct.bPlay = !bReset;
				Ct.bSet = bReset ? ResetDuration - ResetTime < SetLeadS : PlayTime < (Current.Info.contact_s >= 0.f ? Current.Info.contact_s : PitchEndT >= 0.f ? PitchEndT : 0.5f);
				Ct.bBall = bBallShown;
				Ct.Ball = FVector(Frame.ball[0], Frame.ball[1], Frame.ball[2]);
				Ct.Batting = Current.bValid ? (Current.Info.top ? 0 : 1) : 0;
				TSet<uint32> Shown;
				double BatterAt = 4.0;
				for (uint32 K = 0; K < Frame.n_actors; ++K)
				{
					const BbActor& A = Frame.actors[K];
					const FVector2D P(A.pos[0], A.pos[1]);
					if ((A.agent & 0x80) == 0)
					{
						Ct.Batting = A.agent >> 8;
						if (P.Size() < BatterAt)
						{
							BatterAt = P.Size();
							Ct.BatterX = P.X;
						}
					}
				}
				for (int32 I = 0; I < FigureState.Num() && I < Figures.Num(); ++I)
				{
					const BbPlayer* P = FigureState[I].Agent != 0xFFFF && !Figures[I]->IsHidden() ? Sim->Player(FigureState[I].Agent) : nullptr;
					if (P)
					{
						Shown.Add(P->player_id);
					}
				}
				Ct.Shown = &Shown;
				Crew->Tick(Ct);
			}
			if (Skel.Ok())
			{
				// The bat is drawn while someone holds it, and through the swing wherever the
				// sim has it. A batter who has left the plate has dropped it.
				const bool bShowBat = bBatHeld || (bBatSim && (BatNote.bSwing || PlayTime < 0.5f));
				Bat->SetActorHiddenInGame(!bShowBat);
				if (bShowBat)
				{
					PlaceBat(BatKnob, BatTip);
				}
				if (!bBatHeld)
				{
					BatHolder = 0xFFFF;
				}
			}
			// In play: the ball has been hit and is heading out (contact telemetry present
			// and the ball is in front of the plate after the pitch has arrived).
			if (Current.Info.exit_velocity_mps > 0.f && T > 0.35f && Frame.ball[1] > 1.5f)
			{
				bBallInPlay = true;
			}
			// A held ball is drawn in the holder's hand, not at the sim's holding point over his
			// feet (owner, 2026-11-15: the ball hung in the air through the catcher's throw). The
			// sim's ball is still the truth: the drawn one eases into the glove when he takes it,
			// and it leaves his hand on the sim's path, his hand being on the release point.
			FVector BallAt = ToUE(Frame.ball);
			bool bInHand = false;
			if (Frame.holder >= 0)
			{
				for (int32 I = 0; I < FigureState.Num() && I < Figures.Num(); ++I)
				{
					if (FigureState[I].Agent == (uint16)Frame.holder && Figures[I] && !Figures[I]->IsHidden())
					{
						// Where his pose has it; failing a pose, his throwing hand. (The stock
						// throw clip swings the left arm, mirrored for a right-hander; with the
						// coded motions a left-hander is in a mirror, so it is the right bone.)
						const bool bLeftBone = Skel.Ok() ? false : FigureState[I].Manual == AnimThrow ? bThrowClipLeftArm : Sim->ThrowsLeft((uint16)Frame.holder);
						const FVector Hand = FigureState[I].bHand ? FigureState[I].Hand : Figures[I]->GetSkeletalMeshComponent()->GetSocketLocation(bLeftBone ? TEXT("hand_l") : TEXT("hand_r"));
						HeldOffset = Hand - BallAt;
						bInHand = true;
						if (FigureState[I].bHand)
						{
							// (His glove was on the ball as he took it: nothing to ease.)
							HeldBlend = 1.f;
						}
						break;
					}
				}
			}
			else if (!bReset)
			{
				// The last step into a glove, on the ball's own line (FCatchNote::Prev).
				for (const FCatchNote& N : Catches)
				{
					if (PlayTime > N.T - SimTickS && PlayTime < N.T)
					{
						BallAt = FMath::Lerp(N.Prev, N.At, (double)((PlayTime - (N.T - SimTickS)) / SimTickS));
					}
				}
			}
			HeldBlend = FMath::Clamp(HeldBlend + (bInHand ? SimDt / HeldTakeS : -SimDt / HeldLetGoS), 0.f, 1.f);
			if (!bInHand && Skel.Ok() && HeldBlend > 0.f)
			{
				// Just thrown by a coded throw: nothing to ease, the hand was where the flight starts.
				bool bThrown = bReset && ResetThrower >= 0 && ResetTime >= ResetThrowAt;
				for (const FThrowNote& N : Throws)
				{
					bThrown = bThrown || (!bReset && N.bFrom && PlayTime >= N.ReleaseAt && PlayTime - N.ReleaseAt < 0.25f);
				}
				if (bThrown)
				{
					HeldBlend = 0.f;
				}
			}
			if (!bBallShown)
			{
				HeldBlend = 0.f;
			}
			BallAt += HeldOffset * HeldBlend;
			Ball->SetActorLocation(BallAt);
			Ball->SetActorHiddenInGame(!bBallShown);
			// Through the wind-up the ball is in the pitcher's hand, where his pose has it.
			for (int32 I = 0; !bReset && PlayTime < 0.f && I < MaxActors && I < FigureState.Num(); ++I)
			{
				if (FigureState[I].bHand && FigureState[I].Agent != 0xFFFF)
				{
					Ball->SetActorLocation(FigureState[I].Hand);
					Ball->SetActorHiddenInGame(false);
				}
			}
			UpdateCamera(Frame, DeltaSeconds);
			if (bMotionShots && !bReset && MotionStrip >= 3 && MotionStrip <= 5)
			{
				// The batter: from across the plate, in front of him (a ball hit, a pitch taken);
				// from the mound's side of him (a swing and a miss).
				const double Side = BatNote.bLeftBox ? 1.0 : -1.0;
				const FVector Eye = MotionStrip == 4 ? ToUE(-Side * 1.3, 5.2, 1.5) : ToUE(-Side * 3.4, 0.9, 1.25);
				const FVector Look = ToUE(BatNote.Spot.X * (MotionStrip == 4 ? 0.7 : 1.0), BatNote.Spot.Y, MotionStrip == 4 ? 0.85 : 1.0);
				Camera->SetActorLocationAndRotation(Eye, (Look - Eye).Rotation());
				Camera->GetCameraComponent()->SetFieldOfView(MotionStrip == 4 ? 36.f : 48.f);
				ShownCam = -1;
			}
			else if (bMotionShots && !bReset && MotionStrip >= 6 && Catches.IsValidIndex(StripCatch))
			{
				// The field. The camera goes with the man, 6 m off: in front of him and a little
				// to his glove side until he has the ball (so the glove shows), then on the side
				// his chest turns to as he throws.
				const FCatchNote& Cn = Catches[StripCatch];
				FVector Feet = FVector(Cn.At.X, Cn.At.Y, 0.0);
				for (uint32 I = 0; I < Frame.n_actors; ++I)
				{
					if (Frame.actors[I].agent == Cn.Agent)
					{
						Feet = ToUE(Frame.actors[I].pos);
					}
				}
				const double Side = Sim->ThrowsLeft(Cn.Agent) ? -1.0 : 1.0;
				FVector Eye;
				if (MotionStrip == 6 && Throws.IsValidIndex(StripThrow) && PlayTime > Cn.T + 0.1f)
				{
					const FVector Line = FRotator(0.f, Throws[StripThrow].Yaw, 0.f).Vector();
					Eye = Feet + FVector(-Line.Y, Line.X, 0.0) * (600.0 * Side) + Line * 120.0 + FVector(0.0, 0.0, 140.0);
				}
				else
				{
					// Where the ball comes from: along its line if that has any length on the
					// ground (a throw, a grounder), else from the plate (a fly).
					FVector Line = (Cn.At - Cn.Prev).GetSafeNormal2D();
					if ((Cn.At - Cn.Prev).Size2D() < 0.3 * (Cn.At - Cn.Prev).Size())
					{
						Line = (Feet - ToUE(0.0, 0.0, 0.0)).GetSafeNormal2D();
					}
					Eye = Feet - Line * 500.0 - FVector(-Line.Y, Line.X, 0.0) * (330.0 * Side) + FVector(0.0, 0.0, 150.0);
				}
				const FVector Look = Feet + FVector(0.0, 0.0, 105.0);
				Camera->SetActorLocationAndRotation(Eye, (Look - Eye).Rotation());
				Camera->GetCameraComponent()->SetFieldOfView(40.f);
				ShownCam = -1;
			}
			else if (bMotionShots && !bReset && MotionStrip >= 0)
			{
				// Beside the pitcher (the third-base side, which a right-hander's chest faces),
				// then from the plate, then beside the catcher, away from the batter.
				FVector Eye = ToUE(-7.5, 17.4, 1.5), Look = ToUE(0.0, 17.2, 1.05);
				float Fov = 34.f;
				if (Current.Info.play_index == 2)
				{
					Eye = ToUE(0.9, 8.6, 1.3);
					Look = ToUE(0.0, 16.9, 0.95);
					Fov = 26.f;
				}
				else if (Current.Info.play_index == 3)
				{
					float BatterX = 1.f;
					for (uint32 I = 0; I < Frame.n_actors; ++I)
					{
						if ((Frame.actors[I].agent & 0x80) == 0 && FMath::Abs(Frame.actors[I].pos[1]) < 1.5f)
						{
							BatterX = Frame.actors[I].pos[0];
						}
					}
					Eye = ToUE(BatterX > 0.f ? -2.4 : 2.4, 1.5, 1.1);
					Look = ToUE(0.0, -1.5, 0.6);
					Fov = 42.f;
				}
				Camera->SetActorLocationAndRotation(Eye, (Look - Eye).Rotation());
				Camera->GetCameraComponent()->SetFieldOfView(Fov);
				ShownCam = -1;
			}
			// -BbSlideShots: the camera beside the bag from a little before he slides.
			if (!bReset && Slides.IsValidIndex(ShotSlide) && PlayTime > Slides[ShotSlide].T - 1.f)
			{
				const FSlideNote& N = Slides[ShotSlide];
				const FVector Dir = FRotator(0.f, N.Yaw, 0.f).Vector();
				const FVector Eye = N.At - Dir * 120.f + FVector(-Dir.Y, Dir.X, 0.f) * 520.f + FVector(0, 0, 170.f);
				Camera->SetActorLocationAndRotation(Eye, (N.At - Dir * 60.f + FVector(0, 0, 60.f) - Eye).Rotation());
				Camera->GetCameraComponent()->SetFieldOfView(50.f);
				ShownCam = -1;
			}
			// -BbDiveShots: the camera on the diver, from his side, from a little before he goes.
			if (!bReset && ShotDiver >= 0 && PlayTime > ShotDiveFrom - 0.7f && PlayTime < ShotDiveFrom + 2.5f)
			{
				for (uint32 I = 0; I < Frame.n_actors; ++I)
				{
					if (Frame.actors[I].agent == ShotDiver)
					{
						const FVector P = ToUE(Frame.actors[I].pos);
						const FVector Dir = FRotator(0.f, ShotDiveYaw, 0.f).Vector();
						const FVector Eye = P + FVector(-Dir.Y, Dir.X, 0.f) * 560.f + Dir * 60.f + FVector(0, 0, 170.f);
						Camera->SetActorLocationAndRotation(Eye, (P + Dir * 30.f + FVector(0, 0, 50.f) - Eye).Rotation());
						Camera->GetCameraComponent()->SetFieldOfView(50.f);
						ShownCam = -1;
					}
				}
			}
			// After the camera has moved, since the shadow is sized for it.
			PlaceBallShadow(Frame, bBallShown);
		}
		if (ABbHUD* Hud = GetWorld()->GetFirstPlayerController() ? Cast<ABbHUD>(GetWorld()->GetFirstPlayerController()->GetHUD()) : nullptr)
		{
			const BbPlayInfo& In = Current.Info;
			// The home-run card: up as the camera cuts to the trot, fading in over 0.4 s, down
			// as the reset begins.
			Hud->HomeRunName = HrName;
			Hud->HomeRunLine = HrLine;
			Hud->HomeRunKind = HrKind;
			Hud->HomeRunSide = Before.top ? 0 : 1;
			Hud->HomeRunAlpha = bHomeRunCut && !bReset ? FMath::Clamp((PlayTime - In.dead_s) / 0.4f, 0.f, 1.f) : 0.f;
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
			Hud->Replay = bReplay ? FString::Printf(TEXT("REPLAY    seed %llu    play %u"), SeedShown, Current.Info.play_index) : FString();
			Hud->Notice = Wall < SavedUntil ? SavedNote : FString();
			Hud->Footer = FString::Printf(TEXT("Space: %s    Speed %.2fx: 1-5 for 1/2/4/6/10x, - / = by 0.25x    Enter: %s    R / F: replay    S: save play    C: camera (%s)    Q: quit"), bPaused ? TEXT("paused") : TEXT("pause"), Speed, bRush ? TEXT("skipping") : TEXT("next pitch"), CamNames[CameraMode]);
			// The outs on the board are the ones called so far. Runs, count and bases change
			// when the play is known. A play that ends the half inning leaves it on the board,
			// at three outs, until the next pitch (BeginPlayback).
			{
				const BbPlayInfo& Src = bKnown ? In : Before;
				const bool bHalfOver = In.inning != Before.inning || In.top != Before.top;
				const BbPlayInfo& Half = bHalfOver ? Before : Src;
				const int32 Outs = FMath::Min<int32>(Before.outs + Called, 3);
				FBbBugInfo& Bug = Hud->Bug;
				Bug.bValid = true;
				Bug.Runs[0] = Src.away_runs;
				Bug.Runs[1] = Src.home_runs;
				Bug.Inning = Half.inning;
				Bug.bTop = Half.top != 0;
				Bug.Outs = Outs;
				Bug.Balls = bHalfOver && bKnown ? 0 : Src.balls;
				Bug.Strikes = bHalfOver && bKnown ? 0 : Src.strikes;
				Bug.Bases = Src.bases;
				// The board in the outfield shows the same, with the line score read from the
				// sim's log once a pitch.
				const uint32 Through = In.play_index + (bKnown ? 1u : 0u);
				if (Sim && BoardThrough != Through)
				{
					bBoardValid = Sim->BoxScore(Through, BoardBox);
					BoardThrough = Through;
				}
				// The pitcher and the batter under the score (owner, 2026-11-16): names and
				// numbers from the roster, the pitch count and the batter's game from the box
				// score through what has been shown. The pitch in flight counts once thrown.
				Bug.Pitcher.Reset();
				Bug.Batter.Reset();
				if (bBoardValid)
				{
					const int32 Fielding = Half.top ? 1 : 0;
					const int32 NP = BoardBox.n_pitchers[Fielding];
					if (NP > 0)
					{
						const BbPitchLine& P = BoardBox.pitchers[Fielding][NP - 1];
						Bug.Pitcher = FString(UTF8_TO_TCHAR(P.family)).ToUpper();
						Bug.PitcherNumber = P.number;
						Bug.Pitches = P.pitches + (!bKnown && PlayTime >= 0.f ? 1 : 0);
					}
					const BbBoxTeam& Batting = BoardBox.teams[1 - Fielding];
					for (uint32 I = 0; I < Batting.n_lines; ++I)
					{
						const BbBatLine& L = Batting.lines[I];
						if (L.agent != In.batter)
						{
							continue;
						}
						static const TCHAR* Pos[] = { TEXT(""), TEXT("P"), TEXT("C"), TEXT("1B"), TEXT("2B"), TEXT("3B"), TEXT("SS"), TEXT("LF"), TEXT("CF"), TEXT("RF") };
						Bug.Batter = FString(UTF8_TO_TCHAR(L.family)).ToUpper();
						Bug.BatterNumber = L.number;
						Bug.BatterPosition = L.position <= 9 ? Pos[L.position] : TEXT("");
						// "1-2, HR, 2 RBI": hits for at-bats, then what stands out.
						FString Day;
						if (L.pa == 0)
						{
							Day = TEXT("first time up");
						}
						else
						{
							Day = FString::Printf(TEXT("%d-%d"), L.h, L.ab);
							auto Add = [&Day](int32 N, const TCHAR* What)
							{
								if (N > 0)
								{
									Day += N > 1 ? FString::Printf(TEXT(", %d %s"), N, What) : FString::Printf(TEXT(", %s"), What);
								}
							};
							Add(L.hr, TEXT("HR"));
							Add(L.triples, TEXT("3B"));
							Add(L.doubles, TEXT("2B"));
							Add(L.rbi, TEXT("RBI"));
							Add(L.bb, TEXT("BB"));
							Add(L.k, TEXT("K"));
						}
						Bug.BatterToday = Day;
					}
				}
				if (bBoardValid)
				{
					Park->ShowBoard(BoardBox, Half.inning, Half.top != 0, bHalfOver && bKnown ? 0 : Src.balls, bHalfOver && bKnown ? 0 : Src.strikes, Outs);
				}
			}
			const float Mph = In.release_speed_mps * 2.23694f;
			// Contact measurements only once the bat has met the ball: before that they would
			// say he is going to swing and hit it.
			// The pitch's speed and type once it has reached the plate (or the bat), not
			// before: then the ball off the bat once it is hit.
			static const TCHAR* Types[] = { TEXT(""), TEXT("Four-seam"), TEXT("Sinker"), TEXT("Cutter"), TEXT("Slider"), TEXT("Sweeper"), TEXT("Slurve"), TEXT("Curveball"), TEXT("Knuckle curve"), TEXT("Changeup"), TEXT("Splitter") };
			const TCHAR* Type = In.pitch_type <= 10 ? Types[In.pitch_type] : TEXT("");
			const float Arrived = In.contact_s >= 0.f ? In.contact_s : In.call_s - 0.2f;
			if (PlayTime < Arrived)
			{
				Hud->PitchLine.Reset();
			}
			else if (In.exit_velocity_mps > 0.f && In.contact_s >= 0.f)
			{
				Hud->PitchLine = FString::Printf(TEXT("%.0f mph %s    Off the bat %.0f mph, %.0f deg"), Mph, Type, In.exit_velocity_mps * 2.23694f, FMath::RadiansToDegrees(In.launch_angle_rad));
			}
			else
			{
				Hud->PitchLine = FString::Printf(TEXT("%.0f mph %s"), Mph, Type);
			}
		}
		if (bReplay && ReplayIdx >= History.Num() && PlayTime >= Current.Info.duration_s)
		{
			// The replay has run through the live pitch: the game carries on.
			bReplay = false;
			UE_LOG(LogBb, Display, TEXT("BB_REPLAY end play=%u"), Current.Info.play_index);
		}
		if (bReplay && ReplayIdx < History.Num())
		{
			// A replayed pitch runs straight on into the next one.
			if (PlayTime >= Current.Info.duration_s + HoldAfter)
			{
				ShowReplay(ReplayIdx + 1);
			}
		}
		else if (bReset)
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
				bSideCutPlanned = false;
				if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
				{
					if (PC->PlayerCameraManager)
					{
						PC->PlayerCameraManager->SetManualCameraFade(0.f, FLinearColor::Black, false);
					}
					if (ABbHUD* CardHud = Cast<ABbHUD>(PC->GetHUD()))
					{
						CardHud->CardAlpha = 0.f;
						CardHud->Card.Reset();
					}
				}
				BeginPlayback(MoveTemp(NextPlay));
				NextPlay = FBbPlay();
			}
		}
		else if (PlayTime >= Current.Info.duration_s + HoldAfter && NextPlay.bValid)
		{
			// The next pitch was already in hand when a replay began: stage the walk to it again.
			if (!bFoulOut && StartReset(NextPlay))
			{
				bReset = true;
				ResetTime = 0.f;
			}
			else
			{
				BeginPlayback(MoveTemp(NextPlay));
				NextPlay = FBbPlay();
			}
		}
		else if (PlayTime >= Current.Info.duration_s + HoldAfter && bPending && Pending.IsReady())
		{
			FBbPlay Next = Pending.Get();
			bPending = false;
			if (Next.bValid && !bFoulOut && StartReset(Next))
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
	// -BbParkShots: the camera goes round the park's fixed spots, a picture from each.
	if (ParkShot >= 0 && Sim && Wall >= ParkShotsAt)
	{
		const TArray<FBbParkSpot> Spots = ABbPark::Spots(Sim->Park());
		if (ParkShot >= Spots.Num())
		{
			if (Wall >= ParkShotSince + 1.0)
			{
				Finish();
			}
		}
		else
		{
			const FBbParkSpot& Spot = Spots[ParkShot];
			Camera->SetActorLocationAndRotation(Spot.Eye, Spot.Look);
			Camera->GetCameraComponent()->SetFieldOfView(Spot.Fov);
			ShownCam = -1;
			if (ParkShotSince == 0.0)
			{
				ParkShotSince = Wall;
			}
			if (!bParkShotAsked && Wall >= ParkShotSince + 1.0)
			{
				FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), FString::Printf(TEXT("park_%d_%s.png"), ParkShot + 1, Spot.Name)), false, false);
				bParkShotAsked = true;
			}
			else if (bParkShotAsked && Wall >= ParkShotSince + 1.5)
			{
				++ParkShot;
				ParkShotSince = Wall;
				bParkShotAsked = false;
			}
		}
	}
	// -BbCrewShots: the crew alone, each arrangement held a while and photographed.
	if (CrewShot >= 0 && Wall >= 8.0)
	{
		static const TCHAR* const Names[10] = { TEXT("plate_set"), TEXT("strike"), TEXT("out"), TEXT("safe"), TEXT("foul"), TEXT("base_set"), TEXT("coach"),
			TEXT("bench_home"), TEXT("bench_away"), TEXT("all") };
		if (ParkShotSince == 0.0)
		{
			ArrangeCrewShot(CrewShot);
			ParkShotSince = Wall;
		}
		CrewShotTick.Dt = DeltaSeconds;
		Crew->Tick(CrewShotTick);
		if (!bParkShotAsked && Wall >= ParkShotSince + 1.5)
		{
			FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), FString::Printf(TEXT("crew_%02d_%s.png"), CrewShot + 1, Names[CrewShot])), false, false);
			bParkShotAsked = true;
		}
		else if (bParkShotAsked && Wall >= ParkShotSince + 2.0)
		{
			bParkShotAsked = false;
			ParkShotSince = 0.0;
			if (++CrewShot == 10)
			{
				CrewShot = -1;
				Finish();
			}
		}
	}
	// -BbFigureShots: each arrangement of figures stands a second, is photographed, and
	// gives way to the next.
	if (FigureShot >= 0 && Wall >= 8.0)
	{
		static const TCHAR* const Names[9] = { TEXT("away_front"), TEXT("home_front"), TEXT("away_back"), TEXT("home_back"), TEXT("turn"), TEXT("turn_mirrored"),
			TEXT("kit_field"), TEXT("kit_field_mirrored_back"), TEXT("kit_bat") };
		if (ParkShotSince == 0.0)
		{
			ArrangeFigureShot(FigureShot);
			ParkShotSince = Wall;
		}
		if (!bParkShotAsked && Wall >= ParkShotSince + 1.0)
		{
			FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), FString::Printf(TEXT("figure_%d_%s.png"), FigureShot + 1, Names[FigureShot])), false, false);
			bParkShotAsked = true;
		}
		else if (bParkShotAsked && Wall >= ParkShotSince + 1.5)
		{
			bParkShotAsked = false;
			ParkShotSince = 0.0;
			if (++FigureShot == 9)
			{
				FigureShot = -1;
				Finish();
			}
		}
	}
	UpdateBoxScreen(DeltaSeconds, bTabHeld);
	// A game that has ended leaves the final box up: an unattended run waits for it to settle
	// (BoxHoldS) before it quits; a player quits with Esc or Q.
	const bool bBoxSettled = !bBoxFinal || BoxFinalTime >= BoxHoldS;
	if ((BenchSeconds > 0.f && Wall >= BenchSeconds) || (bGameOver && bBoxSettled && (ShotsLeft > 0 || BenchSeconds > 0.f)))
	{
		Finish();
	}
	if (bBoxFinalShot && Wall >= BoxShotWall + 1.5f)
	{
		Finish();
	}
}

void ABbDirector::RefreshBox(uint32 Through)
{
	if (!Sim || (bBoxValid && BoxThrough == Through))
	{
		return;
	}
	bBoxValid = Sim->BoxScore(Through, Box);
	BoxThrough = Through;
	if (!bBoxValid)
	{
		UE_LOG(LogBb, Warning, TEXT("No box score from the sim library"));
	}
}

void ABbDirector::BoxTitles(FString& Title, FString& Subtitle) const
{
	const int32 Away = Box.teams[0].totals.r;
	const int32 Home = Box.teams[1].totals.r;
	if (Box.is_final)
	{
		Title = Away > Home ? FString::Printf(TEXT("AWAY %d, HOME %d"), Away, Home) : FString::Printf(TEXT("HOME %d, AWAY %d"), Home, Away);
		Subtitle = Box.innings == 9 ? FString(TEXT("FINAL")) : FString::Printf(TEXT("FINAL  -  %d INNINGS"), Box.innings);
	}
	else
	{
		Title = FString::Printf(TEXT("AWAY %d, HOME %d"), Away, Home);
		Subtitle = TEXT("GAME IN PROGRESS");
	}
}

void ABbDirector::UpdateBoxScreen(float DeltaSeconds, bool bTabHeld)
{
	APlayerController* PC = GetWorld()->GetFirstPlayerController();
	ABbHUD* Hud = PC ? Cast<ABbHUD>(PC->GetHUD()) : nullptr;
	if (!Hud)
	{
		return;
	}
	if (bBoxFinal)
	{
		// Fade to black, then the paper comes up on it. The sim has stopped; nothing else moves.
		BoxFinalTime += DeltaSeconds;
		if (PC->PlayerCameraManager)
		{
			PC->PlayerCameraManager->SetManualCameraFade(FMath::Clamp(BoxFinalTime / BoxFadeS, 0.f, 1.f), FLinearColor::Black, false);
		}
		BoxTitles(Hud->BoxTitle, Hud->BoxSubtitle);
		Hud->Box = Box;
		Hud->bBoxData = bBoxValid;
		Hud->BoxDim = 1.f;
		Hud->BoxAlpha = FMath::Clamp((BoxFinalTime - BoxFadeS * 0.8f) / BoxInS, 0.f, 1.f);
		if (BoxShotAtPlay >= 0 && !bBoxFinalShot && BoxFinalTime >= BoxFadeS + BoxInS + 0.8f)
		{
			FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), TEXT("box_final.png")), true, false);
			bBoxFinalShot = true;
			BoxShotWall = (float)Wall;
		}
		return;
	}
	// Unattended: hold the Tab view once, at pitch BoxShotAtPlay.
	bool bWant = bTabHeld;
	if (BoxShotAtPlay >= 0 && !bBoxMidShot && PlaysShown >= BoxShotAtPlay && !bGameOver)
	{
		bWant = true;
		if (BoxShotWall < 0.f)
		{
			BoxShotWall = (float)Wall;
		}
		else if (Wall >= BoxShotWall + 0.8f)
		{
			FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), TEXT("box_midgame.png")), true, false);
			bBoxMidShot = true;
			BoxShotWall = -1.f;
		}
	}
	bWant = bWant && Current.bValid && Sim;
	BoxPeek = FMath::Clamp(BoxPeek + (bWant ? 1.f : -1.f) * DeltaSeconds / BoxPeekS, 0.f, 1.f);
	if (bWant)
	{
		// The pitches that have played out on screen; the worker is a pitch ahead and the
		// box must not be.
		RefreshBox(Current.Info.play_index + (PlayTime >= Current.Info.duration_s ? 1u : 0u));
	}
	Hud->BoxAlpha = bBoxValid ? BoxPeek : 0.f;
	Hud->BoxDim = 0.62f;
	if (BoxPeek > 0.f && bBoxValid)
	{
		BoxTitles(Hud->BoxTitle, Hud->BoxSubtitle);
		Hud->Box = Box;
		Hud->bBoxData = true;
	}
	else
	{
		Hud->bBoxData = false;
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
		UE_LOG(LogBb, Display, TEXT("BB_STAGING speed=%.2f exits_arrived=%d exits_rejoined=%d exits_cut=%d exits_walking=%d throws_shown=%d throws_missed=%d mirrored_frames=%d"),
			Speed, ExitsArrived, ExitsRejoined, ExitsCut, Walking, ThrowsShown, ThrowsMissed, MirroredFrames);
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
	if (GEngine && GEngine->GameViewport)
	{
		if (SaveDialogLayer.IsValid())
		{
			GEngine->GameViewport->RemoveViewportWidgetContent(SaveDialogLayer.ToSharedRef());
		}
		if (SaveButtonLayer.IsValid())
		{
			GEngine->GameViewport->RemoveViewportWidgetContent(SaveButtonLayer.ToSharedRef());
		}
	}
	SaveDialogLayer.Reset();
	SaveDialog.Reset();
	SaveButtonLayer.Reset();
	if (bReplay)
	{
		// The pitch on screen may be one of the kept ones: release each blob once.
		Current = LivePlay;
	}
	for (FShown& Old : History)
	{
		if (Sim && Old.Play.Blob)
		{
			Sim->Release(Old.Play);
		}
	}
	History.Reset();
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
