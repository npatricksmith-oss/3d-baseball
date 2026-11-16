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
	Mark(TEXT("pelvis"), Pelvis, 0, 0.f, FString());
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
		// (Not far, and hardly down at all: a man reaching does not sit down to do it.)
		Shift.Z = FMath::Max(Shift.Z, -6.0);
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

void FBbPoseProxy::PreUpdate(UAnimInstance* InAnimInstance, float DeltaSeconds)
{
	FAnimSingleNodeInstanceProxy::PreUpdate(InAnimInstance, DeltaSeconds);
	const UBbPoseAnim* Anim = CastChecked<UBbPoseAnim>(InAnimInstance);
	Weight = Anim->PoseWeight;
	if (Weight > 0.f)
	{
		Local = Anim->PoseLocal;
	}
}

bool FBbPoseProxy::Evaluate(FPoseContext& Output)
{
	const bool bDone = FAnimSingleNodeInstanceProxy::Evaluate(Output);
	if (Weight > 0.f && Local.Num() > 0)
	{
		const FBoneContainer& Container = Output.Pose.GetBoneContainer();
		for (const FCompactPoseBoneIndex I : Output.Pose.ForEachBoneIndex())
		{
			const int32 M = Container.MakeMeshPoseIndex(I).GetInt();
			if (Local.IsValidIndex(M))
			{
				FTransform& Bone = Output.Pose[I];
				if (Weight >= 1.f)
				{
					Bone = Local[M];
				}
				else
				{
					Bone.NormalizeRotation();
					Bone.BlendWith(Local[M], Weight);
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
