#include "BbDirector.h"
#include "BbBaseMap.h"
#include <limits>
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
#include "HAL/FileManager.h"
#include "Misc/CommandLine.h"
#include "Misc/FileHelper.h"
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

namespace
{
	/** The value of -BbSavedPlay=<path> (or another option given as Name, e.g.
	 *  "-BbSavedPlayCheckFile="): true if the argument is there at all, with Out empty
	 *  when nothing follows the equals sign before the next space (FParse::Value would take the
	 *  next option for the value). A quoted value may hold spaces. */
	bool SavedPlayArgument(const TCHAR* Cmd, FString& Out, const TCHAR* Name = TEXT("-BbSavedPlay="))
	{
		Out.Reset();
		const FString All(Cmd);
		int32 At = All.Find(Name, ESearchCase::IgnoreCase);
		if (At == INDEX_NONE)
		{
			return false;
		}
		int32 I = At + FCString::Strlen(Name);
		if (I < All.Len() && All[I] == TEXT('"'))
		{
			++I;
			while (I < All.Len() && All[I] != TEXT('"'))
			{
				Out.AppendChar(All[I++]);
			}
		}
		else
		{
			while (I < All.Len() && !FChar::IsWhitespace(All[I]))
			{
				Out.AppendChar(All[I++]);
			}
		}
		return true;
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
	Camera->SetActorRotation((CamLook - Camera->GetActorLocation()).Rotation());
	if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
	{
		PC->SetViewTarget(Camera);
	}

	const TCHAR* Cmd = FCommandLine::Get();
	FString Lib = FPaths::ConvertRelativePathToFull(FPaths::Combine(FPaths::ProjectDir(), TEXT("../sim/target/release/libbb_ffi.so")));
	FParse::Value(Cmd, TEXT("BbSimLib="), Lib);
	SimLibPath = Lib;
	uint64 Seed = 7;
	FParse::Value(Cmd, TEXT("BbSeed="), Seed);
	// The headless check of saving and reopening plays: it needs no game of this director's,
	// only the library, and its exit status says whether it passed.
	{
		FString CheckFile;
		const bool bCheckFile = SavedPlayArgument(Cmd, CheckFile, TEXT("-BbSavedPlayCheckFile="));
		if (bCheckFile || FParse::Param(Cmd, TEXT("BbSavedPlayCheck")))
		{
			bGameOver = true;
			FPlatformMisc::RequestExitWithStatus(false, (uint8)RunSavedPlayCheck(Seed, bCheckFile, CheckFile));
			return;
		}
	}
	if (FParse::Param(Cmd, TEXT("BbBaseMapCheck")))
	{
		bGameOver = true;
		FPlatformMisc::RequestExitWithStatus(false, (uint8)RunBaseMapCheck(Seed));
		return;
	}
	// -BbSavedPlay=<toml>: an empty value is still saved mode (never a quiet live game): the
	// viewer then says what is missing. Argument present, so the play comes from the file alone.
	BbSavePlay::FView SavedView;
	FString SavedArg;
	if (SavedPlayArgument(Cmd, SavedArg))
	{
		bSavedPlay = true;
		BbSavePlay::BeginView(SavedArg, SavedView);
		SavedPlay = SavedView.Meta;
		if (SavedView.bRead && SavedView.Meta.bHasSeed)
		{
			Seed = SavedView.Meta.What.Seed;
		}
	}
	// The game is started here, before anything is drawn, because the park comes from it.
	TUniquePtr<FBbSim> Started = MakeUnique<FBbSim>();
	if (!Started->Start(Lib, Seed))
	{
		bGameOver = true;
		if (bSavedPlay)
		{
			// Nothing can be shown without the library, but say so on screen and in the log.
			bSavedPlayBroken = true;
			SavedErrorTitle = TEXT("SIM LIBRARY NOT LOADED");
			SavedErrorBody = FString::Printf(TEXT("Could not start the sim from %s, so this play cannot be shown.\nBuild it with: cd sim && cargo build --release -p bb_ffi"), *Lib);
			UE_LOG(LogBb, Error, TEXT("BB_SAVED_PLAY could not start the sim from %s"), *Lib);
		}
		return;
	}
	if (bSavedPlay && SavedView.ErrTitle.IsEmpty())
	{
		// Loads and checks the timeline and puts the recorded roster and park into the sim,
		// before the park is built from it.
		BbSavePlay::FinishView(*Started, SavedView);
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
	bSaveAtHandoff = FParse::Param(Cmd, TEXT("BbSaveAtHandoff"));
	SeedShown = Seed;
	FParse::Value(Cmd, TEXT("BbShotEvery="), ShotEvery);
	FParse::Value(Cmd, TEXT("BbShotStart="), NextShot);
	FParse::Value(Cmd, TEXT("BbBenchSeconds="), BenchSeconds);
	FParse::Value(Cmd, TEXT("BbDiveShots="), DiveShotsLeft);
	FParse::Value(Cmd, TEXT("BbDiveBackShots="), DiveBackShotsLeft);
	FParse::Value(Cmd, TEXT("BbSlideShots="), SlideShotsLeft);
	FParse::Value(Cmd, TEXT("BbBaseMapShots="), BaseMapShotsLeft);
	FParse::Value(Cmd, TEXT("BbBoxShots="), BoxShotAtPlay);
	FParse::Value(Cmd, TEXT("BbBoxHold="), BoxHoldS);
	FParse::Value(Cmd, TEXT("BbParkShotsAt="), ParkShotsAt);
	ParkShot = FParse::Param(Cmd, TEXT("BbParkShots")) ? 0 : -1;
	{
		int32 Cut = 1;
		FParse::Value(Cmd, TEXT("BbSideCut="), Cut);
		FParse::Value(Cmd, TEXT("BbSideCardHold="), SideCardHoldS);
		FParse::Value(Cmd, TEXT("BbCardEnterAt="), SideCardEnterAt);
		bSideCut = Cut != 0;
	}
	bStagingLog = FParse::Param(Cmd, TEXT("BbStagingLog"));
	bMotionShots = FParse::Param(Cmd, TEXT("BbMotionShots"));
	bBatCheck = FParse::Param(Cmd, TEXT("BbBatCheck"));
	bBatLegacy = FParse::Param(Cmd, TEXT("BbBatLegacy"));
	FParse::Value(Cmd, TEXT("BbBatShots="), BatShotsLeft);
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
	if (bSavedPlay)
	{
		SavedBefore = SavedView.Before;
		SavedWarning = SavedView.CurrentBuildNote;
		if (!SavedView.bOk)
		{
			bSavedPlayBroken = true;
			SavedErrorTitle = SavedView.ErrTitle;
			SavedErrorBody = SavedView.ErrBody;
			return;
		}
		Current = MoveTemp(SavedView.Play);
		RestartSavedPlay();
		UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY loaded seed=%llu play=%u commit=%s timeline=%s names_and_park=%s"),
			SavedPlay.What.Seed, Current.Info.play_index, SavedPlay.Commit.IsEmpty() ? TEXT("unknown") : *SavedPlay.Commit, *SavedPlay.TimelinePath,
			SavedWarning.IsEmpty() ? TEXT("recorded") : TEXT("current build"));
		return;
	}
	// The first pitch, like every other, resolves on a worker; Tick starts it when it is ready.
	RequestNext();
}

float ABbDirector::SideCutFade(float SimDt, float RealDt)
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
		// have him as the picture is due back; both then wait while the card is held.
		const float Skip = FMath::Max(0.f, BackAt - ResetTime);
		ResetTime += Skip;
		ExitClock += Skip;
		bSideCutJumped = true;
		bSideCardOn = SideCardHoldS > 0.f;
		bSideCardSkip = false;
		SideCardHeldS = 0.f;
		if (bStagingLog)
		{
			UE_LOG(LogBb, Display, TEXT("BB_SIDECUT play=%u skipped=%.2f of %.2f s back_at=%.2f"), NextPlay.Info.play_index, Skip, ResetDuration, BackAt);
		}
		return 1.f;
	}
	if (bSideCardOn)
	{
		// The card is held on real time: the speed keys and the rush do not shorten it.
		SideCardHeldS += bPaused ? 0.f : RealDt;
		if (SideCardEnterAt >= 0.f && SideCardHeldS >= SideCardEnterAt)
		{
			SideCardEnterAt = -1.f;
			bSideCardSkip = true;
		}
		if (bSideCardSkip || SideCardHeldS >= SideCardHoldS)
		{
			bSideCardOn = false;
			if (bStagingLog)
			{
				UE_LOG(LogBb, Display, TEXT("BB_CARD play=%u held=%.2f of %.1f s%s"), NextPlay.Info.play_index, SideCardHeldS, SideCardHoldS, bSideCardSkip ? TEXT(" (Enter)") : TEXT(""));
			}
		}
		else
		{
			// Everything stands still behind the black.
			ResetTime -= SimDt;
			ExitClock -= SimDt;
			return 1.f;
		}
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
	if (bSaveAtHandoff && PlaysShown == 3)
	{
		bSaveHandoffNow = true;
	}
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

void ABbDirector::RestartSavedPlay()
{
	if (!Current.bValid)
	{
		return;
	}
	Before = SavedBefore;
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
	UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY play=%u seed=%llu"), Current.Info.play_index, SeedShown);
}

namespace
{
	/** A BbSavePlay::FBoard from the sim's box score. */
	BbSavePlay::FBoard BoardOf(const BbBoxScore& Box)
	{
		BbSavePlay::FBoard B;
		B.bValid = true;
		B.FirstInning = Box.first_inning;
		for (int32 T = 0; T < 2; ++T)
		{
			B.Hits[T] = Box.teams[T].totals.h;
			const int32 N = FMath::Min<int32>(Box.teams[T].n_batted, BB_BOX_MAX_INNINGS);
			for (int32 I = 0; I < N; ++I)
			{
				B.Runs[T].Add(Box.teams[T].runs[I]);
			}
		}
		return B;
	}

	/** What a save of this play records: the board before the pitch (Before) and after it,
	 *  from the sim's box score of the plays through the one before and through this one. */
	BbSavePlay::FWhat WhatOf(const FBbSim& Sim, uint64 Seed, const BbPlayInfo& Info, const BbPlayInfo& Before)
	{
		BbSavePlay::FWhat What;
		What.Seed = Seed;
		What.Play = Info.play_index;
		What.Inning = Before.inning;
		What.bTop = Before.top != 0;
		What.Outs = Before.outs;
		What.Balls = Before.balls;
		What.Strikes = Before.strikes;
		What.AwayRuns = Before.away_runs;
		What.HomeRuns = Before.home_runs;
		What.bHasBases = true;
		What.Bases = Before.bases & 7;
		BbBoxScore Box{};
		if (Sim.BoxScore(Info.play_index, Box))
		{
			What.BoardBefore = BoardOf(Box);
		}
		if (Sim.BoxScore(Info.play_index + 1, Box))
		{
			What.BoardAfter = BoardOf(Box);
		}
		return What;
	}
}

void ABbDirector::SavedBoard(bool bKnown, const BbPlayInfo& Src, int32 Inning, BbBoxScore& Out, bool& bHitsKnown) const
{
	FMemory::Memzero(Out);
	Out.innings = static_cast<uint8>(FMath::Clamp(Inning, 1, 255));
	Out.first_inning = 1;
	// After the result the board shows the play's line score and hits; before it, the earlier
	// ones. A save that did not record them has an empty line score and no hit totals (a dash),
	// never a made-up zero.
	const BbSavePlay::FBoard& B = bKnown ? SavedPlay.What.BoardAfter : SavedPlay.What.BoardBefore;
	bHitsKnown = B.bValid;
	if (B.bValid)
	{
		Out.first_inning = static_cast<uint8>(FMath::Clamp(B.FirstInning, 1, 255));
		for (int32 T = 0; T < 2; ++T)
		{
			const int32 N = FMath::Min(B.Runs[T].Num(), BB_BOX_MAX_INNINGS);
			Out.teams[T].n_batted = static_cast<uint8>(N);
			for (int32 I = 0; I < N; ++I)
			{
				Out.teams[T].runs[I] = B.Runs[T][I];
			}
			Out.teams[T].totals.h = static_cast<uint16>(B.Hits[T]);
		}
		Out.innings = static_cast<uint8>(FMath::Max<int32>(Out.innings, FMath::Max(B.Runs[0].Num(), B.Runs[1].Num()) + B.FirstInning - 1));
	}
	Out.teams[0].totals.r = Src.away_runs;
	Out.teams[1].totals.r = Src.home_runs;
}

int32 ABbDirector::RunBaseMapCheck(uint64 Seed)
{
	FBbSim Game;
	if (!Game.Start(SimLibPath, Seed))
	{
		UE_LOG(LogBb, Error, TEXT("BB_BASE_MAP_CHECK FAIL: could not start the game sim"));
		return 1;
	}
	int32 Failures = 0, Cases = 0, Plays = 0, WithRunners = 0, MovedLit = 0, DotSamples = 0;
	auto Expect = [&](bool bOk, const FString& Why)
	{
		++Cases;
		if (!bOk && ++Failures <= 20)
		{
			UE_LOG(LogBb, Error, TEXT("BB_BASE_MAP_CHECK FAIL: %s"), *Why);
		}
		return bOk;
	};
	const BbPark& Ground = Game.Park();
	auto Pop = [](uint8 Bits) { return (int32)((Bits & 1) + ((Bits >> 1) & 1) + ((Bits >> 2) & 1)); };

	// 1. Fixed points of the projection: the bases' own sim coordinates land on the corners of
	//    the picture, and halfway along a path lands halfway (x toward first base, y toward second).
	{
		auto Near = [](const FVector2D& A, double X, double Y) { return FMath::Abs(A.X - X) < 1e-4 && FMath::Abs(A.Y - Y) < 1e-4; };
		const float* F1 = Ground.bases[0];
		const float* F2 = Ground.bases[1];
		const float* F3 = Ground.bases[2];
		Expect(Near(BbBaseMap::ToDiamond(0.f, 0.f, Ground), 0, 0), TEXT("home does not map to the picture's home"));
		Expect(Near(BbBaseMap::ToDiamond(F1[0], F1[1], Ground), 1, 1), TEXT("first base does not map to the picture's first-base corner"));
		Expect(Near(BbBaseMap::ToDiamond(F2[0], F2[1], Ground), 0, 2), TEXT("second base does not map to the picture's second-base corner"));
		Expect(Near(BbBaseMap::ToDiamond(F3[0], F3[1], Ground), -1, 1), TEXT("third base does not map to the picture's third-base corner"));
		Expect(Near(BbBaseMap::ToDiamond(F1[0] * 0.5f, F1[1] * 0.5f, Ground), 0.5, 0.5), TEXT("halfway to first does not map halfway"));
		Expect(Near(BbBaseMap::ToDiamond((F1[0] + F2[0]) * 0.5f, (F1[1] + F2[1]) * 0.5f, Ground), 0.5, 1.5), TEXT("halfway from first to second does not map halfway"));
		Expect(Near(BbBaseMap::ToDiamond((F2[0] + F3[0]) * 0.5f, (F2[1] + F3[1]) * 0.5f, Ground), -0.5, 1.5), TEXT("halfway from second to third does not map halfway"));
		Expect(Near(BbBaseMap::ToDiamond(F3[0] * 0.5f, F3[1] * 0.5f, Ground), -0.5, 0.5), TEXT("halfway from home to third does not map halfway"));
	}
	const double HomeToFirstM = FMath::Sqrt((double)Ground.bases[0][0] * Ground.bases[0][0] + (double)Ground.bases[0][1] * Ground.bases[0][1]);
	const double ScalePerM = FMath::Sqrt(2.0) / HomeToFirstM; // |(1, 1)| picture units per metre

	int32 PrevOuts = 0;
	uint8 PrevInning = 1, PrevTop = 1, PrevBases = 0;
	bool bPrevEndedHalf = false;
	for (int32 I = 0; I < 400; ++I)
	{
		FBbPlay Play = Game.Next();
		if (!Play.bValid)
		{
			break;
		}
		const BbPlayInfo& Info = Play.Info;
		// The board before the pitch: what the last play left, nothing in a new half inning.
		const bool bFresh = I == 0 || bPrevEndedHalf;
		const uint8 OnBefore = bFresh ? 0 : PrevBases;
		const int32 OutsBefore = bFresh ? 0 : PrevOuts;
		++Plays;
		uint8 Last = 0xFF;
		int32 LitChanges = 0;
		TMap<uint16, FVector2D> PrevDot;
		float PrevT = 0.f;
		// Every sample of the play, to judge the lit bases against each runner's own track.
		struct FTrack { float T; uint8 Lit; TMap<uint16, FVector2D> Pos; };
		TArray<FTrack> Track;
		for (int32 K = 0;; ++K)
		{
			const float T = FMath::Min(K * 0.0333333f, Info.duration_s);
			BbFrame F{};
			if (!Game.Sample(Play, T, F))
			{
				Expect(false, FString::Printf(TEXT("play %u: no frame at t=%.3f"), Info.play_index, T));
				break;
			}
			BbBaseMap::FMap M;
			BbBaseMap::Read(F, Ground, Info, OnBefore, OutsBefore, T, M);
			// Independent of the function: a dot is where the runner is, to scale. The picture's
			// distance from home is the runner's distance from home in the timeline times one
			// constant, and a runner moves no faster on the picture than he runs.
			int32 NRunners = 0;
			for (uint32 A = 0; A < F.n_actors; ++A)
			{
				NRunners += (F.actors[A].agent & 0x80) ? 0 : 1;
			}
			Expect(M.Dots.Num() <= NRunners && Pop(M.Lit) <= M.Dots.Num(), FString::Printf(TEXT("play %u t=%.2f: %d dots, %d lit, %d runners"), Info.play_index, T, M.Dots.Num(), Pop(M.Lit), NRunners));
			for (const BbBaseMap::FDot& D : M.Dots)
			{
				if (D.Agent == 0)
				{
					continue;
				}
				const BbActor* Who = nullptr;
				for (uint32 A = 0; A < F.n_actors; ++A)
				{
					Who = F.actors[A].agent == D.Agent ? &F.actors[A] : Who;
				}
				if (!Expect(Who != nullptr, FString::Printf(TEXT("play %u: a dot for agent %x who is not in the frame"), Info.play_index, D.Agent)))
				{
					continue;
				}
				++DotSamples;
				const double SimR = FMath::Sqrt((double)Who->pos[0] * Who->pos[0] + (double)Who->pos[1] * Who->pos[1]);
				Expect(FMath::Abs(D.At.Size() - SimR * ScalePerM) < 2e-3 * (1.0 + SimR * ScalePerM), FString::Printf(TEXT("play %u t=%.2f: dot %.3f from home, runner %.2f m out should be %.3f"), Info.play_index, T, D.At.Size(), SimR, SimR * ScalePerM));
				if (const FVector2D* Was = PrevDot.Find(D.Agent))
				{
					Expect(FVector2D::Distance(*Was, D.At) <= 12.0 * (T - PrevT) * ScalePerM + 1e-3, FString::Printf(TEXT("play %u t=%.2f: agent %x's dot jumped"), Info.play_index, T, D.Agent));
				}
			}
			{
				FTrack& Row = Track.AddDefaulted_GetRef();
				Row.T = T;
				Row.Lit = M.Lit;
				for (uint32 A = 0; A < F.n_actors; ++A)
				{
					if (!(F.actors[A].agent & 0x80))
					{
						Row.Pos.Add(F.actors[A].agent, FVector2D(F.actors[A].pos[0], F.actors[A].pos[1]));
					}
				}
			}
			// The umpire's out calls and a runner who has scored take their dots away.
			for (const BbBaseMap::FDot& D : M.Dots)
			{
				if (D.Agent == 0)
				{
					continue;
				}
				for (uint32 C = 0; C < Info.n_calls; ++C)
				{
					Expect(!(Info.calls[C].runner == D.Agent && T >= Info.calls[C].t_s), FString::Printf(TEXT("play %u t=%.2f: agent %x has a dot after he was called out at %.2f"), Info.play_index, T, D.Agent, Info.calls[C].t_s));
				}
				for (uint32 A = 0; A < F.n_actors; ++A)
				{
					if (F.actors[A].agent == D.Agent)
					{
						Expect(F.actors[A].pos[0] * F.actors[A].pos[0] + F.actors[A].pos[1] * F.actors[A].pos[1] > 1.f, FString::Printf(TEXT("play %u t=%.2f: agent %x has a dot at the plate"), Info.play_index, T, D.Agent));
					}
				}
			}
			PrevDot.Reset();
			for (const BbBaseMap::FDot& D : M.Dots)
			{
				PrevDot.Add(D.Agent, D.At);
			}
			PrevT = T;
			if (K == 0)
			{
				Expect(M.Lit == OnBefore && M.Dots.Num() == Pop(OnBefore), FString::Printf(TEXT("play %u: at the pitch the map shows %u with %d dots, the runners on were %u"), Info.play_index, M.Lit, M.Dots.Num(), OnBefore));
			}
			if (M.Lit != Last)
			{
				++LitChanges;
				Last = M.Lit;
			}
			if (T >= Info.duration_s)
			{
				Expect(M.Lit == (Info.bases & 7) && M.Dots.Num() == Pop(Info.bases & 7), FString::Printf(TEXT("play %u (%hs): at the end the map shows %u with %d dots (pitch_call %d, before %u), the result is %u"), Info.play_index, Info.summary, M.Lit, M.Dots.Num(), Info.pitch_call, OnBefore, Info.bases & 7));
				break;
			}
		}
		// A base stays lit while its runner is there: for each runner still on the field at the end
		// (not out, not scored, not a foul, not the half's end), by his own track and the base
		// distances in metres, never by the path rule under test. If he began the play at a base
		// and ended at that same base, it was lit at every sample; if he arrived at a base
		// (within a metre of it), it was lit at every sample from then on.
		if (!BbBaseMap::IsFoul(Info) && OutsBefore + (int32)Info.n_calls < 3 && Track.Num() > 1)
		{
			auto Near = [&](const FVector2D& P, int32 B, double R) { return FVector2D::Distance(P, FVector2D(Ground.bases[B][0], Ground.bases[B][1])) <= R; };
			for (const TPair<uint16, FVector2D>& R : Track.Last().Pos)
			{
				bool bGone = R.Value.Size() <= BbBaseMap::HomeHideM;
				for (uint32 C = 0; C < Info.n_calls; ++C)
				{
					bGone |= Info.calls[C].runner == R.Key;
				}
				for (int32 B = 0; B < 3 && !bGone; ++B)
				{
					if (!Near(R.Value, B, 7.0))
					{
						continue;
					}
					const FVector2D* Start = Track[0].Pos.Find(R.Key);
					int32 From = -1;
					if (Start && Near(*Start, B, 7.0))
					{
						From = 0;
					}
					else
					{
						for (int32 K = 0; K < Track.Num() && From < 0; ++K)
						{
							const FVector2D* At = Track[K].Pos.Find(R.Key);
							From = At && Near(*At, B, 1.0) ? K : From;
						}
					}
					for (int32 K = FMath::Max(From, 0); From >= 0 && K < Track.Num(); ++K)
					{
						if (!Expect(((Track[K].Lit >> B) & 1) != 0, FString::Printf(TEXT("play %u (%hs): base %d went dark at t=%.2f while its runner %x was there (reached it at sample %d)"), Info.play_index, Info.summary, B + 1, Track[K].T, R.Key, From)))
						{
							break;
						}
					}
				}
			}
		}
		if (OnBefore != 0 || (Info.bases & 7) != 0)
		{
			++WithRunners;
			MovedLit += LitChanges > 1 ? 1 : 0;
		}
		bPrevEndedHalf = Info.inning != PrevInning || Info.top != PrevTop;
		PrevInning = Info.inning;
		PrevTop = Info.top;
		PrevBases = Info.bases & 7;
		PrevOuts = Info.outs;
		Game.Release(Play);
	}
	UE_LOG(LogBb, Display, TEXT("BB_BASE_MAP_CHECK %s plays=%d with_runners=%d lit_set_changed_in=%d dot_samples=%d cases=%d failures=%d"), Failures == 0 ? TEXT("PASS") : TEXT("FAILED"), Plays, WithRunners, MovedLit, DotSamples, Cases, Failures);
	return Failures == 0 ? 0 : 1;
}

int32 ABbDirector::RunSavedPlayCheck(uint64 Seed, bool bFileOnly, const FString& File)
{
	// A saved play is opened here through BbSavePlay::BeginView and FinishView, the loader the
	// viewer (-BbSavedPlay) itself calls, on sims of this check's own.
	auto OpenFile = [this](const FString& Arg, BbSavePlay::FView& V, FBbSim& Viewer)
	{
		BbSavePlay::BeginView(Arg, V);
		if (V.ErrTitle.IsEmpty())
		{
			if (!Viewer.Start(SimLibPath, V.Meta.What.Seed))
			{
				V.ErrTitle = TEXT("SIM LIBRARY NOT LOADED");
				V.ErrBody = TEXT("The sim library could not be started.");
				return;
			}
			BbSavePlay::FinishView(Viewer, V);
		}
	};
	if (bFileOnly)
	{
		if (File.IsEmpty())
		{
			UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK_FILE REFUSED no file given: -BbSavedPlayCheckFile= needs a path to a .toml"));
			return 1;
		}
		BbSavePlay::FView V;
		FBbSim Lone;
		OpenFile(File, V, Lone);
		if (V.bOk)
		{
			UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK_FILE OPENED %s seed=%llu play=%u names_and_park=%s"), *V.TomlPath, V.Meta.What.Seed, V.Play.Info.play_index, V.CurrentBuildNote.IsEmpty() ? TEXT("recorded") : TEXT("current build"));
			Lone.Release(V.Play);
			return 0;
		}
		UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK_FILE REFUSED %s: %s"), *V.ErrTitle, *V.ErrBody.Replace(TEXT("\n"), TEXT(" ")));
		return 1;
	}

	const FString TempDir = FPaths::ConvertRelativePathToFull(
		FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("SavedPlayCheck"), FString::Printf(TEXT("run-%d"), FPlatformProcess::GetCurrentProcessId())));
	IFileManager::Get().MakeDirectory(*TempDir, true);
	int32 Failures = 0;
	auto Fail = [&](const FString& Why)
	{
		UE_LOG(LogBb, Error, TEXT("BB_SAVED_PLAY_CHECK FAIL: %s"), *Why);
		++Failures;
	};
	int32 Cases = 0;
	auto Expect = [&](bool bOk, const FString& What)
	{
		++Cases;
		if (!bOk)
		{
			Fail(What);
		}
		return bOk;
	};
	auto Finish = [&]() -> int32
	{
		IFileManager::Get().DeleteDirectory(*TempDir, false, true);
		if (Failures == 0)
		{
			UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK PASS commit=%s cases=%d"), *BbSavePlay::BuildCommit(), Cases);
			return 0;
		}
		UE_LOG(LogBb, Error, TEXT("BB_SAVED_PLAY_CHECK FAILED %d of %d checks"), Failures, Cases);
		return 1;
	};
	if (FPaths::IsUnderDirectory(TempDir, BbSavePlay::Folder()))
	{
		Fail(TEXT("the check's folder is inside saved-plays/"));
		return Finish();
	}

	// 1. A play with a ball in play and a runner on, from a game of this check's own, with the
	//    summary of the play before it (the board before the pitch).
	FBbSim Game;
	if (!Game.Start(SimLibPath, Seed))
	{
		Fail(TEXT("could not start the game sim"));
		return Finish();
	}
	BbPlayInfo BeforeInfo{};
	BeforeInfo.inning = 1;
	BeforeInfo.top = 1;
	FBbPlay Chosen;
	BbPlayInfo ChosenBefore = BeforeInfo;
	for (int32 I = 0; I < 60; ++I)
	{
		FBbPlay Next = Game.Next();
		if (!Next.bValid)
		{
			break;
		}
		const bool bPick = I >= 3 && Next.Info.exit_velocity_mps > 0.f && BeforeInfo.bases != 0;
		if (bPick || I == 59)
		{
			ChosenBefore = BeforeInfo;
			Chosen = Next;
			break;
		}
		BeforeInfo = Next.Info;
		Game.Release(Next);
	}
	if (!Expect(Chosen.bValid, TEXT("no play to save")))
	{
		return Finish();
	}
	UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK saving play=%u (runners before: %u, ball in play: %d)"), Chosen.Info.play_index, ChosenBefore.bases, Chosen.Info.exit_velocity_mps > 0.f ? 1 : 0);

	// 2. Save it with a roster and park that differ from today's (a new name, a hand turned
	//    round, a taller fence), reopen it through the viewer's loader on a fresh sim, and
	//    require every frame, the board fields and the recorded (not today's) names and park.
	BbSavePlay::FContext Context = BbSavePlay::ContextOf(Game);
	if (!Expect(Context.HasRoster() && Context.bHasPark, TEXT("the sim did not give 18 players and a park")))
	{
		Game.Release(Chosen);
		return Finish();
	}
	const uint16 ChangedAgent = Context.Roster[0].bat_agent;
	const uint16 ChangedField = Context.Roster[0].field_agent;
	const bool bWasLeft = Context.Roster[0].throws == 1;
	FCStringAnsi::Strncpy(Context.Roster[0].family, "Recorded", BB_NAME_LEN);
	Context.Roster[0].throws = bWasLeft ? 2 : 1;
	const float Fence = Context.Park.fence_height_m + 3.f;
	Context.Park.fence_height_m = Fence;
	const BbSavePlay::FWhat What = WhatOf(Game, Seed, Chosen.Info, ChosenBefore);
	const FString Comment = TEXT("Saved play check: \"quotes\", a back\\slash\nand a second line.");
	const FString Toml = BbSavePlay::WriteTo(TempDir, What, Context, Comment, Chosen.Blob, Chosen.Len);
	if (!Expect(!Toml.IsEmpty(), TEXT("WriteTo gave no file")))
	{
		Game.Release(Chosen);
		return Finish();
	}
	{
		BbSavePlay::FView V;
		FBbSim Viewer;
		OpenFile(Toml, V, Viewer);
		if (Expect(V.bOk, FString::Printf(TEXT("the saved play did not open: %s %s"), *V.ErrTitle, *V.ErrBody)))
		{
			Expect(V.CurrentBuildNote.IsEmpty(), TEXT("a save with names and park says they come from the current build"));
			Expect(V.Meta.Commit == BbSavePlay::BuildCommit(), FString::Printf(TEXT("commit '%s' is not the build's '%s'"), *V.Meta.Commit, *BbSavePlay::BuildCommit()));
			Expect(V.Meta.Comment == Comment, TEXT("the comment did not round-trip"));
			Expect(V.Meta.What.Seed == Seed && V.Meta.What.Play == Chosen.Info.play_index, TEXT("seed or play changed"));
			Expect(FMemory::Memcmp(&V.Play.Info, &Chosen.Info, sizeof(BbPlayInfo)) == 0, TEXT("the play's summary changed"));
			Expect(V.Before.inning == ChosenBefore.inning && V.Before.top == ChosenBefore.top && V.Before.outs == ChosenBefore.outs
				&& V.Before.balls == ChosenBefore.balls && V.Before.strikes == ChosenBefore.strikes
				&& V.Before.away_runs == ChosenBefore.away_runs && V.Before.home_runs == ChosenBefore.home_runs, TEXT("the board before the pitch changed"));
			Expect(V.Before.bases == (ChosenBefore.bases & 7), FString::Printf(TEXT("the runners on changed: saved %u, opened %u"), ChosenBefore.bases & 7, V.Before.bases));
			Expect(V.Meta.What.BoardBefore.bValid && V.Meta.What.BoardAfter.bValid, TEXT("the board's line score and hits were not recorded"));
			if (V.Meta.What.BoardBefore.bValid)
			{
				Expect(V.Meta.What.BoardBefore.Hits[0] == What.BoardBefore.Hits[0] && V.Meta.What.BoardBefore.Hits[1] == What.BoardBefore.Hits[1]
					&& V.Meta.What.BoardBefore.Runs[0] == What.BoardBefore.Runs[0] && V.Meta.What.BoardBefore.Runs[1] == What.BoardBefore.Runs[1], TEXT("the line score before changed"));
			}
			Expect(Viewer.FamilyName(ChangedAgent) == TEXT("Recorded") && Viewer.FamilyName(ChangedField) == TEXT("Recorded"), TEXT("the viewer does not show the recorded name"));
			Expect(Viewer.ThrowsLeft(ChangedAgent) == !bWasLeft, TEXT("the viewer does not use the recorded throwing hand"));
			Expect(FMath::IsNearlyEqual(Viewer.Park().fence_height_m, Fence, 0.f) && Viewer.Park().n_wall == Game.Park().n_wall, TEXT("the viewer does not use the recorded park"));
			int32 Steps = 0;
			for (int32 K = 0;; ++K)
			{
				const float T = FMath::Min(K * SimTickS, Chosen.Info.duration_s);
				BbFrame FA{}, FB{};
				if (!Game.Sample(Chosen, T, FA) || !Viewer.Sample(V.Play, T, FB))
				{
					Fail(FString::Printf(TEXT("a frame could not be sampled at t=%.4f"), T));
					break;
				}
				if (FMemory::Memcmp(&FA, &FB, sizeof(BbFrame)) != 0)
				{
					Fail(FString::Printf(TEXT("frame mismatch at t=%.4f"), T));
					break;
				}
				++Steps;
				if (T >= Chosen.Info.duration_s)
				{
					break;
				}
			}
			Expect(Steps >= 2, TEXT("fewer than two frames compared"));
			UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK opened through the viewer's loader: %d frames equal, recorded name/hand/fence used"), Steps);
		}
		Viewer.Release(V.Play);
	}

	// 3. Files the viewer must refuse, each through the same loader, each with a message.
	auto Refused = [&](const FString& Arg, const TCHAR* ExpectTitle, const TCHAR* Label)
	{
		BbSavePlay::FView V;
		FBbSim Viewer;
		OpenFile(Arg, V, Viewer);
		const bool bRefused = !V.bOk && !V.ErrTitle.IsEmpty() && !V.ErrBody.IsEmpty();
		if (V.bOk)
		{
			Viewer.Release(V.Play);
		}
		Expect(bRefused && (!ExpectTitle || V.ErrTitle == ExpectTitle), FString::Printf(TEXT("%s: %s (%s)"), Label, bRefused ? TEXT("refused with the wrong message") : TEXT("was not refused"), *V.ErrTitle));
		UE_LOG(LogBb, Display, TEXT("BB_SAVED_PLAY_CHECK %s -> %s"), Label, *V.ErrTitle);
	};
	auto Copy = [&](const FString& Name, TFunctionRef<void(FString& Toml, TArray<uint8>& Timeline)> Edit) -> FString
	{
		FString Text;
		TArray<uint8> Bytes;
		FFileHelper::LoadFileToString(Text, *Toml);
		FFileHelper::LoadFileToArray(Bytes, *FPaths::ChangeExtension(Toml, TEXT("timeline")));
		Edit(Text, Bytes);
		const FString Dir = FPaths::Combine(TempDir, Name);
		IFileManager::Get().MakeDirectory(*Dir, true);
		FFileHelper::SaveArrayToFile(Bytes, *FPaths::Combine(Dir, FPaths::GetBaseFilename(Toml) + TEXT(".timeline")));
		const FString Path = FPaths::Combine(Dir, FPaths::GetBaseFilename(Toml) + TEXT(".toml"));
		FFileHelper::SaveStringToFile(Text, *Path, FFileHelper::EEncodingOptions::ForceUTF8WithoutBOM);
		return Path;
	};
	auto DropMd5 = [](FString& Text)
	{
		TArray<FString> Lines;
		Text.ParseIntoArray(Lines, TEXT("\n"), false);
		Lines.RemoveAll([](const FString& L) { return L.StartsWith(TEXT("timeline_md5")); });
		Text = FString::Join(Lines, TEXT("\n"));
	};
	Refused(FString(), TEXT("NO SAVED PLAY GIVEN"), TEXT("empty argument"));
	Refused(FPaths::Combine(TempDir, TEXT("no-such-file.toml")), TEXT("COULD NOT OPEN SAVED PLAY"), TEXT("missing TOML"));
	Refused(Copy(TEXT("flipped"), [](FString&, TArray<uint8>& B) { B[B.Num() / 2] ^= 0x5A; }), TEXT("TIMELINE DOES NOT MATCH ITS SAVE"), TEXT("one flipped byte"));
	Refused(Copy(TEXT("truncated"), [&](FString& T, TArray<uint8>& B) { B.SetNum(B.Num() / 2); DropMd5(T); }), TEXT("TIMELINE UNUSABLE"), TEXT("truncated, no MD5"));
	Refused(Copy(TEXT("empty"), [&](FString& T, TArray<uint8>& B) { B.Reset(); DropMd5(T); }), nullptr, TEXT("empty timeline file"));
	Refused(Copy(TEXT("noname"), [&](FString& T, TArray<uint8>&) { T = T.Replace(TEXT("timeline = \""), TEXT("timeline = \"gone-")); }), TEXT("TIMELINE FILE MISSING"), TEXT("missing timeline file"));
	Refused(Copy(TEXT("play"), [&](FString& T, TArray<uint8>&) { DropMd5(T); T = T.Replace(*FString::Printf(TEXT("\nplay = %u\n"), Chosen.Info.play_index), *FString::Printf(TEXT("\nplay = %u\n"), Chosen.Info.play_index + 1)); }), TEXT("TIMELINE IS ANOTHER PLAY"), TEXT("timeline of another play"));
	{
		// Another game's play with the same play number, put in place of this one's: only the MD5 can tell.
		const uint64 OtherSeed = Seed == 3647 ? 11 : 3647;
		FBbSim Other;
		FBbPlay OtherPlay;
		if (Expect(Other.Start(SimLibPath, OtherSeed), TEXT("could not start the second sim")))
		{
			for (uint32 I = 0; I <= Chosen.Info.play_index; ++I)
			{
				Other.Release(OtherPlay);
				OtherPlay = Other.Next();
			}
			if (Expect(OtherPlay.bValid && OtherPlay.Info.play_index == Chosen.Info.play_index, TEXT("the second sim has no play with the same number")))
			{
				Refused(Copy(TEXT("swapped"), [&](FString&, TArray<uint8>& B) { B.Reset(); B.Append(OtherPlay.Blob, OtherPlay.Len); }), TEXT("TIMELINE DOES NOT MATCH ITS SAVE"), TEXT("same-numbered play of another game"));
			}
			Other.Release(OtherPlay);
		}
	}
	// Summaries that would play wrongly or hang, though they are valid FlatBuffers: refused by the sanity checks.
	{
		FString Why;
		BbPlayInfo I = Chosen.Info;
		Expect(FBbSim::InfoSane(I, Why), FString::Printf(TEXT("the real summary was refused: %s"), *Why));
		auto Bad = [&](const TCHAR* Label, TFunctionRef<void(BbPlayInfo&)> Mutate)
		{
			BbPlayInfo M = Chosen.Info;
			Mutate(M);
			FString W;
			Expect(!FBbSim::InfoSane(M, W) && !W.IsEmpty(), FString::Printf(TEXT("a summary with %s was accepted"), Label));
		};
		Bad(TEXT("zero duration"), [](BbPlayInfo& M) { M.duration_s = 0.f; });
		Bad(TEXT("negative duration"), [](BbPlayInfo& M) { M.duration_s = -3.f; });
		Bad(TEXT("a huge duration"), [](BbPlayInfo& M) { M.duration_s = 3.0e9f; });
		Bad(TEXT("a NaN duration"), [](BbPlayInfo& M) { M.duration_s = std::numeric_limits<float>::quiet_NaN(); });
		Bad(TEXT("an infinite exit speed"), [](BbPlayInfo& M) { M.exit_velocity_mps = std::numeric_limits<float>::infinity(); });
		Bad(TEXT("contact after the end"), [](BbPlayInfo& M) { M.contact_s = M.duration_s + 5.f; });
		Bad(TEXT("too many calls"), [](BbPlayInfo& M) { M.n_calls = 9; });
	}
	// An older save: a recording but no roster, park or board fields. It opens, and says so.
	{
		const BbSavePlay::FWhat Old = [&] { BbSavePlay::FWhat W = What; W.bHasBases = false; W.BoardBefore = BbSavePlay::FBoard(); W.BoardAfter = BbSavePlay::FBoard(); return W; }();
		const FString OldToml = BbSavePlay::WriteTo(TempDir, Old, BbSavePlay::FContext(), TEXT("old"), Chosen.Blob, Chosen.Len);
		BbSavePlay::FView V;
		FBbSim Viewer;
		OpenFile(OldToml, V, Viewer);
		if (Expect(V.bOk, TEXT("a save without roster and park did not open")))
		{
			Expect(!V.CurrentBuildNote.IsEmpty(), TEXT("a save without roster and park does not say its names and park come from the current build"));
			Expect(!V.Meta.What.BoardBefore.bValid && !V.Meta.What.BoardAfter.bValid, TEXT("an old save claims board fields"));
			Viewer.Release(V.Play);
		}
	}
	// A legacy save with no recording at all: parsed, refused with the commit it needs.
	{
		const FString NoTl = FPaths::Combine(TempDir, TEXT("old-no-timeline.toml"));
		FFileHelper::SaveStringToFile(TEXT("seed = 3647\nplay = 15\ncommit = \"30634cf\"\ncommit_inferred = true\ninning = 1\nhalf = \"bottom\"\nouts = 0\ncount = \"1-1\"\nscore = \"away 2, home 0\"\nsaved = 2026-11-15T23:41:03\ncomment = \"\"\"\nold note\n\"\"\"\n"), *NoTl, FFileHelper::EEncodingOptions::ForceUTF8WithoutBOM);
		BbSavePlay::FView V;
		FBbSim Viewer;
		OpenFile(NoTl, V, Viewer);
		Expect(!V.bOk && V.ErrTitle == TEXT("NO RECORDED TIMELINE") && V.ErrBody.Contains(TEXT("30634cf")) && V.Meta.bCommitInferred && V.Meta.Comment == TEXT("old note"), TEXT("a legacy save without a timeline was not refused with its commit"));
	}
	Game.Release(Chosen);
	return Finish();
}

void ABbDirector::ReplayBack()
{
	if (bSavedPlay)
	{
		if (!bSavedPlayBroken && Current.bValid)
		{
			RestartSavedPlay();
		}
		return;
	}
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
			bSideCardOn = false;
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
	if (bSavedPlay || !bReplay)
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

bool ABbDirector::HoldsBlob(const uint8* Blob) const
{
	if (!Blob)
	{
		return false;
	}
	if (Current.Blob == Blob || LivePlay.Blob == Blob || NextPlay.Blob == Blob)
	{
		return true;
	}
	for (const FShown& H : History)
	{
		if (H.Play.Blob == Blob)
		{
			return true;
		}
	}
	return false;
}

void ABbDirector::OpenSaveDialog()
{
	// The play saved is the one the last drawn tick showed (Presented), taken now and copied:
	// Current may already be the next play (a handoff runs at the end of a tick), and nothing
	// that happens while the dialog is open can change what Save writes.
	if (bSaveOpen || !Presented.bValid || !Sim || !GEngine || !GEngine->GameViewport)
	{
		return;
	}
	if (!Presented.Blob || Presented.Len == 0 || !HoldsBlob(Presented.Blob))
	{
		UE_LOG(LogBb, Warning, TEXT("BB_SAVE not opened: the play on screen is no longer held"));
		return;
	}
	SaveTarget = FSaveTarget();
	SaveTarget.Blob.Append(Presented.Blob, static_cast<int32>(Presented.Len));
	SaveTarget.What = WhatOf(*Sim, SeedShown, Presented.Info, Presented.Before);
	SaveTarget.Context = BbSavePlay::ContextOf(*Sim);
	bSaveOpen = true;
	SaveClose = 0;
	bPausedBeforeSave = bPaused;
	bPaused = true;
	TWeakObjectPtr<ABbDirector> Self(this);
	const FText Title = FText::FromString(FString::Printf(TEXT("[SEED %llu    PLAY %u]"), SeedShown, SaveTarget.What.Play));
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
	UE_LOG(LogBb, Display, TEXT("BB_SAVE open seed=%llu play=%u (current play now %u%s)"), SeedShown, SaveTarget.What.Play, Current.Info.play_index, bReset ? TEXT(", walking back into place") : TEXT(""));
}

void ABbDirector::CloseSaveDialog(bool bSave)
{
	if (!bSaveOpen)
	{
		return;
	}
	if (bSave)
	{
		const BbSavePlay::FWhat& What = SaveTarget.What;
		const FString Path = BbSavePlay::Write(What, SaveTarget.Context, SaveComment, SaveTarget.Blob.GetData(), SaveTarget.Blob.Num());
		SavedNote = Path.IsEmpty() ? FString(TEXT("Could not save the play")) : FString::Printf(TEXT("Saved: saved-plays/%s"), *FPaths::GetCleanFilename(Path));
		SavedUntil = Wall + 4.0;
		UE_LOG(LogBb, Display, TEXT("BB_SAVE %s seed=%llu play=%u commit=%s path=%s"), Path.IsEmpty() ? TEXT("failed") : TEXT("saved"), SeedShown, What.Play, *BbSavePlay::BuildCommit(), *Path);
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
	SaveTarget = FSaveTarget();
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

FVector ABbDirector::FromBatterFrame(const FVector& Cm, const FBatNote& N)
{
	const double Side = N.bLeftBox ? 1.0 : -1.0;
	return ToUE(N.Spot.X + Side * Cm.X / 100.0, N.Spot.Y + Cm.Y / 100.0, Cm.Z / 100.0);
}

float ABbDirector::PitchPlaneT(const FBbPlay& Play, float CutAt) const
{
	// The ball's track to the plane the sim's swing aims at (physics: contact_y0, 0.35 m). On a
	// pitch hit the track ends at contact: if that is short of the plane, the crossing is found
	// from the last 30 ms by a parabola (checked against the full track in the probe, step 1).
	constexpr float PlaneY = 0.35f;
	if (!Sim || !Play.bValid)
	{
		return -1.f;
	}
	// (CutAt >= 0: the track as known up to that time, as if it ended there, for checking.)
	const float Contact = CutAt >= 0.f ? CutAt : Play.Info.contact_s;
	const float End = Contact >= 0.f ? Contact : FMath::Min(Play.Info.duration_s, 1.5f);
	auto Y = [&](float T)
	{
		BbFrame F;
		return Sim->Sample(Play, T, F) ? F.ball[1] : -1e9f;
	};
	// (The last step stops at End itself: a ball hit just past the plane crossed it between the
	// last whole step and contact.)
	for (float T = 0.005f; T - 0.005f < End; T += 0.005f)
	{
		if (Y(FMath::Min(T, End)) <= PlaneY)
		{
			float Lo = T - 0.005f, Hi = FMath::Min(T, End);
			for (int32 I = 0; I < 20; ++I)
			{
				const float Mid = 0.5f * (Lo + Hi);
				(Y(Mid) > PlaneY ? Lo : Hi) = Mid;
			}
			return 0.5f * (Lo + Hi);
		}
	}
	if (Contact < 0.f)
	{
		return -1.f;
	}
	const double Y0 = Y(End), A = Y(End - 0.015f) - Y0, B = Y(End - 0.03f) - Y0;
	const double Acc = (B - 2.0 * A) / 0.000225, Vel = (0.0001125 * Acc - A) / 0.015;
	// Y0 + Vel dt + Acc dt^2 / 2 = PlaneY, the smallest dt >= 0.
	const double C = Y0 - PlaneY, Disc = Vel * Vel - 2.0 * Acc * C;
	if (Disc < 0.0)
	{
		return -1.f;
	}
	const double Root = FMath::Sqrt(Disc);
	double Dt = 1e9;
	if (FMath::Abs(Acc) < 1e-9)
	{
		Dt = Vel < 0.0 ? -C / Vel : Dt;
	}
	else
	{
		for (const double R : { (-Vel + Root) / Acc, (-Vel - Root) / Acc })
		{
			if (R >= 0.0 && R < Dt)
			{
				Dt = R;
			}
		}
	}
	return Dt < 1.0 ? End + (float)Dt : -1.f;
}

float ABbDirector::InferContactT(const FBbPlay& Play, const FBatNote& N) const
{
	// The sim's bat turns fastest 0.010 s (at T_swing 0.150, scaled with it) after nominal
	// contact: the parabola through the turning rates of the three 5 ms spans round the fastest,
	// less that offset, T_swing being (contact - start) found by iterating once.
	if (!Sim || !N.bSwing || N.TS < 0.f)
	{
		return -1.f;
	}
	TArray<float> Rate;
	const float T0 = N.TS, T1 = N.EndT;
	BbFrame Prev;
	if (!Sim->Sample(Play, T0, Prev))
	{
		return -1.f;
	}
	int32 Best = -1;
	for (float T = T0 + 0.005f; T <= T1 + 1e-4f; T += 0.005f)
	{
		BbFrame F;
		if (!Sim->Sample(Play, T, F))
		{
			return -1.f;
		}
		const FVector A0 = (ToUE(Prev.bat_tip) - ToUE(Prev.bat_knob)).GetSafeNormal(), A1 = (ToUE(F.bat_tip) - ToUE(F.bat_knob)).GetSafeNormal();
		Rate.Add(FMath::Acos(FMath::Clamp((float)(A0 | A1), -1.f, 1.f)) / 0.005f);
		if (Best < 0 || Rate.Last() > Rate[Best])
		{
			Best = Rate.Num() - 1;
		}
		Prev = F;
	}
	if (Best < 1 || Best + 1 >= Rate.Num())
	{
		return -1.f;
	}
	// A checked swing's bat has stopped by the end of its track (an unchecked one is still
	// turning at about 14% of its contact rate there): its nominal contact is T_swing, 0.150 s
	// at the reference batter, after its start (the batter's own T_swing is not known here).
	if (Rate.Last() < 1.f)
	{
		return T0 + 0.150f;
	}
	const double R0 = Rate[Best - 1], R1 = Rate[Best], R2 = Rate[Best + 1];
	const double A = 0.5 * (R0 + R2) - R1, Bq = 0.5 * (R2 - R0);
	if (A >= 0.0)
	{
		return -1.f;
	}
	// The span i covers (T0 + 0.005 i, T0 + 0.005 (i + 1)): its rate is at the span's middle.
	const double Peak = T0 + 0.005 * (Best + 0.5) + 0.005 * (-Bq / (2.0 * A));
	const double Tsw0 = (Peak - 0.010) - T0;
	const double Tsw = (Peak - 0.010 * Tsw0 / 0.150) - T0;
	return (float)(Peak - 0.010 * Tsw / 0.150);
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
		// The moment the sim's bat first moves, to 0.1 ms (the scan's 5 ms step brackets it; the
		// bat is the stance until then).
		float Lo = Out.StartT, Hi = Out.StartT + 0.005f;
		for (int32 I = 0; I < 6; ++I)
		{
			BbFrame F;
			const float Mid = 0.5f * (Lo + Hi);
			const bool bMoved = Sim->Sample(Play, Mid, F) && (FMemory::Memcmp(F.bat_knob, F0.bat_knob, sizeof(F.bat_knob)) != 0 || FMemory::Memcmp(F.bat_tip, F0.bat_tip, sizeof(F.bat_tip)) != 0);
			(bMoved ? Hi : Lo) = Mid;
		}
		Out.TL = Out.StartT = Hi;
		Out.TS = Play.Info.swing_start_s >= 0.f ? Play.Info.swing_start_s : Out.TL;
		Out.TC = Play.Info.swing_contact_s;
		BbFrame S;
		if (Sim->Sample(Play, Out.TS, S))
		{
			Out.StartKnob = ToUE(S.bat_knob);
			Out.StartTip = ToUE(S.bat_tip);
		}
		if (Out.TC < 0.f)
		{
			Out.bTcInferred = true;
			Out.TC = InferContactT(Play, Out);
		}
	}
	Out.StanceKnobB = BatterFrame(F0.bat_knob, Out);
	Out.StanceAxisB = (BatterFrame(F0.bat_tip, Out) - Out.StanceKnobB).GetSafeNormal();
	const float PlaneAt = PitchPlaneT(Play);
	if (PlaneAt > 0.f)
	{
		Out.TA = PlaneAt;
	}
	Out.CatchT = Play.Info.contact_s < 0.f ? Play.Info.call_s - 0.2f : -1.f;
}

void ABbDirector::DrawnBat(float T, const FBatNote& N, const FVector& SimKnob, const FVector& SimTip, FVector& OutKnob, FVector& OutTip) const
{
	if (bBatLegacy)
	{
		// -BbBatLegacy: the bat as before step 1 (the timeline's, stance and load samples
		// included), for comparing pictures and checks with it.
		OutKnob = SimKnob;
		OutTip = SimTip;
		return;
	}
	FVector CKnob, CAxis;
	BbMotion::LoadBat(T, N.TA, !N.bSwing, N.CatchT, N.StanceKnobB, N.StanceAxisB, CKnob, CAxis);
	// (His frame is in cm and so is Unreal's; the bat is 86.36 cm long in both.)
	const FVector ClientKnob = FromBatterFrame(CKnob, N);
	const FVector ClientTip = FromBatterFrame(CKnob + CAxis * BatLenCm, N);
	if (!N.bSwing || T < N.TL)
	{
		OutKnob = ClientKnob;
		OutTip = ClientTip;
		return;
	}
	// The handover: the client's bat blended into where the sim's will be, the swing's start
	// pose until TS and the sim's bat from then. The weight rises smoothly 0 to 1.
	const float W = BbMotion::HandoverWeight(T, N.TL, N.TS);
	if (W >= 1.f)
	{
		OutKnob = SimKnob;
		OutTip = SimTip;
		return;
	}
	const FVector TargetKnob = T < N.TS ? N.StartKnob : SimKnob;
	const FVector TargetAxis = ((T < N.TS ? N.StartTip : SimTip) - TargetKnob).GetSafeNormal();
	OutKnob = FMath::Lerp(ClientKnob, TargetKnob, (double)W);
	const FVector Axis = FMath::Lerp((ClientTip - ClientKnob).GetSafeNormal(), TargetAxis, (double)W).GetSafeNormal(1e-6, TargetAxis);
	OutTip = OutKnob + Axis * BatLenCm;
}

void ABbDirector::CheckBat(const FBbPlay& Play, const FBatNote& N) const
{
	// The drawn bat over the whole pitch at 1 ms against what player-visuals §13.7 asks of it
	// before and at the handover. The expectations are physical, not the code's own numbers:
	// the hands never faster than 1.5 m/s before the sim's bat moves; no jump (the bat within
	// 2 cm and 2 degrees of the motion carried on from the last 10 ms) where the client's bat
	// gives way and where the handover ends; and, with the swing withheld (the same pitch as a
	// take, the timeline's bat the stance alone), the same bat to 0.1 cm at every time before
	// the sim's bat moves (D22).
	BbFrame F0;
	if (!Sim || !Sim->Sample(Play, 0.f, F0))
	{
		return;
	}
	const FVector StanceK = ToUE(F0.bat_knob), StanceT = ToUE(F0.bat_tip);
	// The same pitch as a take. What the client's bat before the sim's bat moves depends on
	// is the pitch's own clock (TA, the catch), so those are worked out here again, not copied
	// from the swing's note, or the comparison could not fail on them: TA from the ball's track
	// as a take would have it (the whole track), which on a swing that hit is not known past
	// contact, so from the track 30 ms shorter than the swing's (its extrapolation is then
	// from other samples), and on a swing that missed from the track cut at the swing's nominal
	// contact (what the client has when the swing is only just on).
	FBatNote Withheld = N;
	Withheld.bSwing = false;
	Withheld.TL = Withheld.TS = Withheld.TC = -1.f;
	double TaErrMs = 0.0;
	if (N.bSwing)
	{
		const float Contact = Play.Info.contact_s;
		const float Indep = Contact >= 0.f ? PitchPlaneT(Play, Contact - 0.03f) : PitchPlaneT(Play, N.TC);
		if (Indep >= 0.f)
		{
			Withheld.TA = Indep;
			TaErrMs = 1000.0 * FMath::Abs((double)N.TA - Indep);
		}
		Withheld.CatchT = Contact < 0.f ? Play.Info.call_s - 0.2f : Withheld.TA + 0.06f;
	}
	const float T0 = -0.6f, Dt = 0.001f;
	const float Tend = N.bSwing ? N.EndT : FMath::Min(Play.Info.duration_s, (N.CatchT >= 0.f ? N.CatchT : 0.6f) + 1.5f);
	TArray<FVector> Knob, Axis;
	double SpeedBefore = 0.0, SpeedHandover = 0.0, D22 = 0.0;
	for (float T = T0; T <= Tend; T += Dt)
	{
		BbFrame F;
		if (!Sim->Sample(Play, T, F))
		{
			return;
		}
		FVector K, Tip, K2, Tip2;
		DrawnBat(T, N, ToUE(F.bat_knob), ToUE(F.bat_tip), K, Tip);
		DrawnBat(T, Withheld, StanceK, StanceT, K2, Tip2);
		const bool bBefore = !N.bSwing || T < N.TL;
		if (bBefore)
		{
			D22 = FMath::Max(D22, (double)FMath::Max((K - K2).Size(), (Tip - Tip2).Size()));
		}
		if (Knob.Num() > 0)
		{
			const double Mps = (K - Knob.Last()).Size() * 0.01 / Dt;
			if (bBefore)
			{
				SpeedBefore = FMath::Max(SpeedBefore, Mps);
			}
			else if (T < FMath::Max(N.TS, N.TL + BbMotion::BatHandoverS))
			{
				SpeedHandover = FMath::Max(SpeedHandover, Mps);
			}
		}
		Knob.Add(K);
		Axis.Add((Tip - K).GetSafeNormal());
	}
	// How far the drawn bat is from the motion carried on from the 10 ms before At, 5 ms on.
	auto Step = [&](float At, double& Cm, double& Deg)
	{
		Cm = Deg = 0.0;
		const int32 I = FMath::RoundToInt((At - T0) / Dt);
		if (I < 10 || I + 5 >= Knob.Num())
		{
			return;
		}
		const FVector PredK = Knob[I] + (Knob[I] - Knob[I - 10]) * 0.5;
		const FVector PredA = (Axis[I] + (Axis[I] - Axis[I - 10]) * 0.5).GetSafeNormal();
		Cm = (Knob[I + 5] - PredK).Size();
		Deg = FMath::RadiansToDegrees(FMath::Acos(FMath::Clamp((float)(PredA | Axis[I + 5]), -1.f, 1.f)));
	};
	double StepLCm = 0.0, StepLDeg = 0.0, StepECm = 0.0, StepEDeg = 0.0, GapCm = 0.0, GapDeg = 0.0;
	float End = -1.f;
	if (N.bSwing)
	{
		End = FMath::Max(N.TS, N.TL + BbMotion::BatHandoverS);
		Step(N.TL, StepLCm, StepLDeg);
		Step(End, StepECm, StepEDeg);
		// The distance the handover closes: the client's bat at TL against the sim's start pose.
		FVector CK, CA;
		BbMotion::LoadBat(N.TL, N.TA, false, -1.f, N.StanceKnobB, N.StanceAxisB, CK, CA);
		const FVector Ck = FromBatterFrame(CK, N), Ca = (FromBatterFrame(CK + CA * BatLenCm, N) - Ck).GetSafeNormal();
		GapCm = (Ck - N.StartKnob).Size();
		GapDeg = FMath::RadiansToDegrees(FMath::Acos(FMath::Clamp((float)(Ca | (N.StartTip - N.StartKnob).GetSafeNormal()), -1.f, 1.f)));
	}
	const bool bFail = SpeedBefore > 1.5 || StepLCm > 2.0 || StepLDeg > 2.0 || StepECm > 2.0 || StepEDeg > 2.0 || D22 > 0.1 || TaErrMs > 0.5;
	UE_LOG(LogBb, Display, TEXT("BB_BATCHECK play=%u swing=%d t_l=%.4f t_s=%.4f end=%.4f speed_before=%.2f m/s speed_handover=%.2f m/s step_l=%.2f cm %.2f deg step_end=%.2f cm %.2f deg gap=%.1f cm %.1f deg d22=%.4f cm ta_err=%.3f ms%s"),
		Current.Info.play_index, N.bSwing ? 1 : 0, N.TL, N.TS, End, SpeedBefore, SpeedHandover, StepLCm, StepLDeg, StepECm, StepEDeg, GapCm, GapDeg, D22, TaErrMs, bFail ? TEXT(" FAIL") : TEXT(""));
}

void ABbDirector::NotePitch()
{
	PitchEndT = -1.f;
	CrouchUntil = 0.6f;
	const double NoteStart = FPlatformTime::Seconds();
	NoteBat(Current, BatNote);
	const double NoteMs = 1000.0 * (FPlatformTime::Seconds() - NoteStart);
	BatHandMax = 0.f;
	BatHandFrames = 0;
	if (bStagingLog && BatNote.bBat)
	{
		// (With the sim's own t_c, the inference a saved play without it would have been given
		// is logged beside it.)
		const float Inferred = BatNote.bSwing && !BatNote.bTcInferred ? InferContactT(Current, BatNote) : -1.f;
		UE_LOG(LogBb, Display, TEXT("BB_BATNOTE play=%u swing=%d t_a=%.4f catch=%.4f t_l=%.4f t_s=%.4f t_c=%.4f%s handover_end=%.4f infer_err=%.3f ms note=%.1f ms"), Current.Info.play_index, BatNote.bSwing ? 1 : 0, BatNote.TA, BatNote.CatchT, BatNote.TL, BatNote.TS, BatNote.TC,
			BatNote.bTcInferred ? TEXT(" (inferred)") : TEXT(""), BatNote.bSwing ? FMath::Max(BatNote.TS, BatNote.TL + BbMotion::BatHandoverS) : -1.f, Inferred >= 0.f ? 1000.f * (Inferred - BatNote.TC) : 0.f, NoteMs);
	}
	if (bBatCheck && BatNote.bBat)
	{
		CheckBat(Current, BatNote);
	}
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
		static const TCHAR* const Kinds[6] = { TEXT("none"), TEXT("strike"), TEXT("out"), TEXT("safe"), TEXT("foul"), TEXT("fair") };
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
	bSideCutPlanned = bSideCut && bSideChange && ResetDuration > SideCutLeadS + 2.f * SideCutFadeS + SideCutTailS + 3.5f;
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
	else if (bContact && BbBaseMap::IsFoul(Info))
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
		Signals.Add(FBbSignal{ FMath::Min(T + UmpireCallLagS, Info.duration_s), Near(FVector2D(B.X, B.Y), true), ESignal::Foul, FVector2D(B.X, B.Y) });
	}
	else if (bContact && Info.pitch_call == 6)
	{
		// A fair ball that had a call to make is pointed to as soon as it is fair under the
		// rules: first touched by a fielder (not caught in the air), or passing first or third
		// on the ground or after a bound, or settling in front of the bag; a fly beyond the bag
		// where it comes down; a fly over the fence where it clears it. The plate umpire has a
		// ball that is fair in front of the bags, the base umpire one that is fair at or past his.
		// Only a ball decided within FairNearLineDeg of the line (seen from home) is signalled.
		const double BagM = 27.432;
		auto Along = [](const FVector2D& P) { return (FMath::Abs(P.X) + P.Y) / UE_SQRT_2; };
		auto Inside = [](const FVector2D& P) { return (P.Y - FMath::Abs(P.X)) / UE_SQRT_2; };
		const float End = Info.dead_s >= 0.f ? FMath::Min(Info.dead_s, Info.duration_s) : Info.duration_s;
		bool bGround = false, bDecided = false;
		FVector2D Was = FVector2D::ZeroVector, At = FVector2D::ZeroVector;
		float DecidedAt = End, Slow = 0.f;
		for (float Tt = Info.contact_s + 0.05f; Tt <= End && !bDecided; Tt += 0.02f)
		{
			BbFrame F;
			if (!SampleDrawn(Current, Tt, F))
			{
				break;
			}
			const FVector2D P(F.ball[0], F.ball[1]);
			const bool bLanding = !bGround && F.ball[2] < 0.12;
			const bool bPassing = bGround && Along(Was) < BagM && Along(P) >= BagM;
			if (bLanding)
			{
				bGround = true;
			}
			if (bGround)
			{
				Slow = (P - Was).Size() / 0.02 < 0.5 && F.ball[2] < 0.2 && Tt > Info.contact_s + 0.3f ? Slow + 0.02f : 0.f;
			}
			Was = P;
			At = P;
			DecidedAt = Tt;
			if (F.holder >= 0)
			{
				// Taken in the air: the out is the call; taken off the ground, fair there.
				bDecided = bGround;
				if (!bGround)
				{
					break;
				}
			}
			else if (bLanding)
			{
				bDecided = Along(P) >= BagM;
			}
			else if (bPassing || Slow >= 0.1f)
			{
				bDecided = true;
			}
		}
		if (!bDecided && Info.dead_reason == 2 && Info.dead_s >= 0.f)
		{
			// Over the fence in flight: fair where it clears it.
			At = FVector2D(Info.dead_at[0], Info.dead_at[1]);
			DecidedAt = Info.dead_s;
			bDecided = true;
		}
		if (bDecided)
		{
			if (Inside(At) >= 0.0 && FMath::RadiansToDegrees(FMath::Atan2(Inside(At), FMath::Max(Along(At), 1e-3))) <= FairNearLineDeg)
			{
				Signals.Add(FBbSignal{ FMath::Min(DecidedAt + UmpireCallLagS, Info.duration_s), Along(At) < BagM - 3.0 ? 0 : At.X > 0.0 ? 1 : 3, ESignal::Fair, At });
			}
		}
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
	T.Force = Step == 1 ? ESignal::Strike : Step == 2 ? ESignal::Out : Step == 3 ? ESignal::Safe : Step == 4 ? ESignal::Foul : Step == 10 ? ESignal::Fair : ESignal::None;
	CrewShotTick = T;
	FVector Eye, Look;
	float Fov = 40.f;
	if (Step <= 1)
	{
		const FVector2D U = Crew->UmpireAt(0, T);
		Eye = ToUE(U.X + 4.6, U.Y + 3.6, 1.5);
		Look = ToUE(U.X, U.Y, 1.05);
	}
	else if (Step <= 5 || Step == 10)
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
	// -BbBatShots=2: the first swing and the first pitch taken (after the third pitch) held on
	// the moments of the bat before the swing (player-visuals §13.4), from across the plate.
	bBatShotPitch = false;
	if (BatShotsLeft > 0 && Current.bValid && BatNote.bBat && Current.Info.play_index > 3 && !bBatShotDone[BatNote.bSwing ? 0 : 1])
	{
		const FBatNote& Bn1 = BatNote;
		const float Arrive = FMath::Max(FMath::Min(BbMotion::BatLoadAtS, Bn1.TA - BbMotion::BatCommitLeadS - BbMotion::BatLoadBeforeCommitS), BbMotion::BatSettleS + 0.1f);
		TArray<TPair<float, const TCHAR*>> Marks;
		Marks.Emplace(-0.5f, TEXT("stance"));
		Marks.Emplace(BbMotion::BatSettleS + 0.5f * (Arrive - BbMotion::BatSettleS), TEXT("drift"));
		Marks.Emplace(Arrive, TEXT("loaded"));
		if (Bn1.bSwing)
		{
			const float End = FMath::Max(Bn1.TS, Bn1.TL + BbMotion::BatHandoverS);
			Marks.Emplace(Bn1.TL - 0.01f, TEXT("before_sim"));
			Marks.Emplace(0.5f * (Bn1.TL + End), TEXT("handover"));
			Marks.Emplace(End, TEXT("sim_has_it"));
			Marks.Emplace(End + 0.03f, TEXT("swing"));
		}
		else
		{
			Marks.Emplace(Bn1.TA, TEXT("at_the_plate"));
			Marks.Emplace(FMath::Max(Bn1.CatchT, Bn1.TA) + 0.1f, TEXT("caught"));
			Marks.Emplace(FMath::Max(Bn1.CatchT, Bn1.TA) + BbMotion::BatRelaxAfterCatchS + 0.5f * BbMotion::BatRelaxOverS, TEXT("easing"));
			Marks.Emplace(FMath::Max(Bn1.CatchT, Bn1.TA) + BbMotion::BatRelaxAfterCatchS + BbMotion::BatRelaxOverS + 0.1f, TEXT("stance_again"));
		}
		for (int32 M = 0; M < Marks.Num(); ++M)
		{
			ExtraShots.Emplace(Marks[M].Key, FString::Printf(TEXT("bat1_%s_%d_%s.png"), Bn1.bSwing ? TEXT("swing") : TEXT("take"), M + 1, Marks[M].Value));
		}
		ExtraShots.Sort([](const TPair<float, FString>& P, const TPair<float, FString>& Q) { return P.Key < Q.Key; });
		bBatShotDone[Bn1.bSwing ? 0 : 1] = true;
		bBatShotPitch = true;
		--BatShotsLeft;
	}
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
				// A runner diving back into the bag is the sim's dive, drawn as such, not a slide.
				bool bDivingBack = false;
				for (int32 J = FMath::Max(K - 30, 0); J <= FMath::Min(K + 5, Seen.Num() - 1); ++J)
				{
					for (uint32 D = 0; D < Seen[J].n_dives; ++D)
					{
						bDivingBack |= Seen[J].dives[D].agent == Agent && Seen[J].dives[D].base == B + 2;
					}
				}
				const bool bSlide = bPlay && Mps >= SlideMinMps && !bDivingBack;
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
	// -BbDiveBackShots: its first runner's dive back leaving his feet, in the air, the hand on
	// the bag, down, and getting up.
	for (int32 K = 0; DiveBackShotsLeft > 0 && K < Seen.Num() && ShotDiver < 0; ++K)
	{
		for (uint32 D = 0; D < Seen[K].n_dives && ShotDiver < 0; ++D)
		{
			const BbDive& Dv = Seen[K].dives[D];
			if (Dv.base == 0)
			{
				continue;
			}
			const float T0 = K * Step - Dv.since_s, Touch = T0 + Dv.since_s + Dv.end_in_s, Up = T0 + Dv.since_s + Dv.up_in_s;
			const float Marks[5] = { T0, 0.5f * (T0 + Touch), Touch, Up - 0.45f, Up - 0.15f };
			for (int32 M = 0; M < 5; ++M)
			{
				ExtraShots.Emplace(Marks[M], FString::Printf(TEXT("diveback_%02d_%d.png"), DiveBackShotIndex, M + 1));
			}
			ShotDiver = Dv.agent;
			ShotDiveFrom = T0;
			ShotDiveYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
			if (bStagingLog)
			{
				UE_LOG(LogBb, Display, TEXT("BB_DIVEBACK play=%u agent=0x%03x base=%d leave=%.2f end=%.2f touched=%u up=%.2f"), Current.Info.play_index, Dv.agent, Dv.base, T0, Touch, Dv.touched, Up);
			}
			++DiveBackShotIndex;
			--DiveBackShotsLeft;
		}
	}
	for (int32 K = 0; bWantDives && K < Seen.Num() && ShotDiver < 0; ++K)
	{
		if (Seen[K].n_dives > 0 && Seen[K].dives[0].base == 0)
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
	// -BbBaseMapShots: a play with a runner on or one left on, photographed along its length.
	if (BaseMapShotsLeft > 0 && ((Before.bases & 7) != 0 || (Current.Info.bases & 7) != 0))
	{
		const float Marks[4] = { 0.2f, 0.4f, 0.6f, 0.85f };
		for (int32 M = 0; M < 4; ++M)
		{
			ExtraShots.Emplace(Current.Info.duration_s * Marks[M], FString::Printf(TEXT("basemap_%02d_%d.png"), BaseMapShotIndex, M + 1));
		}
		++BaseMapShotIndex;
		--BaseMapShotsLeft;
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
	// A runner diving back to a bag, as the sim has it (player-visuals §11).
	if (Coded && bMotions && !bFielder && In.bDiveBack && !bReset)
	{
		Motion = 7;
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
		else if (St.Motion == 7)
		{
			WantYaw = In.bDiveBack ? In.DiveBackYaw : St.Yaw;
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
		bMirror = St.Motion == 3 ? Bn.bLeftBox : St.Motion >= 6 ? false : bLefty;
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
			else if (St.Motion == 7)
			{
				// (Blending out after he is up: standing on the bag, the pose's last key.)
				S = In.bDiveBack ? BbMotion::DiveBack(In.DiveBackSince, In.DiveBackEndIn, In.DiveBackUpIn, In.bDiveBackTouched, Skel, GroundOwn, Reach) : BbMotion::DiveBack(9.f, -9.f, 0.f, true, Skel, GroundOwn, Reach);
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
			// And through the handover and the swing (the sim's bat first moving to its end): the
			// worst either hand has been from the handle, logged when the swing is over.
			if (bStagingLog && St.Motion == 3 && !bReset && Bn.bSwing)
			{
				if (PlayTime >= Bn.TL && PlayTime <= Bn.EndT)
				{
					const float Off = FMath::Max((Wrist[0] - Reach.At[0]).Size() - Reach.Past[0], (Wrist[1] - Reach.At[1]).Size() - Reach.Past[1]) * FMath::Abs(Scale.Z);
					BatHandMax = FMath::Max(BatHandMax, Off);
					++BatHandFrames;
					if (Off > 0.1f)
					{
						UE_LOG(LogBb, Display, TEXT("BB_BATHAND_OFF play=%u t=%.4f from_t_l=%.4f from_t_s=%.4f to_end=%.4f low_off=%.2f high_off=%.2f cm swingw=%.2f"), Current.Info.play_index, PlayTime, PlayTime - Bn.TL, PlayTime - Bn.TS, Bn.EndT - PlayTime,
							((Wrist[0] - Reach.At[0]).Size() - Reach.Past[0]) * FMath::Abs(Scale.Z), ((Wrist[1] - Reach.At[1]).Size() - Reach.Past[1]) * FMath::Abs(Scale.Z), BatSwingW);
					}
				}
				else if (PlayTime > Bn.EndT && BatHandFrames > 0)
				{
					UE_LOG(LogBb, Display, TEXT("BB_BATHANDS play=%u worst_hand_off=%.2f cm over %d frames from t_l=%.3f to %.3f"), Current.Info.play_index, BatHandMax, BatHandFrames, Bn.TL, Bn.EndT);
					BatHandFrames = 0;
				}
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
	if (bSaveHandoffNow)
	{
		bSaveHandoffNow = false;
		SaveAt = 0.f;
		SaveAuto = 0;
	}
	// -BbSaveAt=<s>: S, then Save a second later, for an unattended check.
	if (SaveAt >= 0.f && Presented.bValid && Wall >= SaveAt + (SaveAuto == 1 ? 1.f : 0.f))
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
		if (PC->WasInputKeyJustPressed(EKeys::S) && Presented.bValid && !bSavedPlay)
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
		if (PC->WasInputKeyJustPressed(EKeys::Z))
		{
			bZoneOn = !bZoneOn;
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
			if (bSideCardOn)
			{
				// The between-innings card: Enter ends the hold and goes on to the next pitch.
				bSideCardSkip = true;
				bRush = true;
			}
			else
			{
				bRush = !bRush;
			}
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
	else if (!Current.bValid && ((!bGameOver && Sim) || (bSavedPlay && bSavedPlayBroken)))
	{
		if (ABbHUD* Hud = GetWorld()->GetFirstPlayerController() ? Cast<ABbHUD>(GetWorld()->GetFirstPlayerController()->GetHUD()) : nullptr)
		{
			if (bSavedPlay && bSavedPlayBroken)
			{
				if (SavedPlay.bHasSeed || SavedPlay.bHasPlay)
				{
					FBbBugInfo& Bug = Hud->Bug;
					Bug = FBbBugInfo();
					Bug.bValid = true;
					Bug.Runs[0] = SavedBefore.away_runs;
					Bug.Runs[1] = SavedBefore.home_runs;
					Bug.Inning = SavedBefore.inning;
					Bug.bTop = SavedBefore.top != 0;
					Bug.Outs = SavedBefore.outs;
					Bug.Balls = SavedBefore.balls;
					Bug.Strikes = SavedBefore.strikes;
					Bug.Bases = SavedBefore.bases;
					FString CommitTag = SavedPlay.Commit.IsEmpty() ? TEXT("unknown") : SavedPlay.Commit;
					if (SavedPlay.bCommitInferred)
					{
						CommitTag += TEXT(" (inferred)");
					}
					Hud->Replay = FString::Printf(TEXT("SAVED PLAY    seed %llu    play %u    %s"), SeedShown, SavedPlay.What.Play, *CommitTag);
					Hud->SavedCommentTitle = SavedPlay.Saved.IsEmpty() ? FString(TEXT("COMMENT")) : FString::Printf(TEXT("COMMENT  (%s)"), *SavedPlay.Saved);
					Hud->SavedComment = SavedPlay.Comment.IsEmpty() ? FString(TEXT("(no comment)")) : SavedPlay.Comment;
					{
						BbBoxScore ErrBox{};
						bool bHits = false;
						SavedBoard(false, SavedBefore, SavedBefore.inning, ErrBox, bHits);
						Park->ShowBoard(ErrBox, SavedBefore.inning, SavedBefore.top != 0, SavedBefore.balls, SavedBefore.strikes, SavedBefore.outs, bHits);
					}
				}
				else
				{
					Hud->Bug = FBbBugInfo();
					Hud->Replay = TEXT("SAVED PLAY");
					Hud->SavedCommentTitle.Reset();
					Hud->SavedComment.Reset();
				}
				Hud->PitchLine.Reset();
				Hud->Call.Reset();
				Hud->Umpire.Reset();
				Hud->bZoneShown = false;
				Hud->HomeRunAlpha = 0.f;
				Hud->SavedErrorTitle = SavedErrorTitle;
				Hud->SavedErrorBody = SavedErrorBody;
				Hud->SavedWarning.Reset();
				Hud->Footer = TEXT("C: camera    Q: quit");
			}
			else
			{
				Hud->Bug = FBbBugInfo();
				Hud->Call = TEXT("Warming up");
			}
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
		float SimDt = bPaused ? 0.f : DeltaSeconds * (bRush ? FMath::Max(Speed, RushSpeed) : Speed);
		if (bSavedPlay)
		{
			const float EndHold = Current.Info.duration_s + HoldAfter;
			const float NextT = FMath::Min(PlayTime + SimDt, EndHold);
			SimDt = FMath::Max(0.f, NextT - PlayTime);
			PlayTime = NextT;
		}
		else
		{
			PlayTime += SimDt;
		}
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
				if (++ExtraShotNext == ExtraShots.Num() && SlideShotsLeft <= 0 && DiveShotsLeft <= 0 && DiveBackShotsLeft <= 0 && BaseMapShotsLeft <= 0 && BatShotsLeft <= 0)
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
			const float Black = SideCutFade(SimDt, DeltaSeconds);
			if (APlayerController* PC = GetWorld()->GetFirstPlayerController())
			{
				if (PC->PlayerCameraManager)
				{
					PC->PlayerCameraManager->SetManualCameraFade(Black, FLinearColor::Black, false);
				}
				if (ABbHUD* CardHud = Cast<ABbHUD>(PC->GetHUD()))
				{
					CardHud->CardAlpha = Black;
					CardHud->bCardHint = bSideCardOn;
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
			// The bat (player-visuals §9, §13.4). It is the timeline's for as long as the sim's bat
			// is in his hands: the client's own through the stance and the load, until the sim's
			// bat first moves, the same on a swing and a pitch taken (D22); then blended into the
			// sim's, and the sim's through the swing. After the swing, and between pitches, it is
			// the batter's own to carry (PoseFigure).
			bBatHeld = false;
			bBatSim = !bReset && Frame.has_bat != 0 && (!BatNote.bSwing || PlayTime <= BatNote.EndT);
			if (bBatSim)
			{
				DrawnBat(PlayTime, BatNote, ToUE(Frame.bat_knob), ToUE(Frame.bat_tip), BatKnob, BatTip);
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
							if (Dv.agent == A.agent && Dv.base != 0)
							{
								// A runner's dive back (agent-models §6.4), on the sim's clock. He is
								// drawn from his touching hand, which the sim's glide carries at a
								// constant speed from hand_from to hand_to (the bag, if he touched),
								// where it lies from the glide's end on. At take-off the origin is
								// still his path point, DIVE_REACH_M (0.4 m) behind the hand; it
								// moves onto the hand over the first half of the glide.
								In.bDiveBack = true;
								In.DiveBackSince = Dv.since_s;
								In.DiveBackEndIn = Dv.end_in_s;
								In.DiveBackUpIn = Dv.up_in_s;
								In.bDiveBackTouched = Dv.touched != 0;
								In.DiveBackYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
								const float F = Dv.end_in_s > 0.f ? FMath::Clamp(Dv.since_s / FMath::Max(Dv.since_s + Dv.end_in_s, 0.05f), 0.f, 1.f) : 1.f;
								const float Behind = 0.4f * (1.f - FMath::SmoothStep(0.f, 0.5f, F));
								A.pos[0] = FMath::Lerp(Dv.hand_from[0], Dv.hand_to[0], F) - Dv.dir[0] * Behind;
								A.pos[1] = FMath::Lerp(Dv.hand_from[1], Dv.hand_to[1], F) - Dv.dir[1] * Behind;
							}
							else if (Dv.agent == A.agent)
							{
								In.DiveSince = Dv.since_s + DiveLeadS;
								In.DiveUpIn = Dv.up_in_s;
								In.bDiveDir = Dv.dir[0] != 0.f || Dv.dir[1] != 0.f;
								In.DiveYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
							}
						}
						// A runner about to dive back: the pose blends in over the DiveBackLeadS
						// before the sim's take-off, so he is leaving his feet when the sim has him
						// leave them (review P3-6); his place is the sim's until then.
						BbFrame Ahead;
						if (!In.bDiveBack && bFigure && PlayTime + DiveBackLeadS <= Current.Info.duration_s && SampleDrawn(Current, PlayTime + DiveBackLeadS, Ahead))
						{
							for (uint32 D = 0; D < Ahead.n_dives; ++D)
							{
								const BbDive& Dv = Ahead.dives[D];
								if (Dv.agent == A.agent && Dv.base != 0 && Dv.since_s < DiveBackLeadS)
								{
									In.bDiveBack = true;
									In.DiveBackSince = Dv.since_s - DiveBackLeadS;
									In.DiveBackEndIn = Dv.end_in_s + DiveBackLeadS;
									In.DiveBackUpIn = Dv.up_in_s + DiveBackLeadS;
									In.bDiveBackTouched = Dv.touched != 0;
									In.DiveBackYaw = ToUE(Dv.dir[0], Dv.dir[1], 0.0).Rotation().Yaw;
								}
							}
						}
						if (In.DiveSince < 0.f && bFigure && PlayTime + DiveLeadS <= Current.Info.duration_s && SampleDrawn(Current, PlayTime + DiveLeadS, Ahead))
						{
							for (uint32 D = 0; D < Ahead.n_dives; ++D)
							{
								const BbDive& Dv = Ahead.dives[D];
								if (Dv.agent == A.agent && Dv.base == 0)
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
			// In play: the ball has been hit and is heading out. Not before the bat meets it on
			// screen (owner, saved play 112): the director holds the whole play, so anything it
			// does because the ball will be hit (the cut from the long lens to the follow view)
			// has to wait for the contact the viewer sees. This used to start at T > 0.35 s with
			// the ball still short of the plate, 0.048 s before contact in play 112.
			if (!bBallInPlay && Current.Info.exit_velocity_mps > 0.f && Current.Info.contact_s >= 0.f && T >= Current.Info.contact_s && Frame.ball[1] > 1.5f)
			{
				bBallInPlay = true;
				if (bStagingLog)
				{
					UE_LOG(LogBb, Display, TEXT("BB_INPLAY play=%u t=%.3f contact=%.3f"), Current.Info.play_index, T, Current.Info.contact_s);
				}
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
			if ((bMotionShots && !bReset && MotionStrip >= 3 && MotionStrip <= 5) || (bBatShotPitch && !bReset))
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
			if (bSavedPlay)
			{
				FString CommitTag = SavedPlay.Commit.IsEmpty() ? TEXT("unknown") : SavedPlay.Commit;
				if (SavedPlay.bCommitInferred)
				{
					CommitTag += TEXT(" (inferred)");
				}
				Hud->Replay = FString::Printf(TEXT("SAVED PLAY    seed %llu    play %u    %s"), SeedShown, SavedPlay.What.Play, *CommitTag);
				Hud->SavedCommentTitle = SavedPlay.Saved.IsEmpty() ? FString(TEXT("COMMENT")) : FString::Printf(TEXT("COMMENT  (%s)"), *SavedPlay.Saved);
				Hud->SavedComment = SavedPlay.Comment.IsEmpty() ? FString(TEXT("(no comment)")) : SavedPlay.Comment;
				Hud->SavedErrorTitle.Reset();
				Hud->SavedErrorBody.Reset();
				Hud->SavedWarning = SavedWarning;
				Hud->Footer = FString::Printf(TEXT("Space: %s    Speed %.2fx: 1-5 for 1/2/4/6/10x, - / = by 0.25x    R: replay    C: camera (%s)    Z: zone    Q: quit"), bPaused ? TEXT("paused") : TEXT("pause"), Speed, CamNames[CameraMode]);
			}
			else
			{
				Hud->Replay = bReplay ? FString::Printf(TEXT("REPLAY    seed %llu    play %u"), SeedShown, Current.Info.play_index) : FString();
				Hud->SavedCommentTitle.Reset();
				Hud->SavedComment.Reset();
				Hud->SavedErrorTitle.Reset();
				Hud->SavedErrorBody.Reset();
				Hud->SavedWarning.Reset();
				Hud->Footer = FString::Printf(TEXT("Space: %s    Speed %.2fx: 1-5 for 1/2/4/6/10x, - / = by 0.25x    Enter: %s    R / F: replay    S: save play    C: camera (%s)    Z: zone    Q: quit"), bPaused ? TEXT("paused") : TEXT("pause"), Speed, bRush ? TEXT("skipping") : TEXT("next pitch"), CamNames[CameraMode]);
			}
			Hud->Notice = Wall < SavedUntil ? SavedNote : FString();
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
				// The base map follows the runners in the timeline at the playback clock (owner,
				// 2026-11-21): a dot for each, a base lit while one is at it. Its end is the
				// sim's result, so there is no jump when the play is known.
				Bug.Dots.Reset();
				if (bHave && Sim)
				{
					BbBaseMap::FMap Map;
					BbBaseMap::Read(Frame, Sim->Park(), In, Before.bases & 7, Before.outs, T, Map);
					Bug.Bases = Map.Lit;
					for (const BbBaseMap::FDot& D : Map.Dots)
					{
						Bug.Dots.Add(D.At);
					}
				}
				// The board in the outfield shows the same, with the line score read from the
				// sim's log once a pitch.
				const uint32 Through = In.play_index + (bKnown ? 1u : 0u);
				if (bSavedPlay)
				{
					Bug.Pitcher.Reset();
					Bug.Batter.Reset();
					const int32 Fielding = Half.top ? 1 : 0;
					if (const BbPlayer* P = Sim->Player(static_cast<uint16>((Fielding << 8) | 0x81)))
					{
						Bug.Pitcher = FString(UTF8_TO_TCHAR(P->family)).ToUpper();
						Bug.PitcherNumber = P->number;
						Bug.Pitches = -1;
					}
					if (const BbPlayer* B = Sim->Player(In.batter))
					{
						static const TCHAR* Pos[] = { TEXT(""), TEXT("P"), TEXT("C"), TEXT("1B"), TEXT("2B"), TEXT("3B"), TEXT("SS"), TEXT("LF"), TEXT("CF"), TEXT("RF") };
						Bug.Batter = FString(UTF8_TO_TCHAR(B->family)).ToUpper();
						Bug.BatterNumber = B->number;
						Bug.BatterPosition = B->position <= 9 ? Pos[B->position] : TEXT("");
						Bug.BatterToday.Reset();
					}
					// The board as the save recorded it, every tick (the park repaints only on a
					// change): the outs go up with the umpire's call, the runs and hits with the result.
					BbBoxScore SavedBox{};
					bool bSavedHits = false;
					SavedBoard(bKnown, Src, Half.inning, SavedBox, bSavedHits);
					Park->ShowBoard(SavedBox, Half.inning, Half.top != 0, bHalfOver && bKnown ? 0 : Src.balls, bHalfOver && bKnown ? 0 : Src.strikes, Outs, bSavedHits);
				}
				else
				{
				if (Sim && BoardThrough != Through)
				{
					bBoardValid = Sim->BoxScore(Through, BoardBox);
					BoardThrough = Through;
					// The between-innings card shows this line score too (hud.md step 4).
					Hud->LineBox = BoardBox;
					// The card shows the half just played (the board before the last play).
					BbHud::TrimUnplayed(Hud->LineBox, Before.inning, Before.top != 0);
					Hud->bLineBox = bBoardValid;
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
			// The strike-zone box (hud.md step 3): this batter's zone, and the pitches of
			// this at-bat. The ones before this pitch are the plays just before it in the
			// history with the same batter (his id carries his side, and nobody bats twice
			// running); this one goes in when it reaches the plate.
			{
				FBbZoneInfo& Zone = Hud->Zone;
				Zone.bValid = In.zone_top_m > In.zone_bottom_m;
				Zone.Bottom = In.zone_bottom_m;
				Zone.Top = In.zone_top_m;
				Zone.HalfWidth = In.zone_half_width_m;
				Zone.Marks.Reset();
				auto Mark = [&Zone](const BbPlayInfo& P)
				{
					if (P.plate_crossed)
					{
						Zone.Marks.Add({ P.plate_x_m, P.plate_z_m, P.pitch_call });
					}
				};
				int32 First = bReplay ? ReplayIdx : History.Num();
				while (First > 0 && History[First - 1].Play.Info.batter == In.batter)
				{
					--First;
				}
				for (int32 I = First; I < (bReplay ? ReplayIdx : History.Num()); ++I)
				{
					Mark(History[I].Play.Info);
				}
				if (PlayTime >= Arrived)
				{
					Mark(In);
				}
				Hud->bZoneShown = bZoneOn;
			}
		}
		// What this tick drew, for the save dialog (before any handoff below replaces Current).
		Presented.bValid = true;
		Presented.Blob = Current.Blob;
		Presented.Len = Current.Len;
		Presented.Info = Current.Info;
		Presented.Before = Before;
		if (bSavedPlay)
		{
			// Single recorded play: hold at the end until R restarts or Q quits.
		}
		else if (bSaveOpen)
		{
			// The save dialog is open: the screen stays on the play being saved.
		}
		else if (bReplay && ReplayIdx >= History.Num() && PlayTime >= Current.Info.duration_s)
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
				bSideCardOn = false;
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
		else if (PlayTime >= Current.Info.duration_s + HoldAfter && NextPlay.bValid && !bSaveOpen)
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
		else if (PlayTime >= Current.Info.duration_s + HoldAfter && bPending && Pending.IsReady() && !bSaveOpen)
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
		static const TCHAR* const Names[11] = { TEXT("plate_set"), TEXT("strike"), TEXT("out"), TEXT("safe"), TEXT("foul"), TEXT("base_set"), TEXT("coach"),
			TEXT("bench_home"), TEXT("bench_away"), TEXT("all"), TEXT("fair") };
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
			if (++CrewShot == 11)
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
	// The clubs by nickname, as on the bug; at the end the winner comes first.
	const int32 Runs[2] = { Box.teams[0].totals.r, Box.teams[1].totals.r };
	const int32 First = Box.is_final && Runs[1] > Runs[0] ? 1 : 0;
	Title = FString::Printf(TEXT("%s %d, %s %d"), BbLook::Clubs[First].Name, Runs[First], BbLook::Clubs[1 - First].Name, Runs[1 - First]);
	if (Box.is_final)
	{
		Subtitle = Box.innings == 9 ? FString(TEXT("FINAL")) : FString::Printf(TEXT("FINAL  -  %d INNINGS"), Box.innings);
	}
	else
	{
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
	bWant = bWant && Current.bValid && Sim && !bSavedPlay;
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
