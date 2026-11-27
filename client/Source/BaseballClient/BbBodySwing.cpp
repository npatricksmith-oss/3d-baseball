#include "BbBodySwing.h"
#include "BbFigure.h"
#include "BbGameMode.h"
#include "BbPark.h"
#include "Camera/CameraActor.h"
#include "Camera/CameraComponent.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/StaticMesh.h"
#include "Engine/World.h"
#include "GameFramework/PlayerController.h"
#include "InputCoreTypes.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "Math/RotationMatrix.h"
#include "Misc/CommandLine.h"
#include "Misc/FileHelper.h"
#include "Misc/Parse.h"
#include "Misc/Paths.h"
#include "UnrealClient.h"

DEFINE_LOG_CATEGORY_STATIC(LogBbSwing, Log, All);

namespace
{
	const TCHAR* const SwingFormat = TEXT("bb-body-swing 1");
	/** The check's limits (player-visuals §14): every drawn joint, the knob, the sweet spot and each
	 *  hand's grip within this of the file's, mm (the honest largest is the shoulders' 4.2 mm as the
	 *  chest piece carries them); every drawn segment's forward axis and the bat's roll within this
	 *  of the file's turn, degrees. */
	constexpr double CheckLimitMm = 6.0;
	constexpr double TurnLimitDeg = 3.0;
	/** Playback speeds, shares of real time. */
	const double Speeds[] = { 1.0, 0.5, 0.25, 0.1, 0.05 };
	const TCHAR* const SpeedNames[] = { TEXT("real speed"), TEXT("1/2"), TEXT("1/4"), TEXT("1/10"), TEXT("1/20") };
	/** How long the first and the last computed pose are held when playing, s of real time. */
	constexpr double HoldStartS = 0.8, HoldEndS = 1.5;
	/** The views: 0 from the pitcher's side, 1 from the open side, 2 from above, 3 orbit. */
	const TCHAR* const ViewNames[] = { TEXT("pitcher's side"), TEXT("open side"), TEXT("above"), TEXT("orbit") };
	const TCHAR* const ViewTags[] = { TEXT("pitcher"), TEXT("open"), TEXT("above"), TEXT("orbit") };
	constexpr double MphPerMps = 1.0 / 0.44704;
	/** The ball's diameter, cm: drawn at its real size here, so its place at the bat reads true. */
	constexpr double BallCm = 7.32;

	/** A unit vector square to A, as near Hint as can be (or to Spare if Hint lies along A). */
	FVector SquareTo(const FVector& A, const FVector& Hint, const FVector& Spare)
	{
		FVector F = Hint - A * (Hint | A);
		if (F.SizeSquared() < 1e-8)
		{
			F = Spare - A * (Spare | A);
		}
		return F.GetSafeNormal();
	}
}

// ---------------------------------------------------------------------------------------------
// The file

bool FBbSwingFile::Load(const FString& InPath, FString& OutError)
{
	Path = InPath;
	Name = FPaths::GetCleanFilename(InPath);
	TArray<FString> Lines;
	if (InPath.IsEmpty())
	{
		OutError = TEXT("-BbBodySwing= has no path after the equals sign");
		return false;
	}
	if (!FFileHelper::LoadFileToStringArray(Lines, *InPath))
	{
		OutError = FString::Printf(TEXT("cannot read %s"), *InPath);
		return false;
	}
	bool bFormat = false;
	int32 WantSamples = -1;
	Data.Reset();
	Samples = 0;
	TArray<FString> Tok;
	for (FString Line : Lines)
	{
		int32 Hash;
		if (Line.FindChar(TEXT('#'), Hash))
		{
			Line.LeftInline(Hash);
		}
		Line.TrimStartAndEndInline();
		if (Line.IsEmpty())
		{
			continue;
		}
		Line.ParseIntoArrayWS(Tok);
		const FString& K = Tok[0];
		auto Rest = [&Line]()
		{
			int32 Sp;
			return Line.FindChar(TEXT(' '), Sp) ? Line.Mid(Sp + 1).TrimStartAndEnd() : FString();
		};
		auto Named = [&Tok](const TCHAR* Key, double& Out)
		{
			for (int32 I = 1; I + 1 < Tok.Num(); ++I)
			{
				if (Tok[I] == Key)
				{
					Out = FCString::Atod(*Tok[I + 1]);
				}
			}
		};
		auto List = [&Tok](TArray<FString>& Out)
		{
			Out.Reset();
			for (int32 I = 2; I < Tok.Num(); ++I)
			{
				Out.Add(Tok[I]);
			}
			return Tok.Num() >= 2 && FCString::Atoi(*Tok[1]) == Out.Num();
		};
		if (K == TEXT("s"))
		{
			if (Columns <= 0 || Tok.Num() - 1 != Columns)
			{
				OutError = FString::Printf(TEXT("sample %d has %d numbers, the header says %d"), Samples + 1, Tok.Num() - 1, Columns);
				return false;
			}
			for (int32 I = 1; I < Tok.Num(); ++I)
			{
				Data.Add(FCString::Atod(*Tok[I]));
			}
			++Samples;
		}
		else if (K == TEXT("format"))
		{
			bFormat = Rest() == SwingFormat;
		}
		else if (K == TEXT("kind"))
		{
			bRecorded = Tok.Num() > 1 && Tok[1] == TEXT("recorded");
		}
		else if (K == TEXT("title"))
		{
			Title = Rest();
		}
		else if (K == TEXT("ball_after_contact"))
		{
			bBallAfter = Tok.Num() > 1 && Tok[1] == TEXT("shown");
		}
		else if (K == TEXT("plan"))
		{
			Plan = Rest();
		}
		else if (K == TEXT("commit"))
		{
			Commit = Tok.Num() > 1 ? Tok[1] : FString();
		}
		else if (K == TEXT("tree"))
		{
			Tree = Rest();
		}
		else if (K == TEXT("batter_height_m") && Tok.Num() > 1)
		{
			HeightM = FCString::Atod(*Tok[1]);
		}
		else if (K == TEXT("contact_speed_mps") && Tok.Num() > 1)
		{
			ContactMps = FCString::Atod(*Tok[1]);
		}
		else if (K == TEXT("contact_speed_mph") && Tok.Num() > 1)
		{
			ContactMph = FCString::Atod(*Tok[1]);
		}
		else if (K == TEXT("miss_cm"))
		{
			Named(TEXT("total"), MissCm);
		}
		else if (K == TEXT("bat"))
		{
			Named(TEXT("length_m"), BatLengthM);
			Named(TEXT("sweet_from_knob_m"), SweetFromKnobM);
		}
		else if (K == TEXT("event") && Tok.Num() > 2)
		{
			Events.Emplace(Tok[1], FCString::Atod(*Tok[2]));
		}
		else if ((K == TEXT("bodies") && !List(Bodies)) || (K == TEXT("joints") && !List(Joints)) || (K == TEXT("points") && !List(Points))
			|| (K == TEXT("contacts") && !List(Contacts)))
		{
			OutError = FString::Printf(TEXT("the %s line's count does not match its names"), *K);
			return false;
		}
		else if (K == TEXT("columns") && Tok.Num() > 1)
		{
			Columns = FCString::Atoi(*Tok[1]);
			if (Columns != 1 + 7 * Bodies.Num() + 3 * Joints.Num() + 3 * Points.Num() + 1 + 4 * Contacts.Num() + 3)
			{
				OutError = FString::Printf(TEXT("the header gives %d columns; its bodies, joints, points and contacts make %d"), Columns,
					1 + 7 * Bodies.Num() + 3 * Joints.Num() + 3 * Points.Num() + 1 + 4 * Contacts.Num() + 3);
				return false;
			}
		}
		else if (K == TEXT("samples") && Tok.Num() > 1)
		{
			WantSamples = FCString::Atoi(*Tok[1]);
		}
	}
	if (!bFormat)
	{
		OutError = FString::Printf(TEXT("%s is not a body-swing file (no \"format %s\" line)"), *Name, SwingFormat);
		return false;
	}
	if (Samples < 2 || Samples != WantSamples)
	{
		OutError = FString::Printf(TEXT("%d samples read, the header says %d"), Samples, WantSamples);
		return false;
	}
	if (BatLengthM <= 0.0 || SweetFromKnobM <= 0.0)
	{
		OutError = TEXT("the bat line gives no length or sweet spot");
		return false;
	}
	JointCol = 1 + 7 * Bodies.Num();
	PointCol = JointCol + 3 * Joints.Num();
	SweetCol = PointCol + 3 * Points.Num();
	ContactCol = SweetCol + 1;
	BallCol = ContactCol + 4 * Contacts.Num();
	for (int32 I = 1; I < Samples; ++I)
	{
		if (!(T(I) > T(I - 1)))
		{
			OutError = FString::Printf(TEXT("the samples' times do not rise at sample %d"), I + 1);
			return false;
		}
	}
	return true;
}

FQuat FBbSwingFile::BodyRot(int32 I, int32 B) const
{
	const int32 C = 1 + 7 * B + 3;
	// The file's (w, x, y, z); a quaternion's algebra does not care which frame it acts in.
	return FQuat(At(I, C + 1), At(I, C + 2), At(I, C + 3), At(I, C)).GetNormalized();
}

bool FBbSwingFile::Event(const TCHAR* EventName, double& Out) const
{
	for (const TPair<FString, double>& E : Events)
	{
		if (E.Key == EventName)
		{
			Out = E.Value;
			return true;
		}
	}
	return false;
}

// ---------------------------------------------------------------------------------------------
// The figure

ABbBodySwing::ABbBodySwing()
{
	PrimaryActorTick.bCanEverTick = true;
	RootComponent = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
}

FVector ABbBodySwing::ToCm(const FVector& Sim) const
{
	// -BbBodySwingCheckFault=mirror: a slip in this map alone (x turned over), which the check must catch.
	const double X = Fault == TEXT("mirror") ? -Sim.X : Sim.X;
	return FVector(100.0 * Sim.Y, 100.0 * X, 100.0 * Sim.Z + GroundCm);
}

FVector ABbBodySwing::AxisUE(const FBbSwingPose& P, int32 Body, const FVector& Local) const
{
	const FVector V = P.BodyRot[Body].RotateVector(Local);
	return FVector(V.Y, V.X, V.Z);
}

FVector ABbBodySwing::MapVector(const FBbSwingSeg& S, const FVector& V) const
{
	return S.FM * (S.SF * (V | S.F)) + S.LM * (S.SL * (V | S.L)) + S.AM * (S.SA * (V | S.A));
}

FVector ABbBodySwing::MapPoint(const FBbSwingSeg& S, const FVector& P) const
{
	return S.OM + MapVector(S, P - S.O);
}

bool ABbBodySwing::BuildFigure(FString& OutError)
{
	// What the figure needs from the file.
	for (const TCHAR* B : { TEXT("pelvis"), TEXT("trunk"), TEXT("thigh_L"), TEXT("shank_L"), TEXT("foot_L"), TEXT("thigh_R"), TEXT("shank_R"), TEXT("foot_R"),
		TEXT("uarm_L"), TEXT("farm_L"), TEXT("uarm_R"), TEXT("farm_R"), TEXT("bat") })
	{
		if (File.Body(B) == INDEX_NONE)
		{
			OutError = FString::Printf(TEXT("the file has no body %s"), B);
			return false;
		}
	}
	for (const TCHAR* J : { TEXT("lumbar"), TEXT("hip_L"), TEXT("knee_L"), TEXT("ankle_L"), TEXT("hip_R"), TEXT("knee_R"), TEXT("ankle_R"),
		TEXT("shoulder_L"), TEXT("elbow_L"), TEXT("wrist_L"), TEXT("shoulder_R"), TEXT("elbow_R"), TEXT("wrist_R") })
	{
		if (File.Joint(J) == INDEX_NONE)
		{
			OutError = FString::Printf(TEXT("the file has no joint %s"), J);
			return false;
		}
	}
	for (const TCHAR* P : { TEXT("knob"), TEXT("sweet"), TEXT("tip"), TEXT("grip_L"), TEXT("grip_R") })
	{
		if (File.Point(P) == INDEX_NONE)
		{
			OutError = FString::Printf(TEXT("the file has no bat point %s"), P);
			return false;
		}
	}
	for (const TCHAR* C : { TEXT("L_ball_in"), TEXT("L_ball_out"), TEXT("R_ball_in"), TEXT("R_ball_out") })
	{
		if (File.Contact(C) == INDEX_NONE)
		{
			OutError = FString::Printf(TEXT("the file has no contact point %s"), C);
			return false;
		}
	}
	// The mannequin's joints as it stands when made: the pieces were placed from them.
	bool bOk = true;
	auto M = [&](const TCHAR* Bone)
	{
		FTransform T;
		if (!Shop->RefBone(FName(Bone), T))
		{
			bOk = false;
			OutError = FString::Printf(TEXT("the mannequin has no bone %s"), Bone);
		}
		return T.GetLocation();
	};
	const FVector Up(0, 0, 1);
	FVector Lumbar = FVector::ZeroVector;
	const double Fwd = M(TEXT("ball_l")).Y >= M(TEXT("foot_l")).Y ? 1.0 : -1.0;
	const FVector Ahead(0, Fwd, 0), Across(1, 0, 0);
	Segs.SetNum(SegNumSeg);
	// A segment from Near to Far on the mannequin; the model's from its near joint to its far one.
	auto Limb = [&](EBbSwingSeg S, const FVector& Near, const FVector& Far, const TCHAR* Body, int32 NearJ, int32 FarJ, int32 FarP = INDEX_NONE)
	{
		FBbSwingSeg& G = Segs[S];
		G.O = Near;
		G.A = (Far - Near).GetSafeNormal();
		G.Len = (Far - Near).Size();
		G.F = SquareTo(G.A, Ahead, Across);
		G.L = G.A ^ G.F;
		G.Body = File.Body(Body);
		G.Near = NearJ;
		G.Far = FarJ;
		G.FarPoint = FarP;
	};
	const TCHAR* Sd[2] = { TEXT("l"), TEXT("r") };
	const TCHAR* SD[2] = { TEXT("L"), TEXT("R") };
	auto J = [this](const FString& N) { return File.Joint(*N); };
	for (int32 K = 0; K < 2; ++K)
	{
		const FString l = Sd[K], R = SD[K];
		const FVector Hip = M(*(TEXT("thigh_") + l)), Knee = M(*(TEXT("calf_") + l)), Ankle = M(*(TEXT("foot_") + l)), Ball = M(*(TEXT("ball_") + l));
		const FVector Shoulder = M(*(TEXT("upperarm_") + l)), Elbow = M(*(TEXT("lowerarm_") + l)), Wrist = M(*(TEXT("hand_") + l)), Knuckle = M(*(TEXT("middle_01_") + l));
		const int32 Off = K == 0 ? 0 : 3;
		Limb((EBbSwingSeg)(SegThighL + Off), Hip, Knee, *(TEXT("thigh_") + R), J(TEXT("hip_") + R), J(TEXT("knee_") + R));
		Limb((EBbSwingSeg)(SegShankL + Off), Knee, Ankle, *(TEXT("shank_") + R), J(TEXT("knee_") + R), J(TEXT("ankle_") + R));
		const int32 AOff = K == 0 ? 0 : 3;
		Limb((EBbSwingSeg)(SegUarmL + AOff), Shoulder, Elbow, *(TEXT("uarm_") + R), J(TEXT("shoulder_") + R), J(TEXT("elbow_") + R));
		Limb((EBbSwingSeg)(SegFarmL + AOff), Elbow, Wrist, *(TEXT("farm_") + R), J(TEXT("elbow_") + R), J(TEXT("wrist_") + R));
		// The hand is part of the bat in the model: from the wrist to the handle's axis.
		Limb((EBbSwingSeg)(SegHandL + AOff), Wrist, Knuckle, TEXT("bat"), J(TEXT("wrist_") + R), INDEX_NONE, File.Point(*(TEXT("grip_") + R)));
		// The foot: up its own vertical from the ankle, forward along the shoe to the ball of the foot.
		FBbSwingSeg& G = Segs[SegFootL + Off];
		G.O = Ankle;
		G.A = Up;
		G.F = SquareTo(Up, Ball - Ankle, Ahead);
		G.L = G.A ^ G.F;
		G.Len = Ankle.Z;                                   // the ankle above the ground (the mesh stands on z = 0)
		G.Fwd = (Ball - Ankle) | G.F;
		G.Body = File.Body(*(TEXT("foot_") + R));
		G.Near = J(TEXT("ankle_") + R);
	}
	{
		const FVector HipL = M(TEXT("thigh_l")), HipR = M(TEXT("thigh_r")), Spine = M(TEXT("spine_01"));
		const FVector ShL = M(TEXT("upperarm_l")), ShR = M(TEXT("upperarm_r"));
		// The model's lumbar joint is at the navel (de Leva's lower trunk, omphalion to the hips),
		// higher up the back than the mannequin's spine_01: its point on the mannequin is put on
		// the line from between his hips to between his shoulders, as far up it as the model's is
		// up its own (at the file's first sample), so that the pelvis and the trunk are stretched
		// alike to the model's lengths.
		const FVector HipMid = (HipL + HipR) * 0.5, ShMid = (ShL + ShR) * 0.5;
		const double PelvisLen = (File.JointPos(0, J(TEXT("lumbar"))) - (File.JointPos(0, J(TEXT("hip_L"))) + File.JointPos(0, J(TEXT("hip_R")))) * 0.5).Size();
		const double TrunkLen = ((File.JointPos(0, J(TEXT("shoulder_L"))) + File.JointPos(0, J(TEXT("shoulder_R")))) * 0.5 - File.JointPos(0, J(TEXT("lumbar")))).Size();
		Lumbar = HipMid + (ShMid - HipMid) * (PelvisLen / FMath::Max(1e-6, PelvisLen + TrunkLen));
		UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING the model's lumbar joint on the mannequin: %.1f cm above his hips (his spine_01 is %.1f above them)"),
			Lumbar.Z - HipMid.Z, Spine.Z - HipMid.Z);
		Limb(SegPelvis, HipMid, Lumbar, TEXT("pelvis"), INDEX_NONE, J(TEXT("lumbar")));
		Segs[SegPelvis].Wide = (HipL - HipR).Size();
		Segs[SegPelvis].WideA = J(TEXT("hip_L"));
		Segs[SegPelvis].WideB = J(TEXT("hip_R"));
		Limb(SegTrunk, Lumbar, ShMid, TEXT("trunk"), J(TEXT("lumbar")), INDEX_NONE);
		Segs[SegTrunk].Wide = (ShL - ShR).Size();
		Segs[SegTrunk].WideA = J(TEXT("shoulder_L"));
		Segs[SegTrunk].WideB = J(TEXT("shoulder_R"));
	}
	// A recorded swing's head turns on its own (the file's head body): about the mannequin's head
	// bone, its forward the mannequin's ahead.
	HeadBody = File.Body(TEXT("head"));
	HeadPivotMesh = M(TEXT("head"));
	HeadAheadMesh = Ahead;
	if (!bOk)
	{
		return false;
	}

	// The joints each segment carries: the file's joint and the mannequin's.
	TArray<TArray<TPair<int32, FVector>>> Carried;
	Carried.SetNum(SegNumSeg);
	auto Carry = [&Carried](int32 S, int32 FileJoint, const FVector& MeshJoint) { Carried[S].Emplace(FileJoint, MeshJoint); };
	Carry(SegPelvis, J(TEXT("hip_L")), M(TEXT("thigh_l")));
	Carry(SegPelvis, J(TEXT("hip_R")), M(TEXT("thigh_r")));
	Carry(SegPelvis, J(TEXT("lumbar")), Lumbar);
	Carry(SegTrunk, J(TEXT("lumbar")), Lumbar);
	Carry(SegTrunk, J(TEXT("shoulder_L")), M(TEXT("upperarm_l")));
	Carry(SegTrunk, J(TEXT("shoulder_R")), M(TEXT("upperarm_r")));
	for (int32 K = 0; K < 2; ++K)
	{
		const FString l = Sd[K], R = SD[K];
		const int32 Off = K == 0 ? 0 : 3;
		Carry(SegThighL + Off, J(TEXT("hip_") + R), M(*(TEXT("thigh_") + l)));
		Carry(SegThighL + Off, J(TEXT("knee_") + R), M(*(TEXT("calf_") + l)));
		Carry(SegShankL + Off, J(TEXT("knee_") + R), M(*(TEXT("calf_") + l)));
		Carry(SegShankL + Off, J(TEXT("ankle_") + R), M(*(TEXT("foot_") + l)));
		Carry(SegFootL + Off, J(TEXT("ankle_") + R), M(*(TEXT("foot_") + l)));
		Carry(SegUarmL + Off, J(TEXT("shoulder_") + R), M(*(TEXT("upperarm_") + l)));
		Carry(SegUarmL + Off, J(TEXT("elbow_") + R), M(*(TEXT("lowerarm_") + l)));
		Carry(SegFarmL + Off, J(TEXT("elbow_") + R), M(*(TEXT("lowerarm_") + l)));
		Carry(SegFarmL + Off, J(TEXT("wrist_") + R), M(*(TEXT("hand_") + l)));
		Carry(SegHandL + Off, J(TEXT("wrist_") + R), M(*(TEXT("hand_") + l)));
	}

	// The pieces: a batter's (his helmet, bare hands), each on its segment.
	struct FOnSeg
	{
		const TCHAR* Bone;
		int32 Seg;
		bool bRigid;
	};
	static const FOnSeg OnSeg[] = {
		{ TEXT("pelvis"), SegPelvis, false }, { TEXT("spine_02"), SegTrunk, false }, { TEXT("spine_04"), SegTrunk, false },
		{ TEXT("neck_01"), SegTrunk, true }, { TEXT("head"), SegTrunk, true },
		{ TEXT("thigh_l"), SegThighL, false }, { TEXT("calf_l"), SegShankL, false }, { TEXT("foot_l"), SegFootL, false },
		{ TEXT("thigh_r"), SegThighR, false }, { TEXT("calf_r"), SegShankR, false }, { TEXT("foot_r"), SegFootR, false },
		{ TEXT("upperarm_l"), SegUarmL, false }, { TEXT("lowerarm_l"), SegFarmL, false }, { TEXT("hand_l"), SegHandL, false },
		{ TEXT("upperarm_r"), SegUarmR, false }, { TEXT("lowerarm_r"), SegFarmR, false }, { TEXT("hand_r"), SegHandR, false } };
	for (const FBbPiece& P : Shop->Pieces())
	{
		if (P.Wear != EBbWear::Always && P.Wear != EBbWear::Helmet && P.Wear != EBbWear::HelmetBill)
		{
			continue;
		}
		const FOnSeg* S = nullptr;
		for (const FOnSeg& O : OnSeg)
		{
			if (P.Bone == FName(O.Bone))
			{
				S = &O;
			}
		}
		FTransform BoneRef;
		if (!S || !Shop->RefBone(P.Bone, BoneRef))
		{
			UE_LOG(LogBbSwing, Warning, TEXT("BB_BODYSWING a piece on bone %s has no segment: not drawn"), *P.Bone.ToString());
			continue;
		}
		FBbSwingPart Part;
		Part.Seg = S->Seg;
		Part.bRigid = S->bRigid;
		Part.bHead = P.Bone == FName(TEXT("head"));
		Part.Ref = P.OnBone * BoneRef;
		if (!Part.bRigid)
		{
			for (const TPair<int32, FVector>& C : Carried[Part.Seg])
			{
				Part.Carries.Emplace(C.Key, Part.Ref.InverseTransformPosition(C.Value));
			}
		}
		if (Part.Seg == SegHandL || Part.Seg == SegHandR)
		{
			const FString l = Part.Seg == SegHandL ? TEXT("l") : TEXT("r");
			Part.Grip = Segs[Part.Seg].FarPoint;
			Part.GripLocal = Part.Ref.InverseTransformPosition(M(*(TEXT("middle_01_") + l)));
		}
		if (Part.Seg == SegFootL || Part.Seg == SegFootR)
		{
			const FString l = Part.Seg == SegFootL ? TEXT("l") : TEXT("r"), R = Part.Seg == SegFootL ? TEXT("L") : TEXT("R");
			const FVector Ball = M(*(TEXT("ball_") + l));
			Part.SoleLocal = Part.Ref.InverseTransformPosition(FVector(Ball.X, Ball.Y, 0.0));
			Part.BallIn = File.Contact(*(R + TEXT("_ball_in")));
			Part.BallOut = File.Contact(*(R + TEXT("_ball_out")));
		}
		UStaticMeshComponent* C = NewObject<UStaticMeshComponent>(this);
		C->SetMobility(EComponentMobility::Movable);
		C->SetStaticMesh(Shop->MeshOf(P));
		C->SetMaterial(0, Shop->PaintFor(P.Cloth, 1, 1, 0));
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		C->SetupAttachment(RootComponent);
		C->RegisterComponent();
		Part.C = C;
		Parts.Add(Part);
	}
	BatC = NewObject<UStaticMeshComponent>(this);
	BatC->SetMobility(EComponentMobility::Movable);
	BatC->SetStaticMesh(Shop->BatMesh());
	BatC->SetMaterial(0, Shop->BatPaint());
	BatC->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	BatC->SetupAttachment(RootComponent);
	BatC->RegisterComponent();
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING %d pieces; the mannequin's segments against the file's at its first sample (cm): thigh %.1f/%.1f shank %.1f/%.1f upper arm %.1f/%.1f forearm %.1f/%.1f hand %.1f/%.1f pelvis %.1f/%.1f wide %.1f/%.1f trunk %.1f/%.1f wide %.1f/%.1f"),
		Parts.Num(), Segs[SegThighL].Len, (ToCm(File.JointPos(0, Segs[SegThighL].Near)) - ToCm(File.JointPos(0, Segs[SegThighL].Far))).Size(),
		Segs[SegShankL].Len, (ToCm(File.JointPos(0, Segs[SegShankL].Near)) - ToCm(File.JointPos(0, Segs[SegShankL].Far))).Size(),
		Segs[SegUarmL].Len, (ToCm(File.JointPos(0, Segs[SegUarmL].Near)) - ToCm(File.JointPos(0, Segs[SegUarmL].Far))).Size(),
		Segs[SegFarmL].Len, (ToCm(File.JointPos(0, Segs[SegFarmL].Near)) - ToCm(File.JointPos(0, Segs[SegFarmL].Far))).Size(),
		Segs[SegHandL].Len, (ToCm(File.JointPos(0, Segs[SegHandL].Near)) - ToCm(File.PointPos(0, Segs[SegHandL].FarPoint))).Size(),
		Segs[SegPelvis].Len, (ToCm(File.JointPos(0, Segs[SegPelvis].Far)) - ToCm((File.JointPos(0, Segs[SegPelvis].WideA) + File.JointPos(0, Segs[SegPelvis].WideB)) * 0.5)).Size(),
		Segs[SegPelvis].Wide, (ToCm(File.JointPos(0, Segs[SegPelvis].WideA)) - ToCm(File.JointPos(0, Segs[SegPelvis].WideB))).Size(),
		Segs[SegTrunk].Len, (ToCm(File.JointPos(0, Segs[SegTrunk].Near)) - ToCm((File.JointPos(0, Segs[SegTrunk].WideA) + File.JointPos(0, Segs[SegTrunk].WideB)) * 0.5)).Size(),
		Segs[SegTrunk].Wide, (ToCm(File.JointPos(0, Segs[SegTrunk].WideA)) - ToCm(File.JointPos(0, Segs[SegTrunk].WideB))).Size());
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING the mannequin's foot: ankle %.2f cm above the sole, ball of the foot %.2f cm ahead of the ankle (ratio %.4f)"),
		Segs[SegFootL].Len, Segs[SegFootL].Fwd, Segs[SegFootL].Fwd / FMath::Max(1e-6, Segs[SegFootL].Len));
	return true;
}

void ABbBodySwing::Sample(double T, FBbSwingPose& Out) const
{
	const int32 N = File.Num();
	T = FMath::Clamp(T, File.T(0), File.T(N - 1));
	int32 I = FMath::Clamp((int32)FMath::FloorToDouble((T - File.T(0)) / (File.T(1) - File.T(0))), 0, N - 2);
	while (I > 0 && File.T(I) > T)
	{
		--I;
	}
	while (I < N - 2 && File.T(I + 1) <= T)
	{
		++I;
	}
	const double A = FMath::Clamp((T - File.T(I)) / (File.T(I + 1) - File.T(I)), 0.0, 1.0);
	auto Lerp = [A](const FVector& P, const FVector& Q) { return A == 0.0 ? P : (A == 1.0 ? Q : P + (Q - P) * A); };
	Out.BodyPos.SetNum(File.Bodies.Num());
	Out.BodyRot.SetNum(File.Bodies.Num());
	for (int32 B = 0; B < File.Bodies.Num(); ++B)
	{
		Out.BodyPos[B] = Lerp(File.BodyPos(I, B), File.BodyPos(I + 1, B));
		Out.BodyRot[B] = A == 0.0 ? File.BodyRot(I, B) : A == 1.0 ? File.BodyRot(I + 1, B) : FQuat::Slerp(File.BodyRot(I, B), File.BodyRot(I + 1, B), A);
	}
	Out.JointPos.SetNum(File.Joints.Num());
	for (int32 J = 0; J < File.Joints.Num(); ++J)
	{
		Out.JointPos[J] = Lerp(File.JointPos(I, J), File.JointPos(I + 1, J));
	}
	Out.PointPos.SetNum(File.Points.Num());
	for (int32 P = 0; P < File.Points.Num(); ++P)
	{
		Out.PointPos[P] = Lerp(File.PointPos(I, P), File.PointPos(I + 1, P));
	}
	Out.ContactPos.SetNum(File.Contacts.Num());
	for (int32 C = 0; C < File.Contacts.Num(); ++C)
	{
		Out.ContactPos[C] = Lerp(File.ContactPos(I, C), File.ContactPos(I + 1, C));
	}
	Out.Ball = Lerp(File.BallPos(I), File.BallPos(I + 1));
	Out.Sweet = File.SweetMps(I) + (File.SweetMps(I + 1) - File.SweetMps(I)) * A;
}

void ABbBodySwing::Pose(const FBbSwingPose& P)
{
	// Each segment as the model has it now: from its near joint to its far one, turned as its body is.
	auto Mid = [&P](int32 A, int32 B) { return (P.JointPos[A] + P.JointPos[B]) * 0.5; };
	for (int32 S = 0; S < SegNumSeg; ++S)
	{
		FBbSwingSeg& G = Segs[S];
		if (S == SegFootL || S == SegFootR)
		{
			const int32 Bi = S == SegFootL ? File.Contact(TEXT("L_ball_in")) : File.Contact(TEXT("R_ball_in"));
			const int32 Bo = S == SegFootL ? File.Contact(TEXT("L_ball_out")) : File.Contact(TEXT("R_ball_out"));
			const FVector BallM = ToCm((P.ContactPos[Bi] + P.ContactPos[Bo]) * 0.5);
			G.OM = ToCm(P.JointPos[G.Near]);
			G.AM = AxisUE(P, G.Body, FVector(0, 0, 1));
			G.FM = SquareTo(G.AM, AxisUE(P, G.Body, FVector(1, 0, 0)), G.AM);
			G.LM = G.AM ^ G.FM;
			G.SA = ((G.OM - BallM) | G.AM) / G.Len;
			G.SF = ((BallM - G.OM) | G.FM) / G.Fwd;
			G.SL = 1.0;
			continue;
		}
		const FVector Near = G.Near != INDEX_NONE ? P.JointPos[G.Near] : Mid(G.WideA, G.WideB);
		const FVector Far = G.Far != INDEX_NONE ? P.JointPos[G.Far] : G.FarPoint != INDEX_NONE ? P.PointPos[G.FarPoint] : Mid(G.WideA, G.WideB);
		G.OM = ToCm(Near);
		const FVector D = ToCm(Far) - G.OM;
		G.AM = D.GetSafeNormal();
		G.SA = D.Size() / G.Len;
		G.FM = SquareTo(G.AM, AxisUE(P, G.Body, FVector(1, 0, 0)), AxisUE(P, G.Body, FVector(0, 1, 0)));
		if (Fault == TEXT("nostretch") && S != SegPelvis && S != SegTrunk)
		{
			G.SA = 1.0;                                     // planted: the limb left at the mannequin's length
		}
		if (Fault == TEXT("twist") && S != SegPelvis && S != SegTrunk)
		{
			G.FM = G.AM ^ G.FM;                             // planted: the limb turned 90 degrees about its own axis
		}
		G.LM = G.AM ^ G.FM;
		G.SF = 1.0;
		G.SL = G.Wide > 0.0 ? (ToCm(P.JointPos[G.WideA]) - ToCm(P.JointPos[G.WideB])).Size() / G.Wide : 1.0;
	}
	for (const FBbSwingPart& Part : Parts)
	{
		const FBbSwingSeg& G = Segs[Part.Seg];
		FTransform W;
		if (Part.bRigid)
		{
			// Neck and head ride on the trunk's far end (between the shoulders) as they are: the
			// model has no neck, so they are neither scaled nor turned on their own.
			const FQuat Turn = FQuat(FMatrix(G.FM, G.LM, G.AM, FVector::ZeroVector)) * FQuat(FMatrix(G.F, G.L, G.A, FVector::ZeroVector)).Inverse();
			const FVector FarMesh = G.O + G.A * G.Len, FarModel = G.OM + G.AM * G.Len * G.SA;
			W = FTransform(Turn * Part.Ref.GetRotation(), FarModel + Turn.RotateVector(Part.Ref.GetLocation() - FarMesh));
			if (Part.bHead && HeadBody != INDEX_NONE)
			{
				// The head turned as the file's head is (its x the face's forward, its z up through the
				// crown), about the head bone where the trunk carries it.
				const FVector HA = AxisUE(P, HeadBody, FVector(0, 0, 1)).GetSafeNormal();
				const FVector HF = SquareTo(HA, AxisUE(P, HeadBody, FVector(1, 0, 0)), HA);
				const FVector MA(0, 0, 1), MF = SquareTo(MA, HeadAheadMesh, FVector(1, 0, 0));
				const FQuat HeadTurn = FQuat(FMatrix(HF, HA ^ HF, HA, FVector::ZeroVector)) * FQuat(FMatrix(MF, MA ^ MF, MA, FVector::ZeroVector)).Inverse();
				const FVector Pivot = FarModel + Turn.RotateVector(HeadPivotMesh - FarMesh);
				W = FTransform(HeadTurn * Part.Ref.GetRotation(), Pivot + HeadTurn.RotateVector(Part.Ref.GetLocation() - HeadPivotMesh));
			}
		}
		else
		{
			// The piece's origin and its own axis land exactly where the segment's map takes them;
			// across it the piece is scaled as the segment is.
			const FVector X = MapVector(G, Part.Ref.GetUnitAxis(EAxis::X)), Y = MapVector(G, Part.Ref.GetUnitAxis(EAxis::Y)), Z = MapVector(G, Part.Ref.GetUnitAxis(EAxis::Z));
			W = FTransform(FRotationMatrix::MakeFromZX(Z, X).ToQuat(), MapPoint(G, Part.Ref.GetLocation()), FVector(X.Size(), Y.Size(), Z.Size()));
		}
		Part.C->SetWorldTransform(W);
	}
	// The bat (with the hands on it, one body in the model): from the file's knob to its tip.
	{
		const FVector Knob = ToCm(P.PointPos[File.Point(TEXT("knob"))]), Tip = ToCm(P.PointPos[File.Point(TEXT("tip"))]);
		const FVector Z = Tip - Knob;
		BatC->SetWorldTransform(FTransform(FRotationMatrix::MakeFromZX(Z, AxisUE(P, File.Body(TEXT("bat")), FVector(1, 0, 0))).ToQuat(), Knob,
			FVector(1.0, 1.0, Z.Size() / UBbFigureShop::BatCm)));
	}
	if (BallC)
	{
		BallC->SetWorldLocation(ToCm(P.Ball));
	}
}

// ---------------------------------------------------------------------------------------------
// Start, the check, playing

int32 ABbBodySwing::Start(const FString& Path, UBbFigureShop* InShop, ABbPark* InPark, ACameraActor* InCamera, UStaticMesh* Sphere, UMaterialInterface* BaseMaterial)
{
	Shop = InShop;
	Park = InPark;
	Camera = InCamera;
	const TCHAR* Cmd = FCommandLine::Get();
	FParse::Value(Cmd, TEXT("BbBodySwingCheckFault="), Fault);
	const bool bCheck = FParse::Param(Cmd, TEXT("BbBodySwingCheck")) || !Fault.IsEmpty();
	if (!Fault.IsEmpty() && Fault != TEXT("nostretch") && Fault != TEXT("mirror") && Fault != TEXT("twist"))
	{
		UE_LOG(LogBbSwing, Error, TEXT("BB_BODYCHECK result FAIL: no such fault %s (nostretch, mirror or twist)"), *Fault);
		return 1;
	}
	// -BbBodySwingShots: the eight fixed instants from two sides. -BbBodySwingShots=<ms>: a picture every <ms> from the
	// model's first instant to 60 ms after contact (whole multiples of <ms> counted from contact), each from the
	// pitcher's side, the open side and above (player-visuals §14).
	float StepMs = 0.f;
	const bool bStep = FParse::Value(Cmd, TEXT("BbBodySwingShots="), StepMs) && StepMs >= 1.f;
	const bool bShots = bStep || FParse::Param(Cmd, TEXT("BbBodySwingShots"));
	GroundCm = Park ? 100.0 * Park->GroundZ(0.0, 0.4) : 0.0;
	bool bOk = File.Load(Path, Error);
	if (bOk && !Shop)
	{
		Error = TEXT("no figure to draw: the mannequin assets are missing (client/run.sh copies them) or -BbMannequin was given");
		bOk = false;
	}
	if (bOk)
	{
		bOk = BuildFigure(Error);
	}
	if (!bOk)
	{
		UE_LOG(LogBbSwing, Error, TEXT("BB_BODYSWING cannot show %s: %s"), *Path, *Error);
		if (bCheck || bShots)
		{
			UE_LOG(LogBbSwing, Error, TEXT("BB_BODYCHECK result FAIL: the file will not open (%s)"), *Error);
			return 1;
		}
		UpdateText();
		return -1;
	}
	BallC = NewObject<UStaticMeshComponent>(this);
	BallC->SetMobility(EComponentMobility::Movable);
	BallC->SetStaticMesh(Sphere);
	BallC->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	BallC->SetWorldScale3D(FVector(BallCm / 100.0));
	if (BaseMaterial)
	{
		UMaterialInstanceDynamic* Paint = UMaterialInstanceDynamic::Create(BaseMaterial, this);
		Paint->SetVectorParameterValue(TEXT("Color"), FLinearColor(1.f, 1.f, 0.92f));
		BallC->SetMaterial(0, Paint);
	}
	BallC->SetupAttachment(RootComponent);
	BallC->RegisterComponent();
	BallC->SetWorldScale3D(FVector(BallCm / 100.0));
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING %s: %d samples from %.1f to %.1f ms (contact 0), plan %s, commit %s, contact %.2f m/s (%.1f mph), missed by %.2f cm"),
		*File.Name, File.Num(), 1e3 * File.T(0), 1e3 * File.T(File.Num() - 1), *File.Plan, *File.Commit, File.ContactMps, File.ContactMph, File.MissCm);
	if (bCheck)
	{
		return RunCheck();
	}
	// What the cameras look at: between his hips at the start and the meeting point, at the belt.
	{
		double Contact = 0.0;
		File.Event(TEXT("contact"), Contact);
		FBbSwingPose AtContact;
		Sample(Contact, AtContact);
		const FVector Hips = File.BodyPos(0, File.Body(TEXT("pelvis")));
		Target = FVector((Hips.X + AtContact.Ball.X) * 0.5, (Hips.Y + AtContact.Ball.Y) * 0.5, 1.0);
	}
	Clock = File.T(0);
	HoldStart = HoldStartS;
	if (bShots)
	{
		if (bStep)
		{
			// Every StepMs, from the file's first instant to 60 ms after contact (a recorded swing: to its last
			// sample, the end of the finish), from three sides.
			ShotViews = 3;
			ShotBefore = 0.6;
			ShotAfter = 0.3;
			const double LastMs = File.bRecorded ? 1e3 * File.T(File.Num() - 1) : FMath::Min(60.0, 1e3 * File.T(File.Num() - 1));
			for (int32 K = (int32)FMath::CeilToDouble(1e3 * File.T(0) / StepMs - 1e-6); K * StepMs <= LastMs + 1e-6; ++K)
			{
				const int32 Ms = (int32)FMath::RoundToDouble(K * StepMs);
				ShotTimes.Emplace(FString::Printf(TEXT("%s%03dms"), Ms < 0 ? TEXT("m") : TEXT("p"), FMath::Abs(Ms)), 1e-3 * K * StepMs);
			}
		}
		else
		{
			// The fixed instants, each from the pitcher's side and from the open side.
			for (const TCHAR* E : { TEXT("start"), TEXT("launch"), TEXT("front_foot_down") })
			{
				double T;
				if (File.Event(E, T))
				{
					ShotTimes.Emplace(E, T);
				}
			}
			ShotTimes.Emplace(TEXT("m100ms"), -0.100);
			ShotTimes.Emplace(TEXT("m050ms"), -0.050);
			ShotTimes.Emplace(TEXT("m020ms"), -0.020);
			ShotTimes.Emplace(TEXT("contact"), 0.0);
			ShotTimes.Emplace(TEXT("end"), File.T(File.Num() - 1));
		}
		Shot = 0;
	}
	SetView(0);
	FBbSwingPose P;
	Sample(Clock, P);
	Pose(P);
	UpdateText();
	return -1;
}

int32 ABbBodySwing::RunCheck()
{
	// The sim-to-Unreal map written out again here, apart from ToCm and AxisUE, so that a slip in
	// those is caught (the frames: AGENTS.md "Units and frames"; BbFrame.h): Unreal X is the sim's Y
	// (toward the pitcher), Unreal Y the sim's X (toward first base), Unreal Z up; centimetres; the
	// file's ground (z = 0) on the drawn ground under the plate.
	auto UeVec = [](const FVector& S) { return FVector(S.Y, S.X, S.Z); };
	auto Ue = [this](const FVector& S) { return FVector(100.0 * S.Y, 100.0 * S.X, 100.0 * S.Z + GroundCm); };
	auto SimOf = [this](const FVector& U) { return FVector(U.Y / 100.0, U.X / 100.0, (U.Z - GroundCm) / 100.0); };
	auto Across = [](const FVector& V, const FVector& Axis) { return (V - Axis * (V | Axis)).GetSafeNormal(); };
	auto Deg = [](const FVector& A, const FVector& B) { return FMath::RadiansToDegrees(FMath::Acos(FMath::Clamp(A | B, -1.0, 1.0))); };

	TArray<double> JointWorst, JointAt;
	JointWorst.Init(0.0, File.Joints.Num());
	JointAt.Init(0.0, File.Joints.Num());
	TArray<double> TurnWorst, TurnAt;
	TurnWorst.Init(0.0, SegNumSeg);
	TurnAt.Init(0.0, SegNumSeg);
	double KnobWorst = 0.0, KnobAt = 0.0, SweetWorst = 0.0, SweetAt = 0.0, RollWorst = 0.0, RollAt = 0.0;
	double GripWorst[2] = { 0.0, 0.0 }, GripAt[2] = { 0.0, 0.0 }, SoleWorst[2] = { 0.0, 0.0 };
	int32 LeftWrong = 0;
	double LeftFirstWrong = 0.0;
	bool bStartRight = true;
	FString StartNote;
	const int32 Knob = File.Point(TEXT("knob")), Sweet = File.Point(TEXT("sweet")), Tip = File.Point(TEXT("tip")), BatBody = File.Body(TEXT("bat"));
	const double SweetLocal = 100.0 * File.SweetFromKnobM * UBbFigureShop::BatCm / (100.0 * File.BatLengthM);
	int32 Measured = 0, Turns = 0;
	static const TCHAR* const SegNames[] = { TEXT("pelvis"), TEXT("trunk"), TEXT("thigh_L"), TEXT("shank_L"), TEXT("foot_L"), TEXT("thigh_R"), TEXT("shank_R"),
		TEXT("foot_R"), TEXT("uarm_L"), TEXT("farm_L"), TEXT("hand_L"), TEXT("uarm_R"), TEXT("farm_R"), TEXT("hand_R") };
	for (int32 I = 0; I < File.Num(); ++I)
	{
		FBbSwingPose P;
		Sample(File.T(I), P);
		Pose(P);
		const double Ms = 1e3 * File.T(I);
		auto Worse = [Ms](double D, double& Worst, double& At)
		{
			if (D > Worst)
			{
				Worst = D;
				At = Ms;
			}
		};
		// The drawn hips and ankles of each leg, from the leg's own pieces, for the handedness tests.
		FVector Hip[2], Ankle[2], PelvisFwd = FVector::ZeroVector;
		for (const FBbSwingPart& Part : Parts)
		{
			const FTransform& W = Part.C->GetComponentTransform();
			for (const TPair<int32, FVector>& C : Part.Carries)
			{
				Worse(10.0 * (W.TransformPosition(C.Value) - Ue(File.JointPos(I, C.Key))).Size(), JointWorst[C.Key], JointAt[C.Key]);
				++Measured;
			}
			if (Part.Seg == SegThighL || Part.Seg == SegThighR)
			{
				Hip[Part.Seg == SegThighL ? 0 : 1] = SimOf(W.TransformPosition(Part.Carries[0].Value));
			}
			if (Part.Seg == SegShankL || Part.Seg == SegShankR)
			{
				Ankle[Part.Seg == SegShankL ? 0 : 1] = SimOf(W.TransformPosition(Part.Carries[1].Value));
			}
			if (Part.Grip != INDEX_NONE)
			{
				const int32 K = Part.Seg == SegHandL ? 0 : 1;
				Worse(10.0 * (W.TransformPosition(Part.GripLocal) - Ue(File.PointPos(I, Part.Grip))).Size(), GripWorst[K], GripAt[K]);
			}
			if (Part.BallIn != INDEX_NONE)
			{
				const int32 K = Part.Seg == SegFootL ? 0 : 1;
				double Unused = 0.0;
				Worse(10.0 * (W.TransformPosition(Part.SoleLocal) - Ue((File.ContactPos(I, Part.BallIn) + File.ContactPos(I, Part.BallOut)) * 0.5)).Size(), SoleWorst[K], Unused);
			}
			if (Part.bRigid)
			{
				continue;
			}
			// The segment's forward as drawn (the piece's image of the mannequin's forward, square to
			// the piece's image of the segment's axis) against the file's: its body's x turned by the
			// file's quaternion, square to the segment's axis from the file's joints (the foot: its z).
			const FBbSwingSeg& G = Segs[Part.Seg];
			const FVector DrawnA = W.TransformVector(Part.Ref.InverseTransformVectorNoScale(G.A)).GetSafeNormal();
			const FVector DrawnF = Across(W.TransformVector(Part.Ref.InverseTransformVectorNoScale(G.F)), DrawnA);
			const FQuat Q = File.BodyRot(I, G.Body);
			FVector FileA;
			if (Part.Seg == SegFootL || Part.Seg == SegFootR)
			{
				FileA = UeVec(Q.RotateVector(FVector(0, 0, 1)));
			}
			else
			{
				auto Mid = [&](int32 A, int32 B) { return (File.JointPos(I, A) + File.JointPos(I, B)) * 0.5; };
				const FVector Near = G.Near != INDEX_NONE ? File.JointPos(I, G.Near) : Mid(G.WideA, G.WideB);
				const FVector Far = G.Far != INDEX_NONE ? File.JointPos(I, G.Far) : G.FarPoint != INDEX_NONE ? File.PointPos(I, G.FarPoint) : Mid(G.WideA, G.WideB);
				FileA = (Ue(Far) - Ue(Near)).GetSafeNormal();
			}
			const FVector FileF = Across(UeVec(Q.RotateVector(FVector(1, 0, 0))), FileA);
			Worse(Deg(DrawnF, FileF), TurnWorst[Part.Seg], TurnAt[Part.Seg]);
			++Turns;
			if (Part.Seg == SegPelvis)
			{
				PelvisFwd = SimOf(Ue(FVector::ZeroVector) + DrawnF * 100.0) - SimOf(Ue(FVector::ZeroVector));
			}
		}
		const FTransform& B = BatC->GetComponentTransform();
		Worse(10.0 * (B.TransformPosition(FVector::ZeroVector) - Ue(File.PointPos(I, Knob))).Size(), KnobWorst, KnobAt);
		Worse(10.0 * (B.TransformPosition(FVector(0, 0, SweetLocal)) - Ue(File.PointPos(I, Sweet))).Size(), SweetWorst, SweetAt);
		{
			const FVector FileA = (Ue(File.PointPos(I, Tip)) - Ue(File.PointPos(I, Knob))).GetSafeNormal();
			Worse(Deg(Across(B.GetUnitAxis(EAxis::X), FileA), Across(UeVec(File.BodyRot(I, BatBody).RotateVector(FVector(1, 0, 0))), FileA)), RollWorst, RollAt);
		}
		// Handedness, read from the drawing in the sim's frame: the leg drawn as his left (the
		// mannequin's left leg) is on his own left as he is drawn facing (left = up x forward in the
		// sim's right-handed frame), at every sample.
		const FVector Left = FVector(0, 0, 1) ^ FVector(PelvisFwd.X, PelvisFwd.Y, 0.0).GetSafeNormal();
		if (((Hip[0] - Hip[1]) | Left) <= 0.0)
		{
			if (LeftWrong++ == 0)
			{
				LeftFirstWrong = Ms;
			}
		}
		if (I == 0)
		{
			// At the start: a right-hander on the third-base side of the plate (-X), facing it (+X),
			// his lead (left) side toward the pitcher (+Y).
			const FVector Hips = (Hip[0] + Hip[1]) * 0.5;
			bStartRight = Hips.X < 0.0 && PelvisFwd.X > 0.0 && Ankle[0].Y > Ankle[1].Y;
			StartNote = FString::Printf(TEXT("hips at x %+.3f m (wants < 0), facing (%+.2f, %+.2f) (wants x > 0), lead ankle y %+.3f against rear %+.3f (wants lead > rear)"),
				Hips.X, PelvisFwd.X, PelvisFwd.Y, Ankle[0].Y, Ankle[1].Y);
		}
	}
	double Largest = FMath::Max3(KnobWorst, SweetWorst, FMath::Max(GripWorst[0], GripWorst[1]));
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK file=%s samples=%d joint_readings=%d turn_readings=%d limit_mm %.1f turn_limit_deg %.1f fault=%s (positions through the check's own sim-to-Unreal map)"),
		*File.Name, File.Num(), Measured, Turns, CheckLimitMm, TurnLimitDeg, Fault.IsEmpty() ? TEXT("none") : *Fault);
	for (int32 J = 0; J < File.Joints.Num(); ++J)
	{
		UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK joint %-10s max_mm %.4f at_ms %+.1f"), *File.Joints[J], JointWorst[J], JointAt[J]);
		Largest = FMath::Max(Largest, JointWorst[J]);
	}
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK bat knob max_mm %.4f at_ms %+.1f"), KnobWorst, KnobAt);
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK bat sweet_spot max_mm %.4f at_ms %+.1f"), SweetWorst, SweetAt);
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK hand_on_handle L max_mm %.4f at_ms %+.1f  R max_mm %.4f at_ms %+.1f (the drawn hand's knuckles against the file's grip point on the handle's axis)"),
		GripWorst[0], GripAt[0], GripWorst[1], GripAt[1]);
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK sole_under_ball (information, not checked) L max_mm %.2f R max_mm %.2f (the drawn sole under the ball of the foot against the file's ball contact points)"),
		SoleWorst[0], SoleWorst[1]);
	double TurnLargest = RollWorst;
	for (int32 S = 0; S < SegNumSeg; ++S)
	{
		UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK turn %-8s max_deg %.3f at_ms %+.1f (the drawn forward axis against the file's quaternion)"), SegNames[S], TurnWorst[S], TurnAt[S]);
		TurnLargest = FMath::Max(TurnLargest, TurnWorst[S]);
	}
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK turn bat_roll max_deg %.3f at_ms %+.1f"), RollWorst, RollAt);
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK hand start %s: %s"), bStartRight ? TEXT("PASS") : TEXT("FAIL"), *StartNote);
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK hand left_leg_on_his_left %s: wrong at %d of %d samples%s"), LeftWrong == 0 ? TEXT("PASS") : TEXT("FAIL"), LeftWrong, File.Num(),
		LeftWrong ? *FString::Printf(TEXT(", first at %+.1f ms"), LeftFirstWrong) : TEXT(""));
	const bool bPass = Largest <= CheckLimitMm && TurnLargest <= TurnLimitDeg && bStartRight && LeftWrong == 0 && Measured > 0 && Turns > 0;
	UE_LOG(LogBbSwing, Display, TEXT("BB_BODYCHECK result %s largest_mm %.4f limit_mm %.1f largest_turn_deg %.3f turn_limit_deg %.1f handedness %s"), bPass ? TEXT("PASS") : TEXT("FAIL"),
		Largest, CheckLimitMm, TurnLargest, TurnLimitDeg, bStartRight && LeftWrong == 0 ? TEXT("right") : TEXT("WRONG"));
	return bPass ? 0 : 1;
}

void ABbBodySwing::SetView(int32 InView)
{
	View = InView;
	if (!Camera)
	{
		return;
	}
	FVector Eye;
	float Fov = 40.f;
	switch (View)
	{
	case 0: Eye = Target + FVector(0.3, 7.5, 0.35); Fov = 36.f; break;
	case 1: Eye = Target + FVector(4.8, 0.5, 0.25); Fov = 52.f; break;
	case 2: Eye = Target + FVector(0.0, -0.4, 6.0); Fov = 50.f; break;
	default:
	{
		const double Yaw = FMath::DegreesToRadians(OrbitYaw), Pitch = FMath::DegreesToRadians(OrbitPitch);
		Eye = Target + OrbitDist * FVector(FMath::Cos(Pitch) * FMath::Sin(Yaw), FMath::Cos(Pitch) * FMath::Cos(Yaw), FMath::Sin(Pitch));
		Fov = 40.f;
	}
	}
	const FVector From = ToCm(Eye), To = ToCm(Target);
	Camera->SetActorLocationAndRotation(From, (To - From).Rotation());
	Camera->GetCameraComponent()->SetFieldOfView(Fov);
}

void ABbBodySwing::UpdateText()
{
	APlayerController* PC = GetWorld() ? GetWorld()->GetFirstPlayerController() : nullptr;
	ABbHUD* Hud = PC ? Cast<ABbHUD>(PC->GetHUD()) : nullptr;
	if (!Hud)
	{
		return;
	}
	Hud->BodyLines.Reset();
	Hud->BodyLines.Add(!File.Title.IsEmpty() ? File.Title : FString(TEXT("THE BODY'S PRACTISED SWING AS THE MODEL COMPUTED IT (PLAN D43): a viewing aid, not the game's swing")));
	if (!Error.IsEmpty())
	{
		Hud->BodyLines.Add(FString::Printf(TEXT("CANNOT SHOW %s: %s"), *File.Name, *Error));
		Hud->Footer = TEXT("Q quit");
		return;
	}
	Hud->BodyLines.Add(File.MissCm >= 0.0
		? FString::Printf(TEXT("%s   (plan %s, commit %s)   met at %.1f mph (%.2f m/s), missed by %.2f cm"), *File.Name, *File.Plan, *File.Commit, File.ContactMph, File.ContactMps, File.MissCm)
		: FString::Printf(TEXT("%s   (commit %s)   sweet spot %.1f mph at the contact sample"), *File.Name, *File.Commit, File.ContactMph));
	FBbSwingPose P;
	Sample(Clock, P);
	FString Last;
	for (const TPair<FString, double>& E : File.Events)
	{
		if (E.Value <= Clock + 1e-9)
		{
			Last = FString::Printf(TEXT("   last event: %s at %+.1f ms"), *E.Key, 1e3 * E.Value);
		}
	}
	Hud->BodyLines.Add(FString::Printf(TEXT("t %+7.1f ms from contact   sweet spot %5.1f mph   %s%s   view: %s%s"), 1e3 * Clock, P.Sweet * MphPerMps,
		SpeedNames[SpeedStep], bPaused ? TEXT("   PAUSED") : TEXT(""), ViewNames[View], *Last));
	const int32 N = File.Num();
	if (Clock <= File.T(0) + 1e-9)
	{
		Hud->BodyLines.Add(File.bRecorded ? TEXT("The recording's first sample (it starts here)") : TEXT("The model's first computed pose (it has nothing before this instant)"));
	}
	else if (Clock >= File.T(N - 1) - 1e-9)
	{
		Hud->BodyLines.Add(File.bRecorded ? TEXT("The recording's last sample (it ends here)") : TEXT("The model's last computed pose (its run ends here)"));
	}
	if (Clock > 1e-9)
	{
		Hud->BodyLines.Add(File.bBallAfter ? TEXT("After contact: the batted ball at the measured exit speed, launch and spray, under gravity (a picture, not computed)")
			: TEXT("After contact the ball is not shown: no batted ball is computed"));
	}
	Hud->Footer = TEXT("Space pause   Left/Right step one sample   R restart   - / = slower, faster   1 pitcher's side  2 open side  3 above  4 orbit (right-drag or J L I K, wheel or U O)   Q quit");
}

void ABbBodySwing::TickShots(float DeltaSeconds)
{
	// Like -BbFigureShots: the park settles for 8 s; then each instant is posed, given a second,
	// photographed, and given half a second more (0.6 and 0.3 s with -BbBodySwingShots=<ms>, which takes many).
	if (Wall < 8.0 || Shot < 0)
	{
		return;
	}
	if (Shot >= ShotViews * ShotTimes.Num())
	{
		Shot = -1;
		UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING pictures taken: %d"), ShotViews * ShotTimes.Num());
		FPlatformMisc::RequestExit(false);
		return;
	}
	const TPair<FString, double>& At = ShotTimes[Shot / ShotViews];
	if (ShotSince == 0.0)
	{
		Clock = FMath::Clamp(At.Value, File.T(0), File.T(File.Num() - 1));
		FBbSwingPose P;
		Sample(Clock, P);
		Pose(P);
		BallC->SetVisibility(Clock <= 1e-9 || File.bBallAfter);
		SetView(Shot % ShotViews);
		UpdateText();
		ShotSince = Wall;
	}
	if (!bShotAsked && Wall >= ShotSince + ShotBefore)
	{
		// two sides: bodyswing_<n>_<instant>_<side>; three: bodyswing_<nnn>_<m|p><ms>ms_<side>, so that they sort by time
		const FString Name = ShotViews == 2
			? FString::Printf(TEXT("bodyswing_%02d_%s_%s.png"), Shot / ShotViews + 1, *At.Key, ViewTags[Shot % ShotViews])
			: FString::Printf(TEXT("bodyswing_%03d_%s_%s.png"), Shot / ShotViews + 1, *At.Key, ViewTags[Shot % ShotViews]);
		FScreenshotRequest::RequestScreenshot(FPaths::Combine(FPaths::ProjectSavedDir(), TEXT("Shots"), Name), false, false);
		UE_LOG(LogBbSwing, Display, TEXT("BB_BODYSWING picture %s at %+.1f ms"), *Name, 1e3 * Clock);
		bShotAsked = true;
	}
	else if (bShotAsked && Wall >= ShotSince + ShotBefore + ShotAfter)
	{
		bShotAsked = false;
		ShotSince = 0.0;
		++Shot;
	}
}

void ABbBodySwing::Tick(float DeltaSeconds)
{
	Super::Tick(DeltaSeconds);
	Wall += DeltaSeconds;
	APlayerController* PC = GetWorld()->GetFirstPlayerController();
	if (PC && (PC->WasInputKeyJustPressed(EKeys::Escape) || PC->WasInputKeyJustPressed(EKeys::Q)))
	{
		FPlatformMisc::RequestExit(false);
		return;
	}
	if (!Error.IsEmpty() || File.Num() < 2)
	{
		return;
	}
	if (Shot >= 0)
	{
		TickShots(DeltaSeconds);
		return;
	}
	const int32 N = File.Num();
	if (PC)
	{
		if (PC->WasInputKeyJustPressed(EKeys::SpaceBar))
		{
			bPaused = !bPaused;
		}
		const bool bBack = PC->WasInputKeyJustPressed(EKeys::Left), bOn = PC->WasInputKeyJustPressed(EKeys::Right);
		if (bBack || bOn)
		{
			// One sample back or on from the one nearest the clock; paused there.
			int32 I = 0;
			for (int32 K = 1; K < N; ++K)
			{
				if (FMath::Abs(File.T(K) - Clock) < FMath::Abs(File.T(I) - Clock))
				{
					I = K;
				}
			}
			I = FMath::Clamp(I + (bOn ? 1 : -1), 0, N - 1);
			Clock = File.T(I);
			bPaused = true;
			HoldStart = HoldEnd = 0.0;
		}
		if (PC->WasInputKeyJustPressed(EKeys::R) || PC->WasInputKeyJustPressed(EKeys::Home))
		{
			Clock = File.T(0);
			HoldStart = HoldStartS;
			HoldEnd = 0.0;
		}
		if (PC->WasInputKeyJustPressed(EKeys::Hyphen) || PC->WasInputKeyJustPressed(EKeys::Subtract))
		{
			SpeedStep = FMath::Min(SpeedStep + 1, (int32)UE_ARRAY_COUNT(Speeds) - 1);
		}
		if (PC->WasInputKeyJustPressed(EKeys::Equals) || PC->WasInputKeyJustPressed(EKeys::Add))
		{
			SpeedStep = FMath::Max(SpeedStep - 1, 0);
		}
		static const FKey ViewKeys[] = { EKeys::One, EKeys::Two, EKeys::Three, EKeys::Four };
		for (int32 K = 0; K < 4; ++K)
		{
			if (PC->WasInputKeyJustPressed(ViewKeys[K]))
			{
				SetView(K);
			}
		}
		// The orbit: right mouse button held and dragged, or J L (round) I K (up, down); the wheel or U O nearer and farther.
		float Dx = 0.f, Dy = 0.f;
		if (PC->IsInputKeyDown(EKeys::RightMouseButton))
		{
			PC->GetInputMouseDelta(Dx, Dy);
		}
		const float Rate = 90.f * DeltaSeconds;
		Dx += (PC->IsInputKeyDown(EKeys::L) ? 1.f : 0.f) * Rate / 3.f - (PC->IsInputKeyDown(EKeys::J) ? 1.f : 0.f) * Rate / 3.f;
		Dy += (PC->IsInputKeyDown(EKeys::I) ? 1.f : 0.f) * Rate / 3.f - (PC->IsInputKeyDown(EKeys::K) ? 1.f : 0.f) * Rate / 3.f;
		float Zoom = 0.f;
		Zoom += PC->WasInputKeyJustPressed(EKeys::MouseScrollUp) ? -0.4f : 0.f;
		Zoom += PC->WasInputKeyJustPressed(EKeys::MouseScrollDown) ? 0.4f : 0.f;
		Zoom += (PC->IsInputKeyDown(EKeys::O) ? 3.f : 0.f) * DeltaSeconds - (PC->IsInputKeyDown(EKeys::U) ? 3.f : 0.f) * DeltaSeconds;
		if (Dx != 0.f || Dy != 0.f || Zoom != 0.f)
		{
			if (View != 3)
			{
				View = 3;
			}
			OrbitYaw += 3.f * Dx;
			OrbitPitch = FMath::Clamp(OrbitPitch + 3.f * Dy, -5.f, 85.f);
			OrbitDist = FMath::Clamp(OrbitDist + Zoom, 1.5f, 25.f);
		}
		if (View == 3)
		{
			SetView(3);
		}
	}
	if (!bPaused)
	{
		if (HoldStart > 0.0)
		{
			HoldStart -= DeltaSeconds;
		}
		else if (Clock < File.T(N - 1))
		{
			Clock = FMath::Min(File.T(N - 1), Clock + DeltaSeconds * Speeds[SpeedStep]);
			if (Clock >= File.T(N - 1))
			{
				HoldEnd = HoldEndS;
			}
		}
		else if ((HoldEnd -= DeltaSeconds) <= 0.0)
		{
			Clock = File.T(0);
			HoldStart = HoldStartS;
		}
	}
	FBbSwingPose P;
	Sample(Clock, P);
	Pose(P);
	BallC->SetVisibility(Clock <= 1e-9 || File.bBallAfter);
	UpdateText();
}
