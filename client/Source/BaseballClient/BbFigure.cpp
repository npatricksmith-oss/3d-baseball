#include "BbFigure.h"
#include "BbLook.h"
#include "BbPark.h"
#include "BbText.h"
#include "Components/SkeletalMeshComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/SkeletalMesh.h"
#include "Engine/StaticMesh.h"
#include "Engine/Texture2D.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "MeshDescription.h"
#include "ReferenceSkeleton.h"
#include "StaticMeshAttributes.h"

DEFINE_LOG_CATEGORY_STATIC(LogBbFigure, Log, All);

namespace
{
	const int32 ClubCloths = 8;
	/** Where a club's paint for a cloth is kept, or -1 if the cloth is not the club's. */
	int32 ClubSlot(EBbCloth C)
	{
		switch (C)
		{
		case EBbCloth::Jersey: return 0;
		case EBbCloth::Pants: return 1;
		case EBbCloth::Socks: return 2;
		case EBbCloth::Cap: return 3;
		case EBbCloth::Bill: return 4;
		case EBbCloth::Chest: return 5;
		case EBbCloth::Helmet: return 6;
		case EBbCloth::Gear: return 7;
		default: return -1;
		}
	}

	/** A limb: a round tube from Z0 to Z1 along the axis, R0 thick at the start and R1 at the
	 *  end, each end either rounded off (a joint) or cut flat. */
	TArray<FBbRing> Tube(double Z0, double Z1, double R0, double R1, bool bRound0, bool bRound1)
	{
		TArray<FBbRing> R;
		if (bRound0)
		{
			R.Add({ Z0 - 0.9 * R0, 0, 0, 0 });
			R.Add({ Z0 - 0.62 * R0, 0.70 * R0, 0.70 * R0, 0 });
			R.Add({ Z0 - 0.28 * R0, 0.94 * R0, 0.94 * R0, 0 });
		}
		else
		{
			R.Add({ Z0, 0, 0, 0 });
		}
		R.Add({ Z0, R0, R0, 0 });
		R.Add({ Z1, R1, R1, 0 });
		if (bRound1)
		{
			R.Add({ Z1 + 0.28 * R1, 0.94 * R1, 0.94 * R1, 0 });
			R.Add({ Z1 + 0.62 * R1, 0.70 * R1, 0.70 * R1, 0 });
			R.Add({ Z1 + 0.9 * R1, 0, 0, 0 });
		}
		else
		{
			R.Add({ Z1, 0, 0, 0 });
		}
		return R;
	}

	/** An egg centred Cz along the axis and Oy off it: all of it (Lo -1, Hi 1) or the slice
	 *  between two heights given as fractions of its half-height, cut flat. */
	TArray<FBbRing> Egg(double Cz, double Rx, double Ry, double Rz, double Oy, double Lo = -1.0, double Hi = 1.0)
	{
		TArray<FBbRing> R;
		const int32 N = 8;
		for (int32 J = 0; J <= N; ++J)
		{
			const double S = FMath::Lerp(Lo, Hi, (double)J / N);
			const double C = FMath::Sqrt(FMath::Max(0.0, 1.0 - S * S));
			if (J == 0 && C > 1e-6)
			{
				R.Add({ Cz + Rz * S, 0, 0, Oy });
			}
			R.Add({ Cz + Rz * S, Rx * C, Ry * C, Oy });
			if (J == N && C > 1e-6)
			{
				R.Add({ Cz + Rz * S, 0, 0, Oy });
			}
		}
		return R;
	}
}

UStaticMesh* UBbFigureShop::Lathe(const TArray<FBbRing>& Rings, int32 Sides)
{
	FMeshDescription Desc;
	FStaticMeshAttributes Attr(Desc);
	Attr.Register();
	const FPolygonGroupID Group = Desc.CreatePolygonGroup();
	Attr.GetPolygonGroupMaterialSlotNames()[Group] = TEXT("Paint");
	TVertexAttributesRef<FVector3f> Pos = Attr.GetVertexPositions();
	TVertexInstanceAttributesRef<FVector3f> Nor = Attr.GetVertexInstanceNormals();
	TVertexInstanceAttributesRef<FVector3f> Tan = Attr.GetVertexInstanceTangents();
	TVertexInstanceAttributesRef<float> Bin = Attr.GetVertexInstanceBinormalSigns();
	TVertexInstanceAttributesRef<FVector2f> Uv = Attr.GetVertexInstanceUVs();

	const int32 M = Rings.Num();
	double ZMin = Rings[0].Z, ZMax = Rings[0].Z;
	for (const FBbRing& R : Rings)
	{
		ZMin = FMath::Min(ZMin, R.Z);
		ZMax = FMath::Max(ZMax, R.Z);
	}
	auto At = [&](int32 I, int32 K)
	{
		const FBbRing& R = Rings[I];
		const double A = 2.0 * PI * K / Sides;
		return FVector(R.Rx * FMath::Cos(A), R.Oy + R.Ry * FMath::Sin(A), R.Z);
	};
	TArray<FVertexID> V;
	TArray<FVertexInstanceID> Inst;
	V.SetNum(M * Sides);
	Inst.SetNum(M * (Sides + 1));
	for (int32 I = 0; I < M; ++I)
	{
		for (int32 K = 0; K < Sides; ++K)
		{
			V[I * Sides + K] = Desc.CreateVertex();
			Pos[V[I * Sides + K]] = FVector3f(At(I, K));
		}
		for (int32 K = 0; K <= Sides; ++K)
		{
			const FVertexInstanceID Id = Desc.CreateVertexInstance(V[I * Sides + K % Sides]);
			Inst[I * (Sides + 1) + K] = Id;
			// Round the ring and along the piece. With rings rising along the axis this
			// cross product points outward (and down or up on a flat end).
			const FBbRing& R = Rings[I];
			const double A = 2.0 * PI * K / Sides;
			const FVector Round(-R.Rx * FMath::Sin(A), R.Ry * FMath::Cos(A), 0.0);
			const FVector Along = At(FMath::Min(I + 1, M - 1), K) - At(FMath::Max(I - 1, 0), K);
			FVector N = Round ^ Along;
			if (!N.Normalize())
			{
				N = FVector(0, 0, I * 2 < M ? -1.0 : 1.0);
			}
			FVector T = Round;
			if (!T.Normalize())
			{
				T = FVector(1, 0, 0);
			}
			Nor[Id] = FVector3f(N);
			Tan[Id] = FVector3f(T);
			Bin[Id] = 1.f;
			// The picture wraps once round; its top row is the far end of the axis.
			Uv.Set(Id, 0, FVector2f((float)K / Sides, ZMax > ZMin ? (float)((ZMax - R.Z) / (ZMax - ZMin)) : 0.f));
		}
	}
	for (int32 I = 0; I + 1 < M; ++I)
	{
		const bool bPoint0 = Rings[I].Rx <= 0.0 && Rings[I].Ry <= 0.0;
		const bool bPoint1 = Rings[I + 1].Rx <= 0.0 && Rings[I + 1].Ry <= 0.0;
		for (int32 K = 0; K < Sides; ++K)
		{
			const FVertexInstanceID A = Inst[I * (Sides + 1) + K], B = Inst[I * (Sides + 1) + K + 1];
			const FVertexInstanceID C = Inst[(I + 1) * (Sides + 1) + K], D = Inst[(I + 1) * (Sides + 1) + K + 1];
			// Wound as the park's faces are: seen from outside, the engine's front.
			if (!bPoint0)
			{
				Desc.CreateTriangle(Group, { A, C, B });
			}
			if (!bPoint1)
			{
				Desc.CreateTriangle(Group, { B, C, D });
			}
		}
	}
	UStaticMesh* Mesh = NewObject<UStaticMesh>(this, NAME_None, RF_Transient);
	Mesh->GetStaticMaterials().Add(FStaticMaterial(Park ? Park->PaintBase() : nullptr, TEXT("Paint")));
	UStaticMesh::FBuildMeshDescriptionsParams Params;
	Params.bFastBuild = true;
	Params.bMarkPackageDirty = false;
	Params.bCommitMeshDescription = false;
	Params.bBuildSimpleCollision = false;
	Mesh->BuildFromMeshDescriptions({ &Desc }, Params);
	Meshes.Add(Mesh);
	return Mesh;
}

bool UBbFigureShop::Init(USkeletalMesh* Skeleton, ABbPark* InPark)
{
	Park = InPark;
	const FReferenceSkeleton& Ref = Skeleton->GetRefSkeleton();
	// Every joint where it is when the mannequin stands as it was made: in the mesh's own
	// frame, cm, Z up, facing along Y.
	TArray<FTransform> Cs;
	Cs.SetNum(Ref.GetNum());
	for (int32 I = 0; I < Ref.GetNum(); ++I)
	{
		const int32 Parent = Ref.GetParentIndex(I);
		Cs[I] = Parent >= 0 ? Ref.GetRefBonePose()[I] * Cs[Parent] : Ref.GetRefBonePose()[I];
	}
	bool bOk = true;
	auto Joint = [&](const FString& Name)
	{
		const int32 I = Ref.FindBoneIndex(FName(*Name));
		if (I == INDEX_NONE)
		{
			UE_LOG(LogBbFigure, Warning, TEXT("The mannequin has no bone %s"), *Name);
			bOk = false;
			return FVector::ZeroVector;
		}
		return Cs[I].GetLocation();
	};
	auto Add = [&](const FString& Bone, const FVector& From, const FVector& Axis, const TArray<FBbRing>& Rings, EBbCloth Cloth, bool bBuild, int32 Sides = 14, EBbWear Wear = EBbWear::Always)
	{
		const int32 I = Ref.FindBoneIndex(FName(*Bone));
		if (I == INDEX_NONE)
		{
			return;
		}
		// The piece's own frame: Z along it, X across the body as nearly as can be, Y the
		// third (the front, for a piece that stands upright).
		const FVector Z = Axis.GetSafeNormal();
		FVector X = FVector(1, 0, 0) - Z * Z.X;
		if (X.SizeSquared() < 0.05)
		{
			X = FVector(0, 1, 0) - Z * Z.Y;
		}
		X.Normalize();
		const FVector Y = Z ^ X;
		FBbPiece P;
		P.Bone = FName(*Bone);
		P.OnBone = FTransform(X, Y, Z, From).GetRelativeTransform(Cs[I]);
		P.Cloth = Cloth;
		P.bBuild = bBuild;
		P.Wear = Wear;
		P.Mesh = Meshes.Num();
		Lathe(Rings, Sides);
		Parts.Add(P);
	};

	const FVector Up(0, 0, 1);
	const FVector Pelvis = Joint(TEXT("pelvis")), Spine1 = Joint(TEXT("spine_01")), Spine3 = Joint(TEXT("spine_03")), Spine4 = Joint(TEXT("spine_04"));
	const FVector Neck = Joint(TEXT("neck_01")), Head = Joint(TEXT("head"));
	const FVector ThighL = Joint(TEXT("thigh_l")), ShoulderL = Joint(TEXT("upperarm_l"));
	const FVector FootL = Joint(TEXT("foot_l")), BallL = Joint(TEXT("ball_l"));
	if (!bOk)
	{
		return false;
	}
	// Which way he faces along Y, from heel to toe.
	const double Fwd = BallL.Y >= FootL.Y ? 1.0 : -1.0;
	const double HeadTop = 20.6;
	Standing = Head.Z + HeadTop;
	UE_LOG(LogBbFigure, Display, TEXT("BB_FIGURE joints (cm): pelvis %s spine_01 %s spine_03 %s neck %s head %s thigh_l %s shoulder_l %s foot_l %s ball_l %s; standing %.1f"),
		*Pelvis.ToCompactString(), *Spine1.ToCompactString(), *Spine3.ToCompactString(), *Neck.ToCompactString(), *Head.ToCompactString(),
		*ThighL.ToCompactString(), *ShoulderL.ToCompactString(), *FootL.ToCompactString(), *BallL.ToCompactString(), Standing);

	// The trunk, in three stiff pieces that overlap where the spine bends: the shirt's
	// middle is tucked inside the chest above it and the trousers below.
	{
		const FVector Base(0, Pelvis.Y, ThighL.Z - 9.0);
		const double Top = FMath::Max(12.0, Spine1.Z - Base.Z + 2.0);
		const double Hip = FMath::Abs(ThighL.X) + 8.8;
		Add(TEXT("pelvis"), Base, Up, { { 0, 0, 0, 0 }, { 1.5, Hip * 0.55, 7.5, 0 }, { 5, Hip * 0.92, 10.2, 0 }, { 9, Hip, 10.9, 0 }, { Top, 14.9, 10.1, 0 }, { Top, 0, 0, 0 } }, EBbCloth::Pants, true, 16);
		Add(TEXT("pelvis"), Base, Up, { { Top - 3.2, 0, 0, 0 }, { Top - 3.2, 15.6, 10.8, 0 }, { Top + 0.8, 15.3, 10.5, 0 }, { Top + 0.8, 0, 0, 0 } }, EBbCloth::Belt, true, 16);
	}
	{
		const double L = (Spine4 - Spine1).Size();
		Add(TEXT("spine_02"), Spine1, Spine4 - Spine1, { { -3, 0, 0, 0 }, { -3, 14.4, 9.7, 0 }, { L * 0.5, 14.8, 10.1, 0 }, { L + 2, 15.0, 10.3, 0 }, { L + 2, 0, 0, 0 } }, EBbCloth::Jersey, true, 16);
	}
	{
		const double L = (Neck - Spine3).Size();
		const double Sw = FMath::Abs(ShoulderL.X);
		Add(TEXT("spine_04"), Spine3, Neck - Spine3,
			{ { -2, 0, 0, 0 }, { -2, 16.0, 11.1, 0 }, { L * 0.35, Sw * 0.9 + 1.0, 11.7, 0 }, { L * 0.72, Sw + 1.5, 11.3, 0 }, { L * 0.9, Sw - 1.0, 9.6, 0 }, { L, 8.6, 7.6, 0 }, { L + 1.5, 6.2, 6.2, 0 }, { L + 1.5, 0, 0, 0 } },
			EBbCloth::Chest, true, 24);
	}
	Add(TEXT("neck_01"), Neck, Head - Neck, Tube(0.0, (Head - Neck).Size(), 5.4, 5.0, true, true), EBbCloth::Skin, false, 12);
	// The head, hair showing below the cap at the back and sides, and the cap with its bill.
	Add(TEXT("head"), Head, Up, Egg(9.0, 8.0, 9.6, 11.6, Fwd * 1.5), EBbCloth::Skin, false, 16);
	Add(TEXT("head"), Head, Up, Egg(9.0, 8.3, 9.7, 11.4, Fwd * 0.2, -0.55, 0.3), EBbCloth::Hair, false, 16);
	Add(TEXT("head"), Head, Up, Egg(9.0, 8.75, 10.3, 12.3, Fwd * 1.3, 0.22, 1.0), EBbCloth::Cap, false, 16, EBbWear::Cap);
	{
		const double Out = Fwd * 13.4;
		Add(TEXT("head"), Head + Up * 11.8, Up, { { 0, 0, 0, Out }, { 0, 8.0, 6.1, Out }, { 1.0, 7.8, 5.9, Out }, { 1.0, 0, 0, Out } }, EBbCloth::Bill, false, 16, EBbWear::Cap);
		// The helmet (step 2): a shell over the crown, flaps over the ears and the back of the
		// head that stop short of the face, and a short bill.
		Add(TEXT("head"), Head, Up, Egg(9.0, 9.3, 10.9, 12.9, Fwd * 1.3, 0.10, 1.0), EBbCloth::Helmet, false, 16, EBbWear::Helmet);
		Add(TEXT("head"), Head, Up, Egg(9.0, 9.3, 8.6, 12.9, -Fwd * 1.0, -0.50, 0.12), EBbCloth::Helmet, false, 16, EBbWear::Helmet);
		const double Brim = Fwd * 12.6;
		Add(TEXT("head"), Head + Up * 11.2, Up, { { 0, 0, 0, Brim }, { 0, 8.2, 5.4, Brim }, { 1.2, 8.0, 5.2, Brim }, { 1.2, 0, 0, Brim } }, EBbCloth::Helmet, false, 16, EBbWear::HelmetBill);
		// The catcher's mask: a dark cage over the face, from the brow to under the chin.
		Add(TEXT("head"), Head, Up, Egg(7.2, 7.2, 6.4, 10.2, Fwd * 6.6, -0.95, 0.62), EBbCloth::Mask, false, 14, EBbWear::Catcher);
	}
	{
		// His chest protector: a plate hung in front of the chest and down over the belly.
		const double L = (Neck - Spine3).Size(), Oy = Fwd * 10.4;
		Add(TEXT("spine_04"), Spine3, Neck - Spine3,
			{ { -17, 0, 0, Oy }, { -17, 9.0, 2.4, Oy }, { -9, 13.0, 3.0, Oy }, { L * 0.5, 15.0, 3.3, Oy }, { L * 0.8, 13.0, 3.0, Oy }, { L * 0.96, 7.0, 2.4, Oy }, { L * 0.96, 0, 0, Oy } },
			EBbCloth::Gear, true, 16, EBbWear::Catcher);
	}
	for (const TCHAR* S : { TEXT("_l"), TEXT("_r") })
	{
		const FVector Shoulder = Joint(FString(TEXT("upperarm")) + S), Elbow = Joint(FString(TEXT("lowerarm")) + S), Wrist = Joint(FString(TEXT("hand")) + S);
		const FVector Hip = Joint(FString(TEXT("thigh")) + S), Knee = Joint(FString(TEXT("calf")) + S), Ankle = Joint(FString(TEXT("foot")) + S), Ball = Joint(FString(TEXT("ball")) + S);
		if (!bOk)
		{
			return false;
		}
		// The arm: a short sleeve over the shoulder, the bare arm, the hand.
		const double Upper = (Elbow - Shoulder).Size(), Fore = (Wrist - Elbow).Size();
		Add(FString(TEXT("upperarm")) + S, Shoulder, Elbow - Shoulder, Tube(0.0, Upper * 0.55, 6.5, 5.9, true, false), EBbCloth::Jersey, true, 12);
		Add(FString(TEXT("upperarm")) + S, Shoulder, Elbow - Shoulder, Tube(Upper * 0.3, Upper, 5.0, 4.4, false, true), EBbCloth::Skin, true, 12);
		Add(FString(TEXT("lowerarm")) + S, Elbow, Wrist - Elbow, Tube(0.0, Fore, 4.5, 3.1, true, true), EBbCloth::Skin, true, 12);
		const int32 Knuckle = Ref.FindBoneIndex(FName(*(FString(TEXT("middle_01")) + S)));
		const FVector Reach = Knuckle != INDEX_NONE ? Cs[Knuckle].GetLocation() - Wrist : Wrist - Elbow;
		Add(FString(TEXT("hand")) + S, Wrist, Reach, Egg(5.0, 3.4, 3.4, 5.8, 0.0), EBbCloth::Skin, false, 10);
		// A glove over the hand (step 2), drawn a little large so that it reads from the
		// stands, as the ball and the bat are. Worn on one hand only (ABbFigure::ApplyKit).
		Add(FString(TEXT("hand")) + S, Wrist, Reach, Egg(9.5, BbLook::GloveThickCm, BbLook::GloveWideCm, BbLook::GloveLongCm, 0.0, -0.8, 1.0), EBbCloth::Leather, false, 14,
			S[1] == TEXT('l') ? EBbWear::GloveL : EBbWear::GloveR);
		// The leg: trousers to the knee, the sock, the shoe.
		Add(FString(TEXT("thigh")) + S, Hip, Knee - Hip, Tube(0.0, (Knee - Hip).Size(), 9.8, 6.9, true, true), EBbCloth::Pants, true, 14);
		Add(FString(TEXT("calf")) + S, Knee, Ankle - Knee, Tube(0.0, (Ankle - Knee).Size(), 6.3, 4.6, true, true), EBbCloth::Socks, true, 12);
		{
			// The catcher's shin guard: over the knee and down the front of the shin, dark like
			// the mask so that it shows against the sock. (This
			// piece's own Y points to the back, its axis running down the leg.)
			const double L = (Ankle - Knee).Size(), Oy = -Fwd * 3.2;
			Add(FString(TEXT("calf")) + S, Knee, Ankle - Knee,
				{ { -7, 0, 0, Oy }, { -7, 5.0, 3.6, Oy }, { -2, 6.9, 5.0, Oy }, { 5, 6.3, 4.3, Oy }, { L - 2, 5.0, 3.8, Oy }, { L - 2, 0, 0, Oy } },
				EBbCloth::Mask, true, 12, EBbWear::Catcher);
		}
		{
			// The shoe lies along the foot from ankle to ball and on past it to the toe. Its
			// frame's Y points down, so an offset of Oy lowers a ring's centre: each ring is
			// centred so that its underside is the sole, on the ground.
			const FVector Along = Ball - Ankle;
			const double L = Along.Size(), Drop = (Ankle.Z - Ball.Z) / FMath::Max(L, 1.0);
			auto Ring = [&](double Z, double Rx, double TopCm)
			{
				// TopCm: how high the shoe stands there. The axis there is this high:
				const double AxisZ = Ankle.Z - Drop * Z;
				return FBbRing{ Z, Rx, TopCm * 0.5, AxisZ - TopCm * 0.5 };
			};
			TArray<FBbRing> R;
			FBbRing Heel = Ring(-5.5, 0, 5.0);
			Heel.Ry = 0;
			R.Add(Heel);
			R.Add(Ring(-5.0, 3.2, 6.5));
			R.Add(Ring(0.0, 4.4, Ankle.Z + 1.5));
			R.Add(Ring(L, 5.0, Ball.Z + 3.5));
			R.Add(Ring(L + 4.0, 4.3, 4.4));
			FBbRing Toe = Ring(L + 6.0, 0, 3.0);
			Toe.Ry = 0;
			R.Add(Toe);
			Add(FString(TEXT("foot")) + S, Ankle, Along, R, EBbCloth::Shoe, false, 12);
		}
	}

	// The paints: one of each, shared by every figure that wears it.
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const BbLook::FBbClub& C = BbLook::Clubs[Side];
		const FBbPaint Helmet{ C.Cap.Color, BbLook::HelmetRough }, Gear{ C.Trim, BbLook::GearRough };
		for (const FBbPaint& P : { C.Jersey, C.Pants, C.Socks, C.Cap, C.Bill, C.Jersey, Helmet, Gear })
		{
			ClubPaints.Add(Park->PaintOf(P));
		}
	}
	for (const FBbPaint& P : BbLook::Skin)
	{
		SkinPaints.Add(Park->PaintOf(P));
	}
	for (int32 I = 0; I < BbLook::HairTones; ++I)
	{
		HairPaints.Add(Park->PaintOf(BbLook::Heads[I]));
	}
	ShoePaint = Park->PaintOf(BbLook::Shoe);
	BeltPaint = Park->PaintOf(BbLook::Belt);
	MaskPaint = Park->PaintOf(BbLook::Mask);
	LeatherPaint = Park->PaintOf(BbLook::Leather);
	WoodPaint = Park->PaintOf(BbLook::Wood);
	// The bat: a knob, a thin handle swelling to the barrel, a rounded end.
	{
		const double L = BatCm;
		Bat = Lathe({ { 0, 0, 0, 0 }, { 0, 2.3, 2.3, 0 }, { 1.0, 2.6, 2.6, 0 }, { 2.0, 2.3, 2.3, 0 }, { 3.2, 1.3, 1.3, 0 }, { L * 0.30, 1.3, 1.3, 0 }, { L * 0.50, 1.8, 1.8, 0 },
			{ L * 0.68, 2.9, 2.9, 0 }, { L * 0.80, 3.3, 3.3, 0 }, { L - 1.6, 3.3, 3.3, 0 }, { L - 0.5, 2.8, 2.8, 0 }, { L, 1.6, 1.6, 0 }, { L, 0, 0, 0 } }, 14);
	}
	UE_LOG(LogBbFigure, Display, TEXT("BB_FIGURE %d pieces a figure"), Parts.Num());
	return bOk;
}

UMaterialInstanceDynamic* UBbFigureShop::PaintFor(EBbCloth Cloth, int32 Side, int32 Skin, int32 Hair) const
{
	switch (Cloth)
	{
	case EBbCloth::Skin: return SkinPaints[Skin % SkinPaints.Num()];
	case EBbCloth::Hair: return HairPaints[Hair % HairPaints.Num()];
	case EBbCloth::Shoe: return ShoePaint;
	case EBbCloth::Belt: return BeltPaint;
	case EBbCloth::Mask: return MaskPaint;
	case EBbCloth::Leather: return LeatherPaint;
	default: return ClubPaints[(Side & 1) * ClubCloths + ClubSlot(Cloth)];
	}
}

UMaterialInstanceDynamic* UBbFigureShop::NewChest(UObject* Outer) const
{
	UMaterialInterface* Base = Park ? Park->PictureBase() : nullptr;
	if (!Base)
	{
		return nullptr;
	}
	UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(Base, Outer);
	M->SetScalarParameterValue(TEXT("Rough"), BbLook::Clubs[0].Jersey.Rough);
	return M;
}

UTexture2D* UBbFigureShop::Jersey(int32 Side, int32 Number, bool bMirror)
{
	const int32 Key = (Side & 1) | (bMirror ? 2 : 0) | (Number << 2);
	if (TObjectPtr<UTexture2D>* Found = Jerseys.Find(Key))
	{
		return *Found;
	}
	// The picture wraps once round the chest from under one arm: the front is centred a
	// quarter of the way across, the back three quarters. Top row: the collar.
	const BbLook::FBbClub& C = BbLook::Clubs[Side & 1];
	const int32 W = 1024, H = 256;
	FBbCanvas Cv(W, H, C.Jersey.Color);
	const int32 Front = W / 4, Back = 3 * W / 4;
	// The collar, and the placket down the front.
	Cv.Box(0, 0, W, 14, C.Trim);
	Cv.Box(Front - 2, 14, Front + 2, H, C.Trim);
	// Across the chest: the club's lettering, as large as fits.
	const FString Word = C.Chest;
	// (A dot of the lettering is wider than it is high in the picture: the picture is
	// about 10 dots to the centimetre round the chest and 6 up it.)
	const int32 Sx = FMath::Clamp(315 / FMath::Max(1, Word.Len() * 6 - 1), 3, 7), Sy = 6;
	const int32 WordY = 92;
	Cv.Box(Front - FBbCanvas::TextW(Word, Sx) / 2 - 6, WordY - 4 * Sy, Front + FBbCanvas::TextW(Word, Sx) / 2 + 6, WordY + 4 * Sy, C.Jersey.Color);
	Cv.Text(Word, Front, WordY, Sx, Sy, C.Trim);
	// The number, on the back.
	if (Number > 0)
	{
		Cv.Text(FString::FromInt(Number), Back, 122, 19, 13, C.Trim);
	}
	// Seen from outside, the picture runs right to left round the piece, so each half is
	// turned over to read; a figure shown in a mirror wants it as painted.
	if (!bMirror)
	{
		for (int32 Y = 0; Y < H; ++Y)
		{
			for (int32 Half = 0; Half < 2; ++Half)
			{
				FColor* Row = &Cv.Px[Y * W + Half * (W / 2)];
				for (int32 X = 0; X < W / 4; ++X)
				{
					Swap(Row[X], Row[W / 2 - 1 - X]);
				}
			}
		}
	}
	UTexture2D* T = Park->MakeTexture(MoveTemp(Cv.Px), W, H);
	Jerseys.Add(Key, T);
	return T;
}

void ABbFigure::Build(UBbFigureShop* InShop)
{
	Shop = InShop;
	USkeletalMeshComponent* Driver = GetSkeletalMeshComponent();
	for (const FBbPiece& P : Shop->Pieces())
	{
		UStaticMeshComponent* C = NewObject<UStaticMeshComponent>(this);
		C->SetMobility(EComponentMobility::Movable);
		C->SetStaticMesh(Shop->MeshOf(P));
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		C->SetupAttachment(Driver, P.Bone);
		C->RegisterComponent();
		C->SetRelativeTransform(P.OnBone);
		Parts.Add(C);
	}
	ChestPaint = Shop->NewChest(this);
	// The mannequin goes on moving its bones, unseen.
	Driver->SetVisibility(false, false);
	Dressed = 0;
	Dress(nullptr, 1);
}

void ABbFigure::Dress(const BbPlayer* Player, int32 InSide)
{
	const uint32 Key = Player ? (Player->player_id | 0x40000000u) : (0x80000000u | (uint32)(InSide & 1));
	if (Key == Dressed || !Shop)
	{
		return;
	}
	Dressed = Key;
	Side = Player ? Player->side & 1 : InSide & 1;
	Number = Player ? Player->number : 0;
	bThrowsLeft = Player && Player->throws == 1;
	// The body: from the player's id, the same every time (owner's Q18). Two draws averaged
	// bunch heights and builds toward the middle.
	FRandomStream Rng((int32)((Player ? Player->player_id : 0u) * 7919u + 13u));
	const float HeightU = (Rng.FRand() + Rng.FRand()) * 0.5f, BuildU = (Rng.FRand() + Rng.FRand()) * 0.5f;
	const int32 Skin = Rng.RandHelper(UE_ARRAY_COUNT(BbLook::Skin)), Hair = Rng.RandHelper(BbLook::HairTones);
	Height = Player ? FMath::Lerp(BbLook::HeightMinCm, BbLook::HeightMaxCm, HeightU) / (float)Shop->StandingCm() : 187.f / (float)Shop->StandingCm();
	const float Build = Player ? FMath::Lerp(BbLook::BuildMin, BbLook::BuildMax, BuildU) : 1.f;
	const TArray<FBbPiece>& Pieces = Shop->Pieces();
	for (int32 I = 0; I < Parts.Num(); ++I)
	{
		const FBbPiece& P = Pieces[I];
		if (P.Cloth == EBbCloth::Chest && ChestPaint)
		{
			Parts[I]->SetMaterial(0, ChestPaint);
		}
		else
		{
			Parts[I]->SetMaterial(0, Shop->PaintFor(P.Cloth, Side, Skin, Hair));
		}
		Parts[I]->SetRelativeScale3D(P.bBuild ? FVector(Build, Build, 1.f) : FVector::OneVector);
	}
	Apply();
	ApplyKit();
}

void ABbFigure::SetRole(EBbRole InRole)
{
	if (InRole != Role)
	{
		Role = InRole;
		ApplyKit();
	}
}

void ABbFigure::ApplyKit()
{
	if (!Shop)
	{
		return;
	}
	const bool bGlove = Role == EBbRole::Fielder || Role == EBbRole::Catcher;
	const bool bHelmet = Role == EBbRole::Batter || Role == EBbRole::Catcher;
	// The glove goes on the hand he does not throw with. In a mirror the bones' left is the
	// viewer's right, so it changes bones to stay on the same hand.
	const bool bOnLeftBone = bThrowsLeft == bMirrored;
	const TArray<FBbPiece>& Pieces = Shop->Pieces();
	for (int32 I = 0; I < Parts.Num(); ++I)
	{
		bool bWorn = true;
		switch (Pieces[I].Wear)
		{
		case EBbWear::Always: continue;
		case EBbWear::Cap: bWorn = !bHelmet; break;
		case EBbWear::Helmet: bWorn = bHelmet; break;
		case EBbWear::HelmetBill: bWorn = Role == EBbRole::Batter; break;
		case EBbWear::Catcher: bWorn = Role == EBbRole::Catcher; break;
		case EBbWear::GloveL: bWorn = bGlove && bOnLeftBone; break;
		case EBbWear::GloveR: bWorn = bGlove && !bOnLeftBone; break;
		}
		Parts[I]->SetVisibility(bWorn);
		if (Pieces[I].Cloth == EBbCloth::Leather)
		{
			Parts[I]->SetRelativeScale3D(FVector(Role == EBbRole::Catcher ? BbLook::MittScale : 1.f));
		}
	}
}

void ABbFigure::SetMirror(bool bMirror)
{
	if (bMirror != bMirrored)
	{
		bMirrored = bMirror;
		Apply();
		ApplyKit();
		// A mirror turns every face over. The engine keeps a piece's drawing orders from
		// when it was made, facing included, so they are made again; otherwise the figure
		// is drawn inside out (seen in figure_6_turn_mirrored, 2026-11-15).
		for (UStaticMeshComponent* C : Parts)
		{
			C->MarkRenderStateDirty();
		}
	}
}

void ABbFigure::Apply()
{
	// The mannequin faces +Y in its own frame, so its left and right are its X.
	SetActorScale3D(FVector(bMirrored ? -Height : Height, Height, Height));
	if (ChestPaint && Shop)
	{
		ChestPaint->SetTextureParameterValue(TEXT("Tex"), Shop->Jersey(Side, Number, bMirrored));
	}
}
