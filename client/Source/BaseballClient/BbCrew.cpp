#include "BbCrew.h"
#include "BbFigure.h"
#include "BbFrame.h"
#include "BbLook.h"
#include "BbPark.h"
#include "BbSim.h"
#include "Animation/AnimSequence.h"
#include "Components/SkeletalMeshComponent.h"
#include "Engine/World.h"

using namespace BbFrameConv;

namespace
{
	/** The bases' square, m. */
	const double BaseL = 27.432 / UE_SQRT_2;
	/** Where the umpires stand (decoration: the sim has none, so a ball goes through one).
	 *  The plate umpire this far behind the catcher's place and this far toward the batter's
	 *  side of him; a base umpire this far past his bag along the foul line and this far foul;
	 *  the second-base umpire on the grass behind the bag, to the shortstop's side. */
	const double PlateBackM = 1.45, PlateSlotM = 0.25;
	const double LinePastM = 3.5, LineFoulM = 0.7;
	const FVector2D SecondUmpire(-2.5, 2.0 * BaseL + 7.0);
	/** The coach's box (as BbPark chalks it): 15 ft off the line, 20 ft long, ending level
	 *  with the bag; he stands in its middle. */
	const double CoachOffM = 17.5 * 0.3048, CoachBackM = 10.0 * 0.3048;
	/** How fast a man turns to face something, and goes down or comes up, per second. */
	const float TurnRate = 5.f, SetRate = 3.5f;
	/** How far down a coach goes for the pitch. */
	const float CoachSet = 0.65f;
	/** Invented people's ids, clear of the roster's. */
	const uint32 CrewId = 0x00E00000u;
	/** A man on the bench has his pose worked out again this often, in ticks (his head follows
	 *  the ball; nothing else of him moves), and his bones are left alone in between: 28 more
	 *  figures animated every frame cost a fifth of the frame rate (143 to 116 fps). */
	const uint32 BenchEvery = 45;
	const int32 BenchThawTicks = 3;

	float YawOf(const FVector2D& Face)
	{
		// (Sim x is Unreal's y.)
		return FMath::RadiansToDegrees(FMath::Atan2(Face.X, Face.Y));
	}
}

FVector2D UBbCrew::CoachAt(double Side)
{
	const double S = (BaseL * UE_SQRT_2 - CoachBackM) / UE_SQRT_2, C = CoachOffM / UE_SQRT_2;
	return FVector2D(Side * (S + C), S - C);
}

FVector2D UBbCrew::UmpireAt(int32 Umpire, const FBbCrewTick& In) const
{
	const double Along = LinePastM / UE_SQRT_2, Foul = LineFoulM / UE_SQRT_2;
	switch (Umpire)
	{
	case 0: return FVector2D(In.Catcher.X + (In.BatterX > 0.0 ? PlateSlotM : In.BatterX < 0.0 ? -PlateSlotM : 0.0), In.Catcher.Y - PlateBackM);
	case 1: return FVector2D(BaseL + Along + Foul, BaseL + Along - Foul);
	case 3: return FVector2D(-(BaseL + Along + Foul), BaseL + Along - Foul);
	default: return SecondUmpire;
	}
}

void UBbCrew::Init(AActor* TickAfter, ABbPark* InPark, UBbFigureShop* Shop, const FBbSkeleton* InSkel, USkeletalMesh* Mesh, UAnimSequence* Idle, const FBbSim& Sim)
{
	Park = InPark;
	Skel = InSkel;
	UWorld* World = TickAfter->GetWorld();
	auto Add = [&](EJob Job, int32 Which, const BbPlayer& Who, EBbRole Role)
	{
		ABbFigure* F = World->SpawnActor<ABbFigure>(FVector::ZeroVector, FRotator::ZeroRotator);
		USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
		C->SetMobility(EComponentMobility::Movable);
		C->SetSkeletalMeshAsset(Mesh);
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		C->SetAnimInstanceClass(UBbPoseAnim::StaticClass());
		C->AddTickPrerequisiteActor(TickAfter);
		// The mannequin itself is never drawn (only the pieces on its bones), so it must be
		// told to work its bones out all the same.
		C->VisibilityBasedAnimTickOption = EVisibilityBasedAnimTickOption::AlwaysTickPoseAndRefreshBones;
		C->SetAnimation(Idle);
		C->Play(true);
		F->Build(Shop);
		F->Dress(&Who, Who.side);
		F->SetRole(Role);
		// The dugouts' roofs shade the men under them.
		F->SetShadows(Job != EJob::Bench && Job != EJob::Manager);
		F->SetActorHiddenInGame(true);
		People.Add(F);
		FMan M;
		M.Job = Job;
		M.Which = Which;
		M.Who = Who;
		Men.Add(M);
		return Men.Num() - 1;
	};
	uint32 NextId = CrewId;
	auto Invented = [&](int32 Side, int32 Number)
	{
		BbPlayer P{};
		P.player_id = ++NextId;
		P.side = (uint8)Side;
		P.number = (uint8)Number;
		P.throws = 2;
		return P;
	};
	for (int32 U = 0; U < 4; ++U)
	{
		Add(EJob::Umpire, U, Invented(BbLook::UmpireSide, 0), U == 0 ? EBbRole::PlateUmpire : EBbRole::Plain);
	}
	// The coaches are dressed for the side at bat when it comes up (Tick).
	for (int32 K = 0; K < 2; ++K)
	{
		Add(EJob::Coach, K, Invented(0, 0), EBbRole::Batter);
	}
	for (int32 Side = 0; Side < 2; ++Side)
	{
		FVector2D From, Along, Face;
		double Length = 0.0, SeatZ = 0.0, FloorZ = 0.0;
		// Home on the first-base side, the visitors on the third-base side (as BbDirector::Dugout).
		if (!Park->Bench(Side == 1 ? 1.0 : -1.0, From, Along, Face, Length, SeatZ, FloorZ))
		{
			continue;
		}
		SeatCm = (SeatZ - FloorZ) * 100.0;
		// The nine, then the reserves, who wear numbers the nine do not.
		TArray<BbPlayer> Sitting;
		TSet<int32> Numbers;
		for (uint16 Order = 1; Order <= 9; ++Order)
		{
			if (const BbPlayer* P = Sim.Player((uint16)(Side << 8 | Order)))
			{
				Sitting.Add(*P);
				Numbers.Add(P->number);
			}
		}
		int32 Number = 30;
		auto Spare = [&]()
		{
			while (Numbers.Contains(++Number))
			{
			}
			Numbers.Add(Number);
			return Number;
		};
		for (int32 K = 0; K < BbLook::BenchReserves; ++K)
		{
			Sitting.Add(Invented(Side, Spare()));
		}
		// The end of the bench nearer the plate is the manager's; he stands in front of it.
		const bool bFromEnd = From.SizeSquared() < (From + Along * Length).SizeSquared();
		const double Room = Length - 1.6, Gap = FMath::Min(0.78, Room / FMath::Max(1, Sitting.Num()));
		for (int32 K = 0; K < Sitting.Num(); ++K)
		{
			const double D = 1.6 + Gap * (K + 0.5);
			const FVector2D At = From + Along * (bFromEnd ? D : Length - D);
			const int32 I = Add(EJob::Bench, Side, Sitting[K], EBbRole::Plain);
			Men[I].Seat = ToUE(At.X, At.Y, FloorZ);
			Men[I].SeatYaw = YawOf(Face);
			Men[I].Variant = (int32)((Sitting[K].player_id * 2654435761u) >> 16) % 3;
		}
		const FVector2D At = From + Along * (bFromEnd ? 0.7 : Length - 0.7) + Face * 1.3;
		const int32 I = Add(EJob::Manager, Side, Invented(Side, Spare()), EBbRole::Plain);
		Men[I].Seat = ToUE(At.X, At.Y, Park->GroundZ(At.X, At.Y));
		Men[I].SeatYaw = YawOf(Face);
	}
}

void UBbCrew::SetSignals(const TArray<FBbSignal>& In)
{
	Signals = In;
	Signals.Sort([](const FBbSignal& A, const FBbSignal& B) { return A.T < B.T; });
}

void UBbCrew::Apply(int32 I, const FBbStick& S)
{
	UBbPoseAnim* Anim = Cast<UBbPoseAnim>(People[I]->GetSkeletalMeshComponent()->GetAnimInstance());
	if (!Anim)
	{
		return;
	}
	if (Anim->PelvisBone == INDEX_NONE)
	{
		Anim->UpperBones = Skel->UpperBones();
		Anim->PelvisBone = Skel->PelvisBone();
	}
	FVector Hold[2], Wrist[2];
	Skel->Solve(S, FBbReach(), Anim->PoseLocal, Hold, Wrist);
	Anim->PoseWeight = 1.f;
	Anim->LowerWeight = 1.f;
}

void UBbCrew::Place(int32 I, const FVector2D& At, double Z, const FVector2D& Face, float Dt)
{
	FMan& M = Men[I];
	const float Want = YawOf(Face);
	M.Yaw = M.bPlaced ? M.Yaw + FMath::FindDeltaAngleDegrees(M.Yaw, Want) * FMath::Min(1.f, Dt * TurnRate) : Want;
	People[I]->SetActorLocationAndRotation(ToUE(At.X, At.Y, Z), FRotator(0.f, M.Yaw - 90.f, 0.f));
}

void UBbCrew::Tick(const FBbCrewTick& In)
{
	if (!Skel || !Skel->Ok() || !Park)
	{
		return;
	}
	++Ticks;
	const auto Flat = [](double, double) { return 0.0; };
	const FBbMotionGround Ground{ Flat };
	const FVector BallAt = ToUE(In.Ball.X, In.Ball.Y, In.Ball.Z);
	const FVector PlateAt = ToUE(0.0, 0.3, 0.9), MoundAt = ToUE(0.0, 17.0, 1.7);
	if (In.Batting != Batting)
	{
		// The other side's coaches come out with its batters.
		Batting = In.Batting;
		for (int32 I = 0; I < Men.Num(); ++I)
		{
			if (Men[I].Job == EJob::Coach)
			{
				BbPlayer P{};
				P.player_id = CrewId + 0x100u + (uint32)(Batting * 2 + Men[I].Which);
				P.side = (uint8)Batting;
				P.throws = 2;
				Men[I].Who = P;
				People[I]->Dress(&P, Batting);
				Men[I].bPlaced = false;
			}
		}
	}
	for (int32 I = 0; I < Men.Num(); ++I)
	{
		FMan& M = Men[I];
		ABbFigure* F = People[I];
		if (M.Job == EJob::Bench || M.Job == EJob::Manager)
		{
			const bool bShow = M.Job == EJob::Manager || M.Who.player_id > CrewId || !In.Shown || !In.Shown->Contains(M.Who.player_id);
			const bool bWas = !F->IsHidden();
			F->SetActorHiddenInGame(!bShow);
			if (!bShow)
			{
				M.Solved = 0;
				continue;
			}
			if (!bWas || !M.bPlaced)
			{
				F->SetActorLocationAndRotation(M.Seat, FRotator(0.f, M.SeatYaw - 90.f, 0.f));
				M.bPlaced = true;
				M.Solved = 0;
			}
			// His head follows the ball; nothing else of him moves, so not every tick. His
			// skeleton is worked out for a few ticks after each new pose and then left.
			USkeletalMeshComponent* C = F->GetSkeletalMeshComponent();
			if (M.Solved >= BenchThawTicks && (Ticks + (uint32)I * 7u) % BenchEvery != 0)
			{
				if (++M.Solved >= 2 * BenchThawTicks)
				{
					C->bNoSkeletonUpdate = true;
				}
				continue;
			}
			C->bNoSkeletonUpdate = false;
			M.Solved = FMath::Min(M.Solved + 1, BenchThawTicks);
			const FVector Look = F->GetActorTransform().InverseTransformPosition(In.bBall ? BallAt : PlateAt);
			if (M.Job == EJob::Manager)
			{
				BbMotion::FCrewIn Ci;
				Ci.Hips = 1.f;
				Ci.Look = Look;
				Apply(I, BbMotion::Crew(Ci, *Skel, Ground));
			}
			else
			{
				Apply(I, BbMotion::Seated(M.Variant, SeatCm / FMath::Max(0.5, F->GetActorScale3D().Z), Look, *Skel));
			}
			continue;
		}
		F->SetActorHiddenInGame(false);
		const bool bUmpire = M.Job == EJob::Umpire;
		const FVector2D At = bUmpire ? UmpireAt(M.Which, In) : CoachAt(M.Which == 0 ? 1.0 : -1.0);
		// An umpire's signal: on the play's clock while the play runs (so that a replay shows
		// it again), then on its own until it is done.
		if (bUmpire)
		{
			if (In.Force != BbMotion::ESignal::None)
			{
				M.Signal = In.Force;
				M.SignalT = 0.6f;
				M.SignalAt = FVector2D(0.0, 0.0);
			}
			else if (In.bPlay)
			{
				M.Signal = BbMotion::ESignal::None;
				for (const FBbSignal& S : Signals)
				{
					if (S.Umpire == M.Which && S.T <= In.PlayTime && In.PlayTime - S.T < BbMotion::SignalS)
					{
						M.Signal = S.Kind;
						M.SignalT = In.PlayTime - S.T;
						M.SignalAt = S.At;
					}
				}
			}
			else if (M.Signal != BbMotion::ESignal::None)
			{
				M.SignalT += In.Dt;
				if (M.SignalT >= BbMotion::SignalS)
				{
					M.Signal = BbMotion::ESignal::None;
				}
			}
		}
		const bool bSignal = bUmpire && M.Signal != BbMotion::ESignal::None;
		// He faces the plate for the pitch (the plate umpire, the mound), the ball once it is
		// hit, and the play he is calling.
		const bool bPlate = bUmpire && M.Which == 0;
		FVector2D Toward = bPlate ? FVector2D(At.X, At.Y + 10.0) : FVector2D(0.0, 0.0);
		// (A strike is called facing the mound, with the arm out to the side.)
		if (bSignal && !(bPlate && M.Signal == BbMotion::ESignal::Strike) && In.Force == BbMotion::ESignal::None)
		{
			Toward = M.SignalAt;
		}
		else if (!bSignal && !In.bSet && In.bBall && In.bPlay)
		{
			Toward = FVector2D(In.Ball.X, In.Ball.Y);
		}
		const FVector2D Face = (Toward - At).SizeSquared() > 1e-4 ? (Toward - At).GetSafeNormal() : FVector2D(0.0, 1.0);
		const float Dt = In.Dt;
		Place(I, At, Park->GroundZ(At.X, At.Y), Face, Dt);
		const float Down = In.bSet && !bSignal ? (bUmpire ? 1.f : CoachSet) : 0.f;
		M.Set = M.bPlaced ? FMath::FInterpConstantTo(M.Set, Down, Dt, SetRate) : Down;
		const FVector See = In.bBall && In.bPlay ? BallAt : bPlate ? MoundAt : PlateAt;
		M.Look = M.bPlaced ? FMath::VInterpTo(M.Look, See, Dt, 10.f) : See;
		M.bPlaced = true;
		BbMotion::FCrewIn Ci;
		Ci.Set = M.Set;
		Ci.bPlate = bPlate;
		Ci.Hips = bUmpire ? 0.f : 1.f;
		Ci.Signal = M.Signal;
		Ci.SignalT = M.SignalT;
		Ci.Look = F->GetActorTransform().InverseTransformPosition(M.Look);
		Apply(I, BbMotion::Crew(Ci, *Skel, Ground));
	}
	LastPlayTime = In.PlayTime;
	bWasPlay = In.bPlay;
}
