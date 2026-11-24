#include "BbPose.h"
#include "Animation/AnimNodeBase.h"
#include "BonePose.h"
#include "Engine/SkeletalMesh.h"
#include "ReferenceSkeleton.h"

DEFINE_LOG_CATEGORY_STATIC(LogBbPose, Log, All);

namespace
{
	/** The turn that takes the mannequin as it was made (facing +Y, standing along +Z) to
	 *  facing Fwd and standing as nearly along Up as that allows. */
	FQuat Facing(const FVector& Fwd, const FVector& Up)
	{
		const FVector F = Fwd.GetSafeNormal(1e-6, FVector(0, 1, 0));
		FVector U = Up - F * (Up | F);
		if (!U.Normalize())
		{
			U = FMath::Abs(F.Z) < 0.9 ? FVector(0, 0, 1) : FVector(0, -1, 0);
			U = (U - F * (U | F)).GetSafeNormal();
		}
		return FRotationMatrix::MakeFromYZ(F, U).ToQuat();
	}

	/** Two bones of lengths A and B from Root to Target, the middle joint toward Pole: where
	 *  the middle joint goes, and where the end can get to (Target, or as near as it reaches). */
	void TwoBone(const FVector& Root, const FVector& Target, double A, double B, const FVector& Pole, FVector& OutMid, FVector& OutEnd)
	{
		FVector D = Target - Root;
		double L = D.Size();
		if (L < 1e-3)
		{
			D = FVector(0, 0, -1);
			L = 1e-3;
		}
		D /= L;
		const double Lc = FMath::Clamp(L, FMath::Abs(A - B) + 0.5, A + B - 0.05);
		const double X = (A * A - B * B + Lc * Lc) / (2.0 * Lc);
		const double H = FMath::Sqrt(FMath::Max(0.0, A * A - X * X));
		FVector P = Pole - D * (Pole | D);
		if (!P.Normalize())
		{
			P = FVector(0, 1, 0) - D * D.Y;
			if (!P.Normalize())
			{
				P = FVector(1, 0, 0);
			}
		}
		OutMid = Root + D * X + P * H;
		OutEnd = Root + D * Lc;
	}

	double Smooth(double A, double B, double X)
	{
		const double U = FMath::Clamp((X - A) / (B - A), 0.0, 1.0);
		return U * U * (3.0 - 2.0 * U);
	}
}

bool FBbSkeleton::Init(const USkeletalMesh* Mesh)
{
	bOk = false;
	if (!Mesh)
	{
		return false;
	}
	const FReferenceSkeleton& Ref = Mesh->GetRefSkeleton();
	const int32 N = Ref.GetNum();
	Bones.SetNum(N);
	for (int32 I = 0; I < N; ++I)
	{
		FBone& B = Bones[I];
		B.Parent = Ref.GetParentIndex(I);
		B.RefLocal = Ref.GetRefBonePose()[I];
		B.Ref = B.Parent >= 0 ? B.RefLocal * Bones[B.Parent].Ref : B.RefLocal;
	}
	bool bAll = true;
	auto Find = [&](const FString& Name)
	{
		const int32 I = Ref.FindBoneIndex(FName(*Name));
		if (I == INDEX_NONE)
		{
			UE_LOG(LogBbPose, Warning, TEXT("The mannequin has no bone %s: no coded motions"), *Name);
			bAll = false;
		}
		return I;
	};
	auto Mark = [&](const FString& Name, ERole Role, int32 Side, float Share, const FString& Next)
	{
		const int32 I = Find(Name);
		if (I == INDEX_NONE)
		{
			return I;
		}
		Bones[I].Role = Role;
		Bones[I].Side = Side;
		Bones[I].Share = Share;
		if (!Next.IsEmpty())
		{
			const int32 J = Find(Next);
			if (J != INDEX_NONE)
			{
				Bones[I].Along = (Bones[J].Ref.GetLocation() - Bones[I].Ref.GetLocation()).GetSafeNormal();
			}
		}
		return I;
	};
	PelvisIndex = Mark(TEXT("pelvis"), Pelvis, 0, 0.f, FString());
	for (int32 K = 1; K <= 5; ++K)
	{
		Mark(FString::Printf(TEXT("spine_%02d"), K), Spine, 0, K / 5.f, FString());
	}
	Mark(TEXT("neck_01"), Neck, 0, 1.f / 3.f, FString());
	Mark(TEXT("neck_02"), Neck, 0, 2.f / 3.f, FString());
	Mark(TEXT("head"), Head, 0, 1.f, FString());
	for (int32 Side = 0; Side < 2; ++Side)
	{
		const TCHAR* S = Side == 0 ? TEXT("_l") : TEXT("_r");
		const int32 Upper = Mark(FString(TEXT("upperarm")) + S, UpperArm0, Side, 0.f, FString(TEXT("lowerarm")) + S);
		const int32 Lower = Mark(FString(TEXT("lowerarm")) + S, LowerArm0, Side, 0.f, FString(TEXT("hand")) + S);
		const int32 Hand = Mark(FString(TEXT("hand")) + S, Hand0, Side, 0.f, FString(TEXT("middle_01")) + S);
		const int32 Hip = Mark(FString(TEXT("thigh")) + S, Thigh0, Side, 0.f, FString(TEXT("calf")) + S);
		const int32 Shin = Mark(FString(TEXT("calf")) + S, Calf0, Side, 0.f, FString(TEXT("foot")) + S);
		const int32 Foot = Mark(FString(TEXT("foot")) + S, Foot0, Side, 0.f, FString());
		if (!bAll)
		{
			return false;
		}
		if (Side == 0)
		{
			UpperArm = (Bones[Lower].Ref.GetLocation() - Bones[Upper].Ref.GetLocation()).Size();
			ForeArm = (Bones[Hand].Ref.GetLocation() - Bones[Lower].Ref.GetLocation()).Size();
			Thigh = (Bones[Shin].Ref.GetLocation() - Bones[Hip].Ref.GetLocation()).Size();
			Calf = (Bones[Foot].Ref.GetLocation() - Bones[Shin].Ref.GetLocation()).Size();
			AnkleZ = Bones[Foot].Ref.GetLocation().Z;
		}
	}
	UE_LOG(LogBbPose, Display, TEXT("BB_POSE skeleton: %d bones; upper arm %.1f forearm %.1f thigh %.1f calf %.1f ankle %.1f cm"), N, UpperArm, ForeArm, Thigh, Calf, AnkleZ);
	bOk = bAll;
	return bOk;
}

TArray<bool> FBbSkeleton::UpperBones() const
{
	// Everything from the first bone of the spine up and out.
	TArray<bool> Out;
	Out.SetNum(Bones.Num());
	for (int32 I = 0; I < Bones.Num(); ++I)
	{
		Out[I] = Bones[I].Role == Spine || (Bones[I].Parent >= 0 && Out[Bones[I].Parent]);
	}
	return Out;
}

void FBbSkeleton::PelvisFacing(const FTransform& Bone, FVector& OutFwd, FVector& OutUp) const
{
	const FQuat D = Bones.IsValidIndex(PelvisIndex) ? Bone.GetRotation() * Bones[PelvisIndex].Ref.GetRotation().Inverse() : FQuat::Identity;
	OutFwd = D.RotateVector(FVector(0, 1, 0));
	OutUp = D.RotateVector(FVector(0, 0, 1));
}

void FBbSkeleton::Solve(const FBbStick& S, const FBbReach& Reach, TArray<FTransform>& OutLocal, FVector OutHold[2], FVector OutWrist[2]) const
{
	const int32 N = Bones.Num();
	OutLocal.SetNum(N);
	TArray<FTransform, TInlineAllocator<128>> Cs;
	Cs.SetNum(N);
	const FQuat Dp = Facing(S.PelvisFwd, S.PelvisUp), Dc = Facing(S.ChestFwd, S.ChestUp), Dh = Facing(S.HeadFwd, S.HeadUp);
	FVector PelvisAt = S.Pelvis;
	// Twice over: the first pass finds how far short an arm falls of a point it must reach,
	// and the second moves the whole body that far (the legs take it up).
	for (int32 Pass = 0; Pass < 2; ++Pass)
	{
		FVector Shift = FVector::ZeroVector;
		// The limbs' joints and turns, found at the first bone of each and used by the rest.
		FVector Mid[2][2], End[2][2];
		FQuat TurnA[2][2], TurnB[2][2];
		for (int32 I = 0; I < N; ++I)
		{
			const FBone& B = Bones[I];
			const FTransform Fk = B.Parent >= 0 ? B.RefLocal * Cs[B.Parent] : B.RefLocal;
			const FQuat Ref = B.Ref.GetRotation();
			switch (B.Role)
			{
			case Pelvis:
				Cs[I] = FTransform(Dp * Ref, PelvisAt);
				break;
			case Spine:
				Cs[I] = FTransform(FQuat::Slerp(Dp, Dc, B.Share) * Ref, Fk.GetLocation());
				break;
			case Neck:
			case Head:
				Cs[I] = FTransform(FQuat::Slerp(Dc, Dh, B.Share) * Ref, Fk.GetLocation());
				break;
			case UpperArm0:
			{
				const int32 Sd = B.Side;
				const FVector Shoulder = Fk.GetLocation();
				FVector Want = Shoulder + Dc.RotateVector(S.Wrist[Sd]);
				const float W = Reach.Weight[Sd];
				if (W > 0.f)
				{
					FVector To = Reach.At[Sd] - Shoulder;
					const FVector At = Reach.At[Sd] - To.GetSafeNormal() * Reach.Past[Sd];
					Want = FMath::Lerp(Want, At, (double)W);
					To = At - Shoulder;
					const double Short = To.Size() - 0.97 * (UpperArm + ForeArm);
					if (Pass == 0 && Short > 0.0)
					{
						Shift += To.GetSafeNormal() * Short * W;
					}
				}
				TwoBone(Shoulder, Want, UpperArm, ForeArm, Dc.RotateVector(S.Elbow[Sd]), Mid[0][Sd], End[0][Sd]);
				// What he holds sits on along the forearm, which is not the line from the
				// shoulder when the elbow is bent: the wrist is moved until it is on the point.
				for (int32 K = 0; K < 3 && W > 0.f && Reach.Past[Sd] > 0.f; ++K)
				{
					const FVector At = Reach.At[Sd] - (End[0][Sd] - Mid[0][Sd]).GetSafeNormal() * Reach.Past[Sd];
					Want = FMath::Lerp(Shoulder + Dc.RotateVector(S.Wrist[Sd]), At, (double)W);
					TwoBone(Shoulder, Want, UpperArm, ForeArm, Dc.RotateVector(S.Elbow[Sd]), Mid[0][Sd], End[0][Sd]);
				}
				TurnA[0][Sd] = FQuat::FindBetweenNormals(Dc.RotateVector(B.Along), (Mid[0][Sd] - Shoulder).GetSafeNormal()) * Dc;
				Cs[I] = FTransform(TurnA[0][Sd] * Ref, Shoulder);
				break;
			}
			case LowerArm0:
			{
				const int32 Sd = B.Side;
				TurnB[0][Sd] = FQuat::FindBetweenNormals(TurnA[0][Sd].RotateVector(B.Along), (End[0][Sd] - Mid[0][Sd]).GetSafeNormal()) * TurnA[0][Sd];
				Cs[I] = FTransform(TurnB[0][Sd] * Ref, Mid[0][Sd]);
				break;
			}
			case Hand0:
			{
				const int32 Sd = B.Side;
				FQuat Turn = TurnB[0][Sd];
				FVector Point = S.Fingers[Sd];
				if (Point.Normalize())
				{
					Turn = FQuat::FindBetweenNormals(Turn.RotateVector(B.Along), Point) * Turn;
				}
				Cs[I] = FTransform(Turn * Ref, End[0][Sd]);
				OutWrist[Sd] = End[0][Sd];
				OutHold[Sd] = End[0][Sd] + (End[0][Sd] - Mid[0][Sd]).GetSafeNormal() * BbMotion::BallInHandCm;
				break;
			}
			case Thigh0:
			{
				const int32 Sd = B.Side;
				const FVector Hip = Fk.GetLocation();
				TwoBone(Hip, S.Ankle[Sd], Thigh, Calf, S.Knee[Sd], Mid[1][Sd], End[1][Sd]);
				TurnA[1][Sd] = FQuat::FindBetweenNormals(Dp.RotateVector(B.Along), (Mid[1][Sd] - Hip).GetSafeNormal()) * Dp;
				Cs[I] = FTransform(TurnA[1][Sd] * Ref, Hip);
				break;
			}
			case Calf0:
			{
				const int32 Sd = B.Side;
				TurnB[1][Sd] = FQuat::FindBetweenNormals(TurnA[1][Sd].RotateVector(B.Along), (End[1][Sd] - Mid[1][Sd]).GetSafeNormal()) * TurnA[1][Sd];
				Cs[I] = FTransform(TurnB[1][Sd] * Ref, Mid[1][Sd]);
				break;
			}
			case Foot0:
				Cs[I] = FTransform(Facing(S.Toe[B.Side], FVector(0, 0, 1)) * Ref, End[1][B.Side]);
				break;
			default:
				Cs[I] = Fk;
				break;
			}
		}
		// (Not far, and not far down: a man reaching does not sit down to do it.)
		Shift.Z = FMath::Max(Shift.Z, -14.0);
		Shift = Shift.GetClampedToMaxSize(24.0);
		if (Shift.IsNearlyZero(0.01))
		{
			break;
		}
		PelvisAt += Shift;
	}
	for (int32 I = 0; I < N; ++I)
	{
		OutLocal[I] = Bones[I].Parent >= 0 ? Cs[I].GetRelativeTransform(Cs[Bones[I].Parent]) : Cs[I];
		OutLocal[I].NormalizeRotation();
	}
}

namespace
{
	struct FKey
	{
		float T;
		FBbStick S;
	};

	/** The pose at T from keys in time order: a curve through them that keeps its speed
	 *  across each key, so the motion flows instead of stopping at every one. */
	FBbStick Through(const TArray<FKey>& Keys, float T)
	{
		const int32 N = Keys.Num();
		if (T <= Keys[0].T)
		{
			return Keys[0].S;
		}
		if (T >= Keys[N - 1].T)
		{
			return Keys[N - 1].S;
		}
		int32 I = 0;
		while (I + 2 < N && T > Keys[I + 1].T)
		{
			++I;
		}
		const FKey& A = Keys[I];
		const FKey& B = Keys[I + 1];
		const FKey& Prev = Keys[FMath::Max(I - 1, 0)];
		const FKey& Next = Keys[FMath::Min(I + 2, N - 1)];
		const double Dt = B.T - A.T, U = (T - A.T) / Dt;
		const double U2 = U * U, U3 = U2 * U;
		const double H00 = 2 * U3 - 3 * U2 + 1, H10 = U3 - 2 * U2 + U, H01 = -2 * U3 + 3 * U2, H11 = U3 - U2;
		FBbStick Out;
		for (int32 K = 0; K < FBbStick::Num; ++K)
		{
			const FVector& Pa = A.S.V()[K];
			const FVector& Pb = B.S.V()[K];
			// The speed through a key: the slope between its neighbours (none at the ends).
			const FVector Ma = I > 0 ? (Pb - Prev.S.V()[K]) / (double)(B.T - Prev.T) : FVector::ZeroVector;
			const FVector Mb = I + 2 < N ? (Next.S.V()[K] - Pa) / (double)(Next.T - A.T) : FVector::ZeroVector;
			Out.V()[K] = Pa * H00 + Ma * (H10 * Dt) + Pb * H01 + Mb * (H11 * Dt);
		}
		return Out;
	}

	/** Puts the ground under each foot: an ankle's third number is how far the foot is off it. */
	void Stand(FBbStick& S, const FBbSkeleton& Skel, const FBbMotionGround& Ground)
	{
		for (int32 Sd = 0; Sd < 2; ++Sd)
		{
			S.Ankle[Sd].Z = Ground.Height(S.Ankle[Sd].X, S.Ankle[Sd].Y) + Skel.AnkleCm() + FMath::Max(0.0, S.Ankle[Sd].Z);
		}
	}

	/** The delivery's keys (BbMotion::Delivery). A right-hander on the rubber: he stands side
	 *  on, chest to third base (-X), glove shoulder to the plate (+Y). */
	const TArray<FKey>& DeliveryKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		auto Add = [&](float T, TFunctionRef<void(FBbStick&)> Fill)
		{
			FKey K;
			K.T = T;
			Fill(K.S);
			Keys.Add(K);
		};
		const int32 L = 0, R = 1;
		auto Set = [&](FBbStick& S)
		{
			S.Pelvis = { 0, 19, 94 };
			S.PelvisFwd = { -1, 0, 0 };
			S.ChestFwd = { -1, 0, -0.05 };
			S.ChestUp = { 0, 0, 1 };
			S.HeadFwd = { -0.35, 1, -0.05 };
			S.Wrist[R] = { 15, 25, -26 };
			S.Wrist[L] = { -15, 25, -26 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
			S.Elbow[L] = { 1, -0.2, -0.6 };
			S.Ankle[R] = { 0, 0, 0 };
			S.Ankle[L] = { 0, 38, 0 };
			S.Knee[R] = { -1, 0, 0 };
			S.Knee[L] = { -1, 0, 0 };
			S.Toe[R] = { -1, 0, 0 };
			S.Toe[L] = { -1, 0, 0 };
		};
		// Set: hands together at the chest, looking in over the glove shoulder.
		Add(BbMotion::DeliveryStartS, Set);
		Add(-1.27f, Set);
		// The leg lift: the front knee to the waist, the weight back over the rubber.
		Add(-0.84f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { 2, 9, 95 };
			S.PelvisFwd = { -1, -0.15, 0 };
			S.ChestFwd = { -1, -0.22, -0.05 };
			S.Wrist[R] = { 15, 23, -19 };
			S.Wrist[L] = { -15, 23, -19 };
			S.Ankle[L] = { -24, 14, 50 };
			S.Knee[L] = { -1, 0.35, 0.7 };
			S.Toe[L] = { -0.55, 0, -0.85 };
		});
		// The stride: the hips go down the mound, the hands break, the ball goes down and back.
		Add(-0.48f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { -2, 54, 85 };
			S.PelvisFwd = { -1, 0.05, 0 };
			S.ChestFwd = { -1, 0, -0.05 };
			S.ChestUp = { 0, -0.14, 1 };
			S.Wrist[R] = { -16, -4, -46 };
			S.Elbow[R] = { -1, -0.5, 0.1 };
			S.Wrist[L] = { 36, 16, -8 };
			S.Elbow[L] = { 0.3, -0.3, -1 };
			S.Knee[R] = { -0.8, 0.6, 0 };
			S.Ankle[L] = { -8, 100, 20 };
			S.Knee[L] = { -0.5, 1, 0.4 };
			S.Toe[L] = { -0.4, 1, -0.15 };
		});
		// The plant: front foot down, hips opening, chest still closed, the arm up and cocked.
		Add(-0.16f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { -2, 100, 76 };
			S.PelvisFwd = { -0.75, 0.65, 0 };
			S.ChestFwd = { -1, 0.15, 0 };
			S.ChestUp = { 0, -0.1, 1 };
			S.HeadFwd = { -0.1, 1, -0.05 };
			S.Wrist[R] = { -26, -6, 26 };
			S.Elbow[R] = { -1, -0.2, -1 };
			S.Wrist[L] = { 30, 18, -10 };
			S.Elbow[L] = { 0.3, -0.3, -1 };
			S.Ankle[R] = { 0, 2, 3 };
			S.Knee[R] = { -0.5, 1, -0.2 };
			S.Toe[R] = { -0.8, 0.4, -0.4 };
			S.Ankle[L] = { 0, 140, 0 };
			S.Knee[L] = { -0.1, 1, 0.3 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The arm laid back: chest square to the plate, the elbow leading the hand.
		Add(-0.045f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { -3, 116, 73 };
			S.PelvisFwd = { -0.1, 1, 0 };
			S.ChestFwd = { -0.35, 1, -0.15 };
			S.ChestUp = { 0.1, 0.15, 1 };
			S.HeadFwd = { 0, 1, -0.05 };
			S.Wrist[R] = { -27, -20, 16 };
			S.Elbow[R] = { -1, 0.5, -0.5 };
			S.Wrist[L] = { 8, 18, -24 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -4, 14, 8 };
			S.Knee[R] = { -0.2, 1, -0.5 };
			S.Toe[R] = { -0.4, 0.6, -0.7 };
			S.Ankle[L] = { 0, 140, 0 };
			S.Knee[L] = { -0.1, 1, 0.3 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// Release: the trunk over the front leg and tipped to the glove side, the arm out and up.
		Add(0.f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { -3, 124, 72 };
			S.PelvisFwd = { 0.15, 1, 0 };
			S.ChestFwd = { 0.4, 1, -0.5 };
			S.ChestUp = { 0.3, 0.4, 1 };
			S.HeadFwd = { 0, 1, -0.1 };
			S.HeadUp = { 0.15, 0.1, 1 };
			S.Wrist[R] = { -38, 22, 28 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
			S.Wrist[L] = { 6, 16, -26 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -8, 30, 14 };
			S.Knee[R] = { -0.1, 1, -0.6 };
			S.Toe[R] = { -0.2, 0.6, -0.8 };
			S.Ankle[L] = { 0, 140, 0 };
			S.Knee[L] = { 0, 1, 0.2 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The follow-through: the arm down across the body, the back leg swinging through.
		Add(0.20f, [&](FBbStick& S)
		{
			Set(S);
			S.Pelvis = { -2, 132, 73 };
			S.PelvisFwd = { 0.4, 1, 0 };
			S.ChestFwd = { 0.7, 0.8, -0.9 };
			S.ChestUp = { 0.2, 0.7, 0.8 };
			S.HeadFwd = { 0, 1, -0.15 };
			S.Wrist[R] = { 28, 30, -34 };
			S.Elbow[R] = { -0.6, 0.2, -1 };
			S.Wrist[L] = { 10, 10, -24 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -30, 92, 40 };
			S.Knee[R] = { -0.3, 1, 0 };
			S.Toe[R] = { 0, 0.6, -0.8 };
			S.Ankle[L] = { 0, 140, 0 };
			S.Knee[L] = { 0, 1, 0.2 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The finish: both feet down, square to the plate, ready for a ball hit back at him.
		Add(BbMotion::DeliveryEndS, [&](FBbStick& S)
		{
			S.Pelvis = { -14, 130, 87 };
			S.ChestFwd = { 0, 1, -0.25 };
			S.ChestUp = { 0, 0.2, 1 };
			S.Wrist[R] = { 6, 24, -34 };
			S.Wrist[L] = { -6, 24, -30 };
			S.Elbow[R] = { -1, -0.3, -0.5 };
			S.Elbow[L] = { 1, -0.3, -0.5 };
			S.Ankle[R] = { -30, 126, 0 };
			S.Knee[R] = { -0.2, 1, 0 };
			S.Toe[R] = { -0.15, 1, 0 };
			S.Ankle[L] = { 2, 140, 0 };
			S.Knee[L] = { 0.1, 1, 0 };
			S.Toe[L] = { 0.1, 1, 0 };
		});
		return Keys;
	}
}

FBbStick BbMotion::Delivery(float T, const FBbSkeleton& Skel, const FBbMotionGround& Ground)
{
	FBbStick S = Through(DeliveryKeys(), T);
	Stand(S, Skel, Ground);
	return S;
}

float BbMotion::DeliveryReach(float T)
{
	return (float)(Smooth(-0.13, -0.01, T) * (1.0 - Smooth(0.0, 0.07, T)));
}

FVector2D BbMotion::DeliveryTravel(float T)
{
	// To where the finish stands him (its pelvis), over the stride and the follow-through,
	// while the pose alone is shown.
	return FVector2D(-15.0, 130.0) * Smooth(-0.5, 0.3, T);
}

FBbStick BbMotion::Crouch(const FVector& MittAt, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach)
{
	const int32 L = 0, R = 1;
	const double Y0 = CrouchBack.Y;
	FBbStick S;
	S.Pelvis = { 0, Y0 - 16, 37 + Ground.Height(0, Y0) };
	S.PelvisFwd = { 0, 1, 0 };
	// He bends to a low pitch and sits up to a high one.
	const double Low = 1.0 - Smooth(15.0, 75.0, MittAt.Z);
	S.ChestFwd = { 0, 1, -0.25 - 0.75 * Low };
	S.ChestUp = { 0, 0.25 + 0.75 * Low, 1 };
	S.HeadFwd = { 0, 1, 0.12 };
	// On the balls of his feet, knees wide.
	S.Ankle[L] = { 24, Y0, 5 };
	S.Ankle[R] = { -24, Y0, 5 };
	S.Knee[L] = { 0.75, 1, 0.5 };
	S.Knee[R] = { -0.75, 1, 0.5 };
	S.Toe[L] = { 0.45, 1, -0.25 };
	S.Toe[R] = { -0.45, 1, -0.25 };
	// The bare hand out of harm's way behind the right leg.
	S.Wrist[R] = { -8, -6, -46 };
	S.Elbow[R] = { -1, -1, 0 };
	S.Wrist[L] = { -8, 40, -5 };
	S.Elbow[L] = { 1, -0.2, -0.8 };
	// The mitt's fingers point away from the middle of him: up for a pitch above the belt,
	// down for a low one, out to the side for a wide one.
	const FVector From(MittAt.X, 0.0, MittAt.Z - 50.0);
	const double Far = From.Size();
	FVector Point = From + FVector(0, 0, 1) * 14.0 * FMath::Exp(-Far / 14.0);
	Point = Point.GetSafeNormal(1e-6, FVector(0, 0, 1)) + FVector(0, 0.3, 0);
	Point.Normalize();
	S.Fingers[L] = Point;
	OutReach.At[L] = MittAt - Point * BallInMittCm;
	OutReach.Weight[L] = 1.f;
	Stand(S, Skel, Ground);
	return S;
}

double BbMotion::HandsDeg(const FVector& Knob, const FVector& Tip)
{
	const FVector H = Knob + (Tip - Knob).GetSafeNormal() * (0.5 * (GripLowCm + GripHighCm));
	return FMath::RadiansToDegrees(FMath::Atan2(H.Y, -H.X));
}

// The mean of the sim's swing-start pose over 5,019 swings of 40 games (swing_pose_probe, 2026-11-23),
// in his frame (the probe's x is toward the plate, here -X): the knob 57 cm back, level with
// 125 cm up, the bat nearly level and laid back toward the catcher.
const FVector BbMotion::LoadKnob(2.9, -56.9, 125.0);
const FVector BbMotion::LoadAxis(0.465, -0.871, 0.160);

void BbMotion::LoadBat(float T, float Ta, bool bTake, float CatchT, const FVector& StanceKnob, const FVector& StanceAxis, FVector& OutKnob, FVector& OutAxis)
{
	const float Arrive = FMath::Max(FMath::Min(BatLoadAtS, Ta - BatCommitLeadS - BatLoadBeforeCommitS), BatSettleS + 0.1f);
	double U = FMath::SmoothStep(BatSettleS, Arrive, T);
	if (bTake && CatchT >= 0.f)
	{
		U *= 1.0 - FMath::SmoothStep(CatchT + BatRelaxAfterCatchS, CatchT + BatRelaxAfterCatchS + BatRelaxOverS, T);
	}
	OutKnob = FMath::Lerp(StanceKnob, LoadKnob, U);
	OutAxis = FMath::Lerp(StanceAxis, LoadAxis.GetSafeNormal(), U).GetSafeNormal(1e-6, StanceAxis);
}

float BbMotion::HandoverWeight(float T, float TL, float TS)
{
	return FMath::SmoothStep(TL, FMath::Max(TS, TL + BatHandoverS), T);
}

FBbStick BbMotion::Swing(const FSwingIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach)
{
	const int32 L = 0, R = 1;
	// A direction seen from above, by degrees round from the plate's side toward the pitcher.
	auto Dir = [](double Deg, double Z)
	{
		const double A = FMath::DegreesToRadians(Deg);
		return FVector(-FMath::Cos(A), FMath::Sin(A), Z);
	};
	const FVector Axis = (In.Tip - In.Knob).GetSafeNormal(1e-6, FVector(0, 0, 1));
	const FVector Hands = In.Knob + Axis * (0.5 * (GripLowCm + GripHighCm));
	const double W = FMath::Clamp((double)In.Swing, 0.0, 1.0);
	const double Turn = FMath::Max(0.0, (double)In.TurnDeg) * W;
	const double Step = FMath::Clamp((double)In.Stride, 0.0, 1.0);
	const double Up = FMath::Sin(PI * Step);
	// The hips go first and are nearly open by contact (the hands about 90 degrees round);
	// the shoulders start turned in and come round with the hands.
	const double Hip = 95.0 * (1.0 - FMath::Exp(-Turn / 45.0));
	const double Still = FMath::Lerp(-20.0, -8.0, (double)In.Rest);
	const double Chest = FMath::Lerp(Still, FMath::Min(-40.0 + 1.08 * Turn, 105.0), W);
	const double Round = FMath::Clamp(Turn / 90.0, 0.0, 1.0);
	// How far he has to go for it: out over the plate, or down.
	const double Far = Smooth(30.0, 75.0, FVector2D(Hands.X, Hands.Y).Size()) * W;
	const double Low = (1.0 - Smooth(85.0, 115.0, Hands.Z)) * W;
	const FVector2D To = FVector2D(Hands.X, Hands.Y).GetSafeNormal();

	FBbStick S;
	S.PelvisFwd = Dir(Hip, 0.0);
	// The weight goes back as the front foot comes up, and through against the front leg
	// as he turns; he sits a little as he strides, more for a low one.
	S.Pelvis = { 5.0 - 3.0 * Round, 3.0 * Step - 9.0 * Up + 7.0 * Round, 88.0 - 6.0 * Step - 5.0 * Low };
	// Bent over the plate; in the swing toward the hands, and the head staying back.
	S.ChestFwd = Dir(Chest, -0.12 - 0.2 * Far - 0.2 * Low);
	S.ChestUp = FVector(-0.16 - 0.2 * Low, -0.18 * Round, 1.0) + FVector(To.X, To.Y, 0.0) * (0.35 * Far);
	// The back foot turns on its toe as the hips come round, the heel up and the knee in.
	S.Ankle[R] = { 0.0, -27.0 + 4.0 * Round, 6.0 * Round };
	S.Toe[R] = Dir(0.85 * Hip, -0.55 * Round);
	S.Knee[R] = Dir(0.9 * Hip, -0.25 * Round);
	// The front foot: up, toward the pitcher, down part open, and firm from then on.
	S.Ankle[L] = { -2.0 * Step, 27.0 + 16.0 * Step, 9.0 * Up };
	S.Toe[L] = Dir(35.0 * Step, -0.3 * Up);
	S.Knee[L] = Dir(25.0 * Step + 20.0 * Round, 0.0);
	// His eyes on the ball, as far round as a neck turns from the chest.
	{
		const FVector Eye(S.Pelvis.X - 6.0, S.Pelvis.Y, S.Pelvis.Z + 68.0);
		FVector See = (In.Look - Eye).GetSafeNormal(1e-6, FVector(0, 1, 0));
		const FVector Front = S.ChestFwd.GetSafeNormal();
		FVector Round3;
		double Angle = 0.0;
		FQuat::FindBetweenNormals(Front, See).ToAxisAndAngle(Round3, Angle);
		const double Most = FMath::DegreesToRadians(105.0);
		if (Angle > Most)
		{
			See = FQuat(Round3, Most).RotateVector(Front);
		}
		See.Z = FMath::Clamp(See.Z, -0.6, 0.35);
		S.HeadFwd = See;
		S.HeadUp = { -0.08, 0.0, 1.0 };
	}
	// Both hands on the handle, his left below his right. The back elbow is out behind him
	// in the stance and comes in to his side as he turns.
	OutReach.At[L] = In.Knob + Axis * GripLowCm;
	OutReach.At[R] = In.Knob + Axis * GripHighCm;
	for (int32 Sd = 0; Sd < 2; ++Sd)
	{
		OutReach.Weight[Sd] = 1.f;
		OutReach.Past[Sd] = (float)GripPastCm;
	}
	S.Elbow[L] = { 0.4, 0.3, -1.0 };
	S.Elbow[R] = FMath::Lerp(FVector(-1.0, -0.5, -0.25), FVector(-0.5, -0.6, -1.0), Round);
	Stand(S, Skel, Ground);
	return S;
}

namespace
{
	FBbStick Mix(const FBbStick& A, const FBbStick& B, double W)
	{
		if (W <= 0.0)
		{
			return A;
		}
		if (W >= 1.0)
		{
			return B;
		}
		FBbStick Out;
		for (int32 K = 0; K < FBbStick::Num; ++K)
		{
			Out.V()[K] = FMath::Lerp(A.V()[K], B.V()[K], W);
		}
		return Out;
	}

	/** His eyes on a point, as far round as a neck turns from the chest. */
	FVector Watch(const FBbStick& S, const FVector& Look, double MostDeg)
	{
		const FVector Eye(S.Pelvis.X, S.Pelvis.Y, S.Pelvis.Z + 68.0);
		FVector See = (Look - Eye).GetSafeNormal(1e-6, FVector(0, 1, 0));
		const FVector Front = S.ChestFwd.GetSafeNormal(1e-6, FVector(0, 1, 0));
		FVector Round;
		double Angle = 0.0;
		FQuat::FindBetweenNormals(Front, See).ToAxisAndAngle(Round, Angle);
		const double Most = FMath::DegreesToRadians(MostDeg);
		if (Angle > Most)
		{
			See = FQuat(Round, Most).RotateVector(Front);
		}
		See.Z = FMath::Clamp(See.Z, -0.7, 0.8);
		return See;
	}

	/** A fielder's throw (BbMotion::Field), toward +Y: a right-hander turns side on (chest to
	 *  -X, glove shoulder at the target), steps at it and throws three-quarter. The stride is
	 *  laid out about where he stands, so that he ends the throw near the same spot. */
	const TArray<FKey>& ThrowKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		auto Add = [&](float T, TFunctionRef<void(FBbStick&)> Fill)
		{
			FKey K;
			K.T = T;
			Fill(K.S);
			Keys.Add(K);
		};
		const int32 L = 0, R = 1;
		// The gather: the ball brought in to the hand, the body starting to turn.
		Add(BbMotion::ThrowStartS, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -8, 91 };
			S.PelvisFwd = { -0.5, 0.85, 0 };
			S.ChestFwd = { -0.5, 0.85, -0.1 };
			S.HeadFwd = { 0, 1, 0 };
			S.Wrist[R] = { 15, 25, -26 };
			S.Wrist[L] = { -15, 25, -26 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
			S.Elbow[L] = { 1, -0.2, -0.6 };
			S.Ankle[R] = { -12, -18, 0 };
			S.Ankle[L] = { 12, -4, 0 };
			S.Knee[R] = { -0.5, 0.85, 0 };
			S.Knee[L] = { -0.5, 0.85, 0 };
			S.Toe[R] = { -0.5, 0.85, 0 };
			S.Toe[L] = { -0.5, 0.85, 0 };
		});
		// Side on, the weight on the back foot, the hands breaking, the front foot coming up.
		Add(-0.27f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -22, 88 };
			S.PelvisFwd = { -1, 0.1, 0 };
			S.ChestFwd = { -1, 0, -0.05 };
			S.ChestUp = { 0, -0.1, 1 };
			S.HeadFwd = { -0.3, 1, 0 };
			S.Wrist[R] = { -16, -4, -44 };
			S.Elbow[R] = { -1, -0.5, 0.1 };
			S.Wrist[L] = { 32, 16, -10 };
			S.Elbow[L] = { 0.3, -0.3, -1 };
			S.Ankle[R] = { -2, -40, 0 };
			S.Knee[R] = { -0.9, 0.4, 0 };
			S.Toe[R] = { -1, 0.1, 0 };
			S.Ankle[L] = { -4, -6, 12 };
			S.Knee[L] = { -0.6, 1, 0.3 };
			S.Toe[L] = { -0.5, 1, -0.2 };
		});
		// The front foot down, the hips opening, the chest still closed, the arm up and cocked.
		Add(-0.12f, [&](FBbStick& S)
		{
			S.Pelvis = { -1, -4, 84 };
			S.PelvisFwd = { -0.75, 0.65, 0 };
			S.ChestFwd = { -1, 0.15, 0 };
			S.ChestUp = { 0, -0.08, 1 };
			S.HeadFwd = { -0.1, 1, 0 };
			S.Wrist[R] = { -26, -6, 26 };
			S.Elbow[R] = { -1, -0.2, -1 };
			S.Wrist[L] = { 30, 18, -10 };
			S.Elbow[L] = { 0.3, -0.3, -1 };
			S.Ankle[R] = { 0, -40, 2 };
			S.Knee[R] = { -0.5, 1, -0.2 };
			S.Toe[R] = { -0.8, 0.4, -0.3 };
			S.Ankle[L] = { 0, 34, 0 };
			S.Knee[L] = { -0.1, 1, 0.3 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The arm laid back: the chest square to the target, the elbow leading the hand.
		Add(-0.04f, [&](FBbStick& S)
		{
			S.Pelvis = { -2, 6, 84 };
			S.PelvisFwd = { -0.1, 1, 0 };
			S.ChestFwd = { -0.35, 1, -0.1 };
			S.ChestUp = { 0.08, 0.1, 1 };
			S.HeadFwd = { 0, 1, 0 };
			S.Wrist[R] = { -27, -20, 16 };
			S.Elbow[R] = { -1, 0.5, -0.5 };
			S.Wrist[L] = { 8, 18, -24 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -4, -32, 6 };
			S.Knee[R] = { -0.2, 1, -0.5 };
			S.Toe[R] = { -0.4, 0.6, -0.6 };
			S.Ankle[L] = { 0, 34, 0 };
			S.Knee[L] = { -0.1, 1, 0.3 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The release: over the front leg, the arm out and up.
		Add(0.f, [&](FBbStick& S)
		{
			S.Pelvis = { -2, 12, 85 };
			S.PelvisFwd = { 0.15, 1, 0 };
			S.ChestFwd = { 0.35, 1, -0.3 };
			S.ChestUp = { 0.2, 0.25, 1 };
			S.HeadFwd = { 0, 1, -0.05 };
			S.HeadUp = { 0.12, 0.05, 1 };
			S.Wrist[R] = { -38, 22, 28 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
			S.Wrist[L] = { 6, 16, -26 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -8, -20, 10 };
			S.Knee[R] = { -0.1, 1, -0.6 };
			S.Toe[R] = { -0.2, 0.6, -0.8 };
			S.Ankle[L] = { 0, 34, 0 };
			S.Knee[L] = { 0, 1, 0.2 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The follow-through: the arm down across him, the back leg coming through.
		Add(0.18f, [&](FBbStick& S)
		{
			S.Pelvis = { -2, 18, 86 };
			S.PelvisFwd = { 0.4, 1, 0 };
			S.ChestFwd = { 0.6, 0.8, -0.6 };
			S.ChestUp = { 0.15, 0.5, 0.9 };
			S.HeadFwd = { 0, 1, -0.1 };
			S.Wrist[R] = { 28, 30, -34 };
			S.Elbow[R] = { -0.6, 0.2, -1 };
			S.Wrist[L] = { 10, 10, -24 };
			S.Elbow[L] = { 1, -0.3, -0.6 };
			S.Ankle[R] = { -20, 8, 14 };
			S.Knee[R] = { -0.3, 1, 0 };
			S.Toe[R] = { 0, 0.8, -0.6 };
			S.Ankle[L] = { 0, 34, 0 };
			S.Knee[L] = { 0, 1, 0.2 };
			S.Toe[L] = { -0.2, 1, 0 };
		});
		// The finish: both feet down, square to where the ball went.
		Add(BbMotion::ThrowEndS, [&](FBbStick& S)
		{
			S.Pelvis = { -4, 22, 93 };
			S.ChestFwd = { 0, 1, -0.1 };
			S.Wrist[R] = { -4, 10, -48 };
			S.Wrist[L] = { 4, 10, -48 };
			S.Ankle[R] = { -18, 20, 0 };
			S.Ankle[L] = { 10, 34, 0 };
		});
		return Keys;
	}
}

FBbStick BbMotion::Field(const FFieldIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach, FVector& OutPocket)
{
	const int32 L = 0, R = 1;
	// Standing easy, and down and set as the pitch is thrown: feet wide, knees bent, the
	// trunk forward, hands hanging in front of the knees.
	FBbStick S;
	{
		FBbStick Set;
		Set.Pelvis = { 0, -6, 81 };
		Set.ChestFwd = { 0, 1, -0.5 };
		Set.ChestUp = { 0, 0.5, 1 };
		Set.Ankle[L] = { 27, 0, 0 };
		Set.Ankle[R] = { -27, 0, 0 };
		Set.Knee[L] = { 0.35, 1, 0 };
		Set.Knee[R] = { -0.35, 1, 0 };
		Set.Toe[L] = { 0.25, 1, 0 };
		Set.Toe[R] = { -0.25, 1, 0 };
		Set.Wrist[L] = { 0, 20, -44 };
		Set.Wrist[R] = { 0, 20, -44 };
		Set.Elbow[L] = { 1, -0.3, -0.2 };
		Set.Elbow[R] = { -1, -0.3, -0.2 };
		S = Mix(S, Set, Smooth(0.0, 1.0, In.Ready));
	}
	// With the ball: hands together in front of his chest.
	{
		FBbStick Has;
		Has.ChestFwd = { 0, 1, -0.08 };
		Has.Wrist[R] = { 15, 25, -26 };
		Has.Wrist[L] = { -15, 25, -26 };
		Has.Elbow[R] = { -1, -0.2, -0.6 };
		Has.Elbow[L] = { 1, -0.2, -0.6 };
		S = Mix(S, Has, FMath::Clamp((double)In.Carry, 0.0, 1.0));
	}
	// The glove to the ball. He bends for a low one and reaches up for a high one, turns his
	// chest to it, and steps toward one that is away from him with the foot on that side.
	OutPocket = FVector(0, 0.3, 1).GetSafeNormal();
	const double Throw = FMath::Clamp((double)In.Throw, 0.0, 1.0);
	if (In.Glove > 0.f)
	{
		// The sim has many a ball caught over the man's own feet. He takes it in front of
		// him: he gives ground until it is a forearm ahead of his middle.
		const double Back = FMath::Clamp(34.0 - In.GloveAt.Y, 0.0, 50.0) * (1.0 - Smooth(40.0, 70.0, FMath::Abs(In.GloveAt.X)));
		const FVector2D Flat(In.GloveAt.X, In.GloveAt.Y + Back);
		const double Dist = Flat.Size();
		// (Which way a ball close in lies means little; one away from him he turns and leans to.)
		const double Away = Smooth(25.0, 60.0, Dist);
		const FVector2D Dir = (Dist > 1.0 ? Flat / Dist : FVector2D(0.0, 1.0)) * Away + FVector2D(0.0, 1.0 - Away);
		const double Low = 1.0 - Smooth(25.0, 95.0, In.GloveAt.Z);
		const double High = Smooth(170.0, 240.0, In.GloveAt.Z);
		const double Far = Smooth(45.0, 110.0, Dist);
		FBbStick G;
		// (One over his head he goes up for: BbMotion::JumpForCm.)
		const double Jump = JumpForCm(In.GloveAt.Z);
		G.Pelvis = { Dir.X * 18.0 * Far, 2.0 + Dir.Y * 18.0 * Far - 14.0 * Low, 96.0 - 58.0 * Low - 4.0 * Far + Jump };
		G.ChestFwd = FVector(Dir.X * 0.6, 0.6 + Dir.Y * 0.6, -0.15 - 1.7 * Low + 0.3 * High);
		G.ChestUp = FVector(Dir.X * 0.3 * Far, 0.2 + 1.0 * Low + Dir.Y * 0.3 * Far, 1.0);
		const double Wide = 14.0 + 14.0 * Low;
		G.Ankle[L] = { Wide, 0, Jump };
		G.Ankle[R] = { -Wide, -4, Jump };
		if (Jump > 0.0)
		{
			G.Toe[L] = G.Toe[R] = FVector(0.0, 1.0, -FMath::Min(Jump / 15.0, 1.2));
		}
		const int32 Near = Dir.X >= 0.0 ? L : R;
		G.Ankle[Near] += FVector(Dir.X, Dir.Y, 0.0) * (45.0 * Far);
		G.Toe[Near] = FVector(Dir.X * Far, 1.0 - Far + Dir.Y * Far, 0.0).GetSafeNormal(1e-6, FVector(0, 1, 0));
		G.Knee[L] = { 0.3, 1, 0 };
		G.Knee[R] = { -0.3, 1, 0 };
		G.Knee[Near] = FMath::Lerp(G.Knee[Near], FVector(Dir.X, Dir.Y, 0.0), 0.7 * Far);
		G.Wrist[L] = { -4, 34, -20 };
		G.Elbow[L] = { 1, -0.2, -0.8 };
		G.Pelvis.Y -= Back;
		G.Ankle[L].Y -= Back;
		G.Ankle[R].Y -= Back;
		// The bare hand beside the glove for a ball in front of him, out of the way otherwise.
		const double Two = (1.0 - Far) * (1.0 - High);
		G.Wrist[R] = FMath::Lerp(FVector(-6, 6, -50), FVector(4, 30, -28 - 14 * Low), Two);
		G.Elbow[R] = { -1, -0.3, -0.5 };
		// The glove's fingers point away from his middle: up above the belt, down below it,
		// out to the side for a wide one.
		const FVector From(In.GloveAt.X, 0.0, In.GloveAt.Z - 100.0);
		FVector Point = From + FVector(0, 0, 1) * 14.0 * FMath::Exp(-From.Size() / 14.0);
		Point = Point.GetSafeNormal(1e-6, FVector(0, 0, 1)) + FVector(0, 0.3, 0);
		Point.Normalize();
		G.Fingers[L] = Point;
		OutPocket = Point;
		// What the bend, the turn and the step leave short, the whole of him makes up: he
		// lunges at one away from him, gets down to one on the ground, leaves his feet for
		// one overhead.
		{
			const FVector Shoulder = G.Pelvis + G.ChestUp.GetSafeNormal() * 46.0 + G.ChestFwd.GetSafeNormal() * 6.0;
			const FVector Need = In.GloveAt - Point * BallInMittCm - Shoulder;
			const double Short = Need.Size() - 0.9 * Skel.ArmCm();
			if (Short > 0.0)
			{
				const FVector Go = Need.GetSafeNormal() * FMath::Min(Short, 90.0);
				const double Up = FMath::Max(Go.Z, 0.0), Down = FMath::Max(-Go.Z, 0.0);
				G.Pelvis += FVector(Go.X, Go.Y, Up - FMath::Min(Down, FMath::Max(G.Pelvis.Z - 34.0, 0.0)));
				G.Ankle[Near] += FVector(Go.X, Go.Y, 0.0);
				G.Ankle[L].Z += Up;
				G.Ankle[R].Z += Up;
			}
		}
		OutReach.At[L] = In.GloveAt - Point * BallInMittCm;
		OutReach.Weight[L] = (float)(FMath::Clamp((double)In.Glove, 0.0, 1.0) * (1.0 - Throw));
		S = Mix(S, G, Smooth(0.0, 1.0, In.Glove));
	}
	if (Throw > 0.0)
	{
		FBbStick Th = Through(ThrowKeys(), In.ThrowT);
		Th.Pelvis.Z -= In.ThrowSink;
		S = Mix(S, Th, Throw);
	}
	// On the run the hips are the clip's, and the trunk is built on them.
	const double Lower = FMath::Clamp((double)In.Lower, 0.0, 1.0);
	S.Pelvis = FMath::Lerp(In.ClipPelvis, S.Pelvis, Lower);
	S.PelvisFwd = FMath::Lerp(In.ClipFwd, S.PelvisFwd, Lower);
	S.PelvisUp = FMath::Lerp(In.ClipUp, S.PelvisUp, Lower);
	// His eyes on the ball; through the throw, on where it is going.
	S.HeadFwd = FMath::Lerp(Watch(S, In.Look, 80.0), S.HeadFwd, Throw);
	Stand(S, Skel, Ground);
	return S;
}

namespace
{
	FKey Key(float T, TFunctionRef<void(FBbStick&)> Fill)
	{
		FKey K;
		K.T = T;
		Fill(K.S);
		return K;
	}

	/** Lying on his front where the dive left him, the glove arm out ahead. */
	void DiveDown(FBbStick& S)
	{
		const int32 L = 0, R = 1;
		S.Pelvis = { 0, -12, 15 };
		S.PelvisUp = { 0, 1, 0.05 };
		S.PelvisFwd = { 0, 0.05, -1 };
		S.ChestUp = { 0, 1, 0.22 };
		S.ChestFwd = { 0, 0.22, -1 };
		S.HeadFwd = { 0, 1, -0.25 };
		S.HeadUp = { 0, 0.5, 1 };
		S.Ankle[L] = { 13, -96, 0 };
		S.Ankle[R] = { -13, -96, 0 };
		S.Knee[L] = { 0.1, 0, -1 };
		S.Knee[R] = { -0.1, 0, -1 };
		S.Toe[L] = { 0.4, -1, -0.1 };
		S.Toe[R] = { -0.4, -1, -0.1 };
		S.Wrist[L] = { 4, 4, 52 };
		S.Elbow[L] = { 1, -0.3, 0 };
		S.Fingers[L] = { 0, 1, 0.1 };
		S.Wrist[R] = { -24, 16, 14 };
		S.Elbow[R] = { -1, -0.6, 0 };
	}

	/** The fall, by seconds since he left his feet. */
	const TArray<FKey>& DiveKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		const int32 L = 0, R = 1;
		// The drive: off the back foot, the trunk going over, the glove leading.
		Keys.Add(Key(0.f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -14, 74 };
			S.PelvisUp = { 0, 0.5, 1 };
			S.PelvisFwd = { 0, 1, -0.5 };
			S.ChestUp = { 0, 0.9, 1 };
			S.ChestFwd = { 0, 1, -0.9 };
			S.HeadFwd = { 0, 1, -0.3 };
			S.Ankle[L] = { 12, -30, 0 };
			S.Ankle[R] = { -12, -58, 4 };
			S.Knee[L] = { 0.2, 1, 0 };
			S.Knee[R] = { -0.2, 1, -0.3 };
			S.Toe[R] = { 0, 0.6, -0.8 };
			S.Wrist[L] = { 6, 14, 40 };
			S.Elbow[L] = { 1, -0.3, 0 };
			S.Fingers[L] = { 0, 1, 0.1 };
			S.Wrist[R] = { -12, 20, -18 };
			S.Elbow[R] = { -1, -0.6, 0 };
		}));
		// Laid out in the air.
		Keys.Add(Key(0.16f, [&](FBbStick& S)
		{
			DiveDown(S);
			S.Pelvis = { 0, -12, 50 };
			S.PelvisUp = { 0, 1, 0.12 };
			S.PelvisFwd = { 0, 0.12, -1 };
			S.ChestUp = { 0, 1, 0.2 };
			S.ChestFwd = { 0, 0.2, -1 };
			S.Ankle[L] = { 12, -95, 36 };
			S.Ankle[R] = { -12, -95, 42 };
			S.Wrist[R] = { -22, 10, 22 };
		}));
		Keys.Add(Key(BbMotion::DiveFallS, DiveDown));
		return Keys;
	}

	/** Getting up, by seconds since he began to. */
	const TArray<FKey>& RiseKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		const int32 L = 0, R = 1;
		Keys.Add(Key(0.f, DiveDown));
		// Onto his hands and knees, the glove coming in under him.
		Keys.Add(Key(0.3f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -30, 48 };
			S.PelvisUp = { 0, 1, 0.25 };
			S.PelvisFwd = { 0, 0.25, -1 };
			S.ChestUp = { 0, 1, 0.45 };
			S.ChestFwd = { 0, 0.45, -1 };
			S.HeadFwd = { 0, 1, -0.3 };
			S.HeadUp = { 0, 0.4, 1 };
			S.Ankle[L] = { 13, -78, 0 };
			S.Ankle[R] = { -13, -78, 0 };
			S.Knee[L] = { 0.1, 1, -0.7 };
			S.Knee[R] = { -0.1, 1, -0.7 };
			S.Toe[L] = { 0.3, -1, -0.2 };
			S.Toe[R] = { -0.3, -1, -0.2 };
			S.Wrist[L] = { 8, 30, 10 };
			S.Elbow[L] = { 1, -0.4, 0 };
			S.Wrist[R] = { -20, 42, 6 };
			S.Elbow[R] = { -1, -0.4, 0 };
		}));
		// One foot under him, the other knee still down, the hands coming together.
		Keys.Add(Key(0.6f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -12, 60 };
			S.PelvisUp = { 0, 0.3, 1 };
			S.PelvisFwd = { 0, 1, -0.3 };
			S.ChestUp = { 0, 0.35, 1 };
			S.ChestFwd = { 0, 1, -0.35 };
			S.Ankle[R] = { -15, 22, 0 };
			S.Knee[R] = { -0.2, 1, 0.4 };
			S.Ankle[L] = { 14, -62, 2 };
			S.Knee[L] = { 0.1, 1, -0.6 };
			S.Toe[L] = { 0.3, -1, -0.2 };
			S.Wrist[L] = { -4, 28, -18 };
			S.Elbow[L] = { 1, -0.2, -0.6 };
			S.Wrist[R] = { 4, 26, -20 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
		}));
		// Up, the ball in both hands.
		Keys.Add(Key(BbMotion::DiveRiseS, [&](FBbStick& S)
		{
			S.ChestFwd = { 0, 1, -0.08 };
			S.Wrist[R] = { 15, 25, -26 };
			S.Wrist[L] = { -15, 25, -26 };
			S.Elbow[R] = { -1, -0.2, -0.6 };
			S.Elbow[L] = { 1, -0.2, -0.6 };
		}));
		return Keys;
	}

	/** The slide, by seconds from his arrival at the bag. */
	const TArray<FKey>& SlideKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		const int32 L = 0, R = 1;
		// Still running: a last stride, the right leg forward.
		Keys.Add(Key(BbMotion::SlideStartS, [&](FBbStick& S)
		{
			S.Pelvis = { 0, 0, 92 };
			S.Ankle[R] = { -10, 36, 4 };
			S.Ankle[L] = { 10, -32, 18 };
			S.Knee[L] = { 0, 1, -0.2 };
			S.Wrist[L] = { 8, 22, -30 };
			S.Wrist[R] = { -8, -14, -36 };
		}));
		// Going down: he sits back, the left leg folding under, the right reaching.
		Keys.Add(Key(-0.3f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -22, 60 };
			S.PelvisUp = { 0, -0.35, 1 };
			S.PelvisFwd = { 0, 1, 0.35 };
			S.ChestUp = { 0, -0.2, 1 };
			S.ChestFwd = { 0, 1, 0.2 };
			S.Ankle[R] = { -10, 52, 10 };
			S.Knee[R] = { -0.1, 1, 0.6 };
			S.Toe[R] = { 0, 1, 0.4 };
			S.Ankle[L] = { 12, 4, 4 };
			S.Knee[L] = { 0.7, 1, 0.1 };
			S.Toe[L] = { -0.5, 1, 0 };
			S.Wrist[L] = { 34, 6, -8 };
			S.Elbow[L] = { 1, -0.4, -0.5 };
			S.Wrist[R] = { -34, 6, -8 };
			S.Elbow[R] = { -1, -0.4, -0.5 };
		}));
		// Down and sliding: on his left hip, leaning back, hands up off the dirt.
		auto Down = [&](FBbStick& S)
		{
			S.Pelvis = { 4, -58, 21 };
			S.PelvisUp = { 0.1, -0.8, 1 };
			S.PelvisFwd = { 0, 1, 0.8 };
			S.ChestUp = { 0.05, -0.5, 1 };
			S.ChestFwd = { 0, 1, 0.5 };
			S.HeadFwd = { 0, 1, -0.15 };
			S.Ankle[R] = { -8, 24, 6 };
			S.Knee[R] = { -0.1, 0.2, 1 };
			S.Toe[R] = { 0, 0.5, 1 };
			S.Ankle[L] = { -4, -28, 0 };
			S.Knee[L] = { 1, 0.6, 0.1 };
			S.Toe[L] = { -1, 0.3, 0 };
			S.Wrist[L] = { 44, 12, -2 };
			S.Elbow[L] = { 0.6, -0.5, -1 };
			S.Wrist[R] = { -44, 12, -2 };
			S.Elbow[R] = { -0.6, -0.5, -1 };
		};
		Keys.Add(Key(-0.16f, Down));
		// The foot on the bag.
		auto There = [&](FBbStick& S)
		{
			Down(S);
			S.Pelvis = { 4, -64, 20 };
			S.Ankle[R] = { -6, 14, 5 };
			S.Knee[R] = { -0.1, 0.1, 1 };
		};
		Keys.Add(Key(0.f, There));
		// He sits up.
		Keys.Add(Key(0.35f, [&](FBbStick& S)
		{
			There(S);
			S.PelvisUp = { 0.05, -0.45, 1 };
			S.PelvisFwd = { 0, 1, 0.45 };
			S.ChestUp = { 0, 0.05, 1 };
			S.ChestFwd = { 0, 1, -0.05 };
			S.Wrist[L] = { 26, 12, -30 };
			S.Wrist[R] = { -26, 12, -30 };
		}));
		// Up over the bag foot, pushing off the other.
		Keys.Add(Key(0.8f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -26, 62 };
			S.PelvisUp = { 0, 0.3, 1 };
			S.PelvisFwd = { 0, 1, -0.3 };
			S.ChestUp = { 0, 0.4, 1 };
			S.ChestFwd = { 0, 1, -0.4 };
			S.Ankle[R] = { -8, 8, 0 };
			S.Knee[R] = { -0.2, 1, 0.3 };
			S.Ankle[L] = { 13, -50, 2 };
			S.Knee[L] = { 0.4, 1, -0.3 };
			S.Toe[L] = { 0.3, 1, -0.5 };
			S.Wrist[L] = { 14, 24, -30 };
			S.Wrist[R] = { -14, 24, -30 };
		}));
		// Standing on the bag.
		Keys.Add(Key(BbMotion::SlideEndS, [&](FBbStick& S) {}));
		return Keys;
	}
}

namespace
{
	/** Down on his front after a dive back, his right hand on the bag at the origin, the left
	 *  beside it, his body laid out behind. */
	void DiveBackDown(FBbStick& S)
	{
		const int32 L = 0, R = 1;
		S.Pelvis = { 0, -100, 15 };
		S.PelvisUp = { 0, 1, 0.05 };
		S.PelvisFwd = { 0, 0.05, -1 };
		S.ChestUp = { 0, 1, 0.15 };
		S.ChestFwd = { 0, 0.15, -1 };
		S.HeadFwd = { 0, 1, 0.1 };
		S.HeadUp = { 0, 0.2, 1 };
		S.Ankle[L] = { 13, -184, 0 };
		S.Ankle[R] = { -13, -184, 0 };
		S.Knee[L] = { 0.1, 0, -1 };
		S.Knee[R] = { -0.1, 0, -1 };
		S.Toe[L] = { 0.4, -1, -0.1 };
		S.Toe[R] = { -0.4, -1, -0.1 };
		S.Wrist[L] = { 14, 6, 46 };
		S.Elbow[L] = { 1, -0.4, 0 };
		S.Fingers[L] = { 0.2, 1, 0 };
		S.Wrist[R] = { -4, 6, 54 };
		S.Elbow[R] = { -1, -0.4, 0 };
		S.Fingers[R] = { 0, 1, 0 };
	}

	/** The dive back in the air, by how far through the glide he is (0 leaving his feet, 1 the
	 *  hand on the bag). */
	const TArray<FKey>& DiveBackKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		const int32 L = 0, R = 1;
		// Off the left foot, low, the trunk going over, both hands reaching for the bag. (The
		// origin is still his path point here, 0.4 m behind where the hand will be.)
		Keys.Add(Key(0.f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -20, 66 };
			S.PelvisUp = { 0, 0.6, 1 };
			S.PelvisFwd = { 0, 1, -0.6 };
			S.ChestUp = { 0, 1, 0.7 };
			S.ChestFwd = { 0, 0.7, -1 };
			S.HeadFwd = { 0, 1, -0.2 };
			S.Ankle[L] = { 12, -38, 0 };
			S.Ankle[R] = { -12, -80, 14 };
			S.Knee[L] = { 0.2, 1, 0 };
			S.Knee[R] = { -0.2, 1, -0.3 };
			S.Toe[R] = { 0, 0.6, -0.8 };
			S.Wrist[L] = { 10, 12, 42 };
			S.Elbow[L] = { 1, -0.3, 0 };
			S.Wrist[R] = { -6, 12, 46 };
			S.Elbow[R] = { -1, -0.3, 0 };
		}));
		// Laid out, coming down chest first.
		Keys.Add(Key(0.55f, [&](FBbStick& S)
		{
			DiveBackDown(S);
			S.Pelvis = { 0, -104, 34 };
			S.PelvisUp = { 0, 1, 0.12 };
			S.PelvisFwd = { 0, 0.12, -1 };
			S.Ankle[L] = { 12, -186, 26 };
			S.Ankle[R] = { -12, -186, 30 };
		}));
		Keys.Add(Key(1.f, DiveBackDown));
		return Keys;
	}

	/** Up from a dive back, by seconds since he began to: onto his hands and knees with the
	 *  hands on the bag, a foot under him, standing on the bag. */
	const TArray<FKey>& DiveBackRiseKeys()
	{
		static TArray<FKey> Keys;
		if (Keys.Num() > 0)
		{
			return Keys;
		}
		const int32 L = 0, R = 1;
		Keys.Add(Key(0.f, DiveBackDown));
		Keys.Add(Key(0.3f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -66, 46 };
			S.PelvisUp = { 0, 1, 0.25 };
			S.PelvisFwd = { 0, 0.25, -1 };
			S.ChestUp = { 0, 1, 0.4 };
			S.ChestFwd = { 0, 0.4, -1 };
			S.HeadFwd = { 0, 1, -0.2 };
			S.HeadUp = { 0, 0.4, 1 };
			S.Ankle[L] = { 13, -118, 0 };
			S.Ankle[R] = { -13, -118, 0 };
			S.Knee[L] = { 0.1, 1, -0.7 };
			S.Knee[R] = { -0.1, 1, -0.7 };
			S.Toe[L] = { 0.3, -1, -0.2 };
			S.Toe[R] = { -0.3, -1, -0.2 };
			S.Wrist[L] = { 6, 34, 8 };
			S.Elbow[L] = { 1, -0.4, 0 };
			S.Wrist[R] = { -6, 34, 8 };
			S.Elbow[R] = { -1, -0.4, 0 };
		}));
		// The right foot up to the bag, the left knee still down.
		Keys.Add(Key(0.5f, [&](FBbStick& S)
		{
			S.Pelvis = { 0, -26, 58 };
			S.PelvisUp = { 0, 0.3, 1 };
			S.PelvisFwd = { 0, 1, -0.3 };
			S.ChestUp = { 0, 0.4, 1 };
			S.ChestFwd = { 0, 1, -0.4 };
			S.Ankle[R] = { -15, -4, 0 };
			S.Knee[R] = { -0.2, 1, 0.4 };
			S.Ankle[L] = { 14, -76, 2 };
			S.Knee[L] = { 0.1, 1, -0.6 };
			S.Toe[L] = { 0.3, -1, -0.2 };
			S.Wrist[L] = { 10, 20, -30 };
			S.Wrist[R] = { -10, 20, -30 };
		}));
		// Standing on the bag.
		Keys.Add(Key(BbMotion::DiveBackRiseS, [&](FBbStick& S) {}));
		return Keys;
	}
}

FBbStick BbMotion::DiveBack(float Since, float EndIn, float UpIn, bool bTouched, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach)
{
	const int32 R = 1;
	FBbStick S;
	double Hand = 0.0;
	if (EndIn > 0.f)
	{
		const float F = Since / FMath::Max(Since + EndIn, 0.05f);
		S = Through(DiveBackKeys(), F);
		Hand = Smooth(0.5, 1.0, F);
	}
	else if (UpIn > DiveBackRiseS)
	{
		S = Through(DiveBackKeys(), 1.f);
		Hand = 1.0;
	}
	else
	{
		const float T = DiveBackRiseS - UpIn;
		S = Through(DiveBackRiseKeys(), T);
		Hand = 1.0 - Smooth(0.3, 0.5, T);
	}
	// The hand on the bag where and when the sim has it (invariant 4); short of it, on the
	// dirt where the sim's glide left it.
	OutReach.At[R] = FVector(0.0, 0.0, bTouched ? 6.0 : 2.0);
	OutReach.Weight[R] = (float)Hand;
	Stand(S, Skel, Ground);
	return S;
}

FBbStick BbMotion::Dive(float Since, float UpIn, const FVector& GloveAt, float Glove, const FVector& Look, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach, FVector& OutPocket)
{
	const int32 L = 0;
	FBbStick S = UpIn < DiveRiseS ? Through(RiseKeys(), DiveRiseS - UpIn) : Through(DiveKeys(), Since);
	// The glove's fingers lead: toward the ball if there is one to take.
	FVector Point = FVector(GloveAt.X * 0.6, 60.0, FMath::Max(GloveAt.Z - 30.0, 0.0) * 0.6).GetSafeNormal();
	const double W = FMath::Clamp((double)Glove, 0.0, 1.0);
	if (W > 0.0)
	{
		S.Fingers[L] = FMath::Lerp(S.Fingers[L], Point, W);
		OutReach.At[L] = GloveAt - Point * BallInMittCm;
		OutReach.Weight[L] = (float)W;
		// A ball farther off than his arm: he is that much farther on his way to it (the
		// sim has the ball in his glove first and his body travelling after).
		const double Lean = FMath::Clamp(GloveAt.Y - 100.0, 0.0, 60.0) * W;
		S.Pelvis.Y += Lean;
		S.Ankle[0].Y += Lean;
		S.Ankle[1].Y += Lean;
	}
	OutPocket = S.Fingers[L].GetSafeNormal(1e-6, FVector(0, 1, 0));
	// While he is down his eyes are on the ball; as he gets up, where he is going.
	const double Down = UpIn < DiveRiseS ? Smooth(0.3, 0.6, UpIn) : 1.0;
	S.HeadFwd = FMath::Lerp(S.HeadFwd, Watch(S, Look, 60.0), 0.6 * Down);
	Stand(S, Skel, Ground);
	return S;
}

FBbStick BbMotion::Slide(float T, const FBbSkeleton& Skel, const FBbMotionGround& Ground)
{
	FBbStick S = Through(SlideKeys(), T);
	Stand(S, Skel, Ground);
	return S;
}

double BbMotion::JumpForCm(double BallZ)
{
	return FMath::Clamp(BallZ - 212.0, 0.0, 55.0);
}

float BbMotion::ThrowReach(float T)
{
	return (float)(Smooth(-0.11, -0.01, T) * (1.0 - Smooth(0.0, 0.07, T)));
}

namespace
{
	/** A direction given in the figure's frame, as the chest (which may be bent over) has it:
	 *  wrists and elbows are told to the solver that way. */
	FVector InChest(const FBbStick& S, const FVector& V)
	{
		const FVector Fwd = S.ChestFwd.GetSafeNormal(1e-6, FVector(0, 1, 0));
		const FVector Up = (S.ChestUp - Fwd * FVector::DotProduct(S.ChestUp, Fwd)).GetSafeNormal(1e-6, FVector(0, 0, 1));
		const FVector Left = FVector::CrossProduct(Fwd, Up);
		return FVector(FVector::DotProduct(V, Left), FVector::DotProduct(V, Fwd), FVector::DotProduct(V, Up));
	}

	/** A hand put at a point of the figure's frame (given for the left; the right is its
	 *  mirror): the wrist from its shoulder as the chest has it. */
	void HandAt(FBbStick& S, int32 Sd, const FVector& At, const FVector& Elbow)
	{
		const double Sign = Sd == 0 ? 1.0 : -1.0;
		const FVector Fwd = S.ChestFwd.GetSafeNormal(1e-6, FVector(0, 1, 0));
		const FVector Up = (S.ChestUp - Fwd * FVector::DotProduct(S.ChestUp, Fwd)).GetSafeNormal(1e-6, FVector(0, 0, 1));
		// (The shoulder is about 46 cm up the trunk from the pelvis and 18 out.)
		const FVector Shoulder = S.Pelvis + Up * 46.0 + FVector(Sign * 18.0, 0, 0);
		S.Wrist[Sd] = InChest(S, FVector(Sign * At.X, At.Y, At.Z) - Shoulder);
		S.Elbow[Sd] = FVector(Sign * Elbow.X, Elbow.Y, Elbow.Z);
	}
}

FBbStick BbMotion::Crew(const FCrewIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground)
{
	const int32 L = 0, R = 1;
	// Standing easy: his hands at his sides, or on his hips.
	FBbStick Easy;
	Easy.Ankle[L] = { 15, 0, 0 };
	Easy.Ankle[R] = { -15, 0, 0 };
	Easy.Toe[L] = { 0.2, 1, 0 };
	Easy.Toe[R] = { -0.2, 1, 0 };
	if (In.Hips > 0.f)
	{
		FBbStick H = Easy;
		H.Wrist[L] = { 4, 3, -40 };
		H.Wrist[R] = { -4, 3, -40 };
		H.Elbow[L] = { 1, -0.7, 0 };
		H.Elbow[R] = { -1, -0.7, 0 };
		Easy = Mix(Easy, H, In.Hips);
	}
	// Down for the pitch: feet wide, knees bent, bent over with his hands on his knees. The
	// plate umpire squats lower and stays more upright, to see over the catcher.
	FBbStick Down;
	const double Wide = In.bPlate ? 30.0 : 27.0;
	Down.Pelvis = In.bPlate ? FVector(0, -16, 66) : FVector(0, -18, 80);
	Down.ChestFwd = In.bPlate ? FVector(0, 1, -0.45) : FVector(0, 1, -0.75);
	Down.ChestUp = In.bPlate ? FVector(0, 0.45, 1) : FVector(0, 0.75, 1);
	Down.Ankle[L] = { Wide, 0, 0 };
	Down.Ankle[R] = { -Wide, 0, 0 };
	Down.Knee[L] = { 0.35, 1, 0 };
	Down.Knee[R] = { -0.35, 1, 0 };
	Down.Toe[L] = { 0.3, 1, 0 };
	Down.Toe[R] = { -0.3, 1, 0 };
	for (int32 Sd = 0; Sd < 2; ++Sd)
	{
		HandAt(Down, Sd, FVector(Wide + 2.0, In.bPlate ? 22.0 : 14.0, In.bPlate ? 52.0 : 58.0), FVector(1, -0.4, 0.2));
	}
	FBbStick S = Mix(Easy, Down, FMath::Clamp((double)In.Set, 0.0, 1.0));
	if (In.Signal != ESignal::None && In.SignalT >= 0.f && In.SignalT < SignalS)
	{
		const double T = In.SignalT;
		// Into the signal's first shape, on to its second, held, and let go.
		const double Go = Smooth(0.12, 0.34, T);
		FBbStick A = Easy, B;
		A.Wrist[L] = { 4, 4, -48 };
		A.Wrist[R] = { -4, 4, -48 };
		switch (In.Signal)
		{
		case ESignal::Strike:
			// He comes up, turns to his right and points that arm straight out to the side.
			A.Wrist[R] = { -16, 14, 10 };
			A.Elbow[R] = { -1, -0.4, -0.3 };
			B = A;
			B.ChestFwd = { -0.35, 1, 0 };
			B.Wrist[R] = { -50, 12, 2 };
			B.Elbow[R] = { -0.2, -0.3, -1 };
			break;
		case ESignal::Out:
			// The fist up beside his head, then punched forward.
			A.Wrist[R] = { -12, 10, 34 };
			A.Elbow[R] = { -1, -0.2, -0.4 };
			B = A;
			B.ChestFwd = { 0.15, 1, -0.18 };
			B.ChestUp = { 0, 0.18, 1 };
			B.Wrist[R] = { -6, 36, 10 };
			B.Elbow[R] = { -0.8, -0.3, -0.6 };
			B.Wrist[L] = { 8, 6, -40 };
			break;
		case ESignal::Safe:
			// Arms crossed in front of his chest, then swept out flat to both sides.
			A.Wrist[L] = { -20, 30, -6 };
			A.Wrist[R] = { 20, 30, -4 };
			A.Elbow[L] = { 0.6, -0.3, -1 };
			A.Elbow[R] = { -0.6, -0.3, -1 };
			A.Pelvis = { 0, -4, 90 };
			A.ChestFwd = { 0, 1, -0.25 };
			A.ChestUp = { 0, 0.25, 1 };
			A.Ankle[L] = { 24, 0, 0 };
			A.Ankle[R] = { -24, 0, 0 };
			B = A;
			B.Wrist[L] = { 51, 8, -3 };
			B.Wrist[R] = { -51, 8, -3 };
			B.Elbow[L] = { 0, -0.5, -1 };
			B.Elbow[R] = { 0, -0.5, -1 };
			break;
		case ESignal::Fair:
		{
			// The arm on the infield side comes up and points straight out toward fair
			// ground; the other hangs.
			const int32 Arm = In.Point.X > 0.0 ? L : R;
			const double Side = Arm == L ? 1.0 : -1.0;
			A.Wrist[Arm] = { Side * 16.0, 14.0, 10.0 };
			A.Elbow[Arm] = { Side * 1.0, -0.4, -0.3 };
			B = A;
			B.Wrist[Arm] = { In.Point.X * 52.0, In.Point.Y * 52.0 + 8.0, 2.0 };
			B.Elbow[Arm] = { Side * 0.2, -0.3, -1.0 };
			break;
		}
		default:
			// Foul: both arms up and out.
			A.Wrist[L] = { 20, 14, 20 };
			A.Wrist[R] = { -20, 14, 20 };
			A.Elbow[L] = { 1, -0.3, -0.5 };
			A.Elbow[R] = { -1, -0.3, -0.5 };
			B = A;
			B.Wrist[L] = { 24, 6, 46 };
			B.Wrist[R] = { -24, 6, 46 };
			B.Elbow[L] = { 1, -0.4, 0 };
			B.Elbow[R] = { -1, -0.4, 0 };
			break;
		}
		const double W = Smooth(0.0, 0.16, T) * (1.0 - Smooth(SignalS - 0.35, SignalS, T));
		S = Mix(S, Mix(A, B, Go), W);
	}
	S.HeadFwd = Watch(S, In.Look, 70.0);
	Stand(S, Skel, Ground);
	return S;
}

FBbStick BbMotion::Seated(int32 Variant, double SeatCm, const FVector& Look, const FBbSkeleton& Skel)
{
	const int32 L = 0, R = 1;
	FBbStick S;
	// Sitting on the front of the bench: thighs level, shins down, feet flat on the floor.
	S.Pelvis = { 0, -12, SeatCm + 9.0 };
	S.Ankle[L] = { 15, 34, Skel.AnkleCm() };
	S.Ankle[R] = { -15, 34, Skel.AnkleCm() };
	S.Knee[L] = { 0.15, 1, 0.8 };
	S.Knee[R] = { -0.15, 1, 0.8 };
	const double KneeY = 30.0, KneeZ = SeatCm + 12.0;
	switch (((Variant % 3) + 3) % 3)
	{
	case 0:
		// Leaning forward, forearms on his thighs.
		S.ChestFwd = { 0, 1, -0.7 };
		S.ChestUp = { 0, 0.7, 1 };
		for (int32 Sd = 0; Sd < 2; ++Sd)
		{
			HandAt(S, Sd, FVector(8, KneeY + 6, KneeZ + 4), FVector(1, -0.2, 0));
		}
		break;
	case 1:
		// Sitting up, a hand on each thigh.
		S.ChestFwd = { 0, 1, -0.12 };
		S.ChestUp = { 0, 0.12, 1 };
		for (int32 Sd = 0; Sd < 2; ++Sd)
		{
			HandAt(S, Sd, FVector(15, 12, KneeZ + 5), FVector(1, -1, 0));
		}
		break;
	default:
		// Leaning back against the wall, arms folded.
		S.Pelvis = { 0, -16, SeatCm + 9.0 };
		S.ChestFwd = { 0, 1, 0.22 };
		S.ChestUp = { 0, -0.22, 1 };
		S.Ankle[L] = { 17, 44, Skel.AnkleCm() };
		S.Ankle[R] = { -17, 44, Skel.AnkleCm() };
		HandAt(S, L, FVector(-9, 6, SeatCm + 36.0), FVector(1, 0.2, -1));
		HandAt(S, R, FVector(-9, 8, SeatCm + 33.0), FVector(1, 0.2, -1));
		break;
	}
	S.HeadFwd = Watch(S, Look, 60.0);
	return S;
}

void FBbPoseProxy::PreUpdate(UAnimInstance* InAnimInstance, float DeltaSeconds)
{
	FAnimSingleNodeInstanceProxy::PreUpdate(InAnimInstance, DeltaSeconds);
	UBbPoseAnim* Anim = CastChecked<UBbPoseAnim>(InAnimInstance);
	Anim->ClipPelvis = ClipPelvis;
	Anim->bClipPelvis = bClipPelvis;
	Weight = Anim->PoseWeight;
	LowerWeight = Anim->LowerWeight;
	PelvisBone = Anim->PelvisBone;
	if (Upper.Num() != Anim->UpperBones.Num())
	{
		Upper = Anim->UpperBones;
	}
	if (Weight > 0.f)
	{
		Local = Anim->PoseLocal;
	}
}

bool FBbPoseProxy::Evaluate(FPoseContext& Output)
{
	const bool bDone = FAnimSingleNodeInstanceProxy::Evaluate(Output);
	const FBoneContainer& Container = Output.Pose.GetBoneContainer();
	// Where the clip alone has the hips: a motion made on the run builds the trunk on them.
	bClipPelvis = false;
	if (PelvisBone != INDEX_NONE)
	{
		FCompactPoseBoneIndex I = Container.MakeCompactPoseIndex(FMeshPoseBoneIndex(PelvisBone));
		if (I.GetInt() != INDEX_NONE)
		{
			FTransform T = Output.Pose[I];
			for (I = Container.GetParentBoneIndex(I); I.GetInt() != INDEX_NONE; I = Container.GetParentBoneIndex(I))
			{
				T = T * Output.Pose[I];
			}
			T.NormalizeRotation();
			ClipPelvis = T;
			bClipPelvis = true;
		}
	}
	if (Weight > 0.f && Local.Num() > 0)
	{
		for (const FCompactPoseBoneIndex I : Output.Pose.ForEachBoneIndex())
		{
			const int32 M = Container.MakeMeshPoseIndex(I).GetInt();
			if (Local.IsValidIndex(M))
			{
				const float W = Upper.IsValidIndex(M) && !Upper[M] ? Weight * LowerWeight : Weight;
				FTransform& Bone = Output.Pose[I];
				if (W >= 1.f)
				{
					Bone = Local[M];
				}
				else if (W > 0.f)
				{
					Bone.NormalizeRotation();
					Bone.BlendWith(Local[M], W);
					Bone.NormalizeRotation();
				}
			}
		}
	}
	return bDone;
}

FAnimInstanceProxy* UBbPoseAnim::CreateAnimInstanceProxy()
{
	return new FBbPoseProxy(this);
}
