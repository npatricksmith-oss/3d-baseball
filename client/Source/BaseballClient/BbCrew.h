// The people who are not playing (docs/player-visuals.md §12, owner's Q20): four umpires who
// signal the sim's calls, the batting side's two base coaches, and the men in the dugouts.
// All decoration: nothing here is known to the sim, and nobody here touches the ball.
#pragma once

#include "CoreMinimal.h"
#include "BbPose.h"
#include "bb_ffi.h"
#include "BbCrew.generated.h"

class ABbFigure;
class ABbPark;
class UBbFigureShop;
class USkeletalMesh;
class UAnimSequence;
class FBbSim;

/** A call for an umpire to signal: when (play time), which umpire (0 plate, 1 first, 2 second,
 *  3 third), what, and the spot he turns to (sim frame, m). */
struct FBbSignal
{
	float T = 0.f;
	int32 Umpire = 0;
	BbMotion::ESignal Kind = BbMotion::ESignal::None;
	FVector2D At = FVector2D::ZeroVector;
};

/** What the crew is told every tick. */
struct FBbCrewTick
{
	/** Play time and how much of it passed this tick (0 while paused); false between pitches. */
	float PlayTime = 0.f, Dt = 0.f;
	bool bPlay = false;
	/** The pitch is about to be thrown or on its way: umpires and coaches are down and set. */
	bool bSet = false;
	/** The ball (sim frame, m), if one is shown. */
	bool bBall = false;
	FVector Ball = FVector::ZeroVector;
	/** 0 the visitors bat, 1 the home side. */
	int32 Batting = 0;
	/** The catcher's place as the pitch is thrown (the plate umpire stands behind it whatever
	 *  the catcher then does), and the side of the plate the batter stands on (sim x; 0 none). */
	FVector2D Catcher = FVector2D(0.0, -1.5);
	double BatterX = 0.0;
	/** The players drawn on the field or walking off it, by player id: not on the bench. */
	const TSet<uint32>* Shown = nullptr;
	/** -BbCrewShots: every umpire holds this signal. */
	BbMotion::ESignal Force = BbMotion::ESignal::None;
};

UCLASS()
class UBbCrew : public UObject
{
	GENERATED_BODY()

public:
	/** Makes the figures. TickAfter: the actor that calls Tick (they animate after it). */
	void Init(AActor* TickAfter, ABbPark* InPark, UBbFigureShop* Shop, const FBbSkeleton* InSkel, USkeletalMesh* Mesh, UAnimSequence* Idle, const FBbSim& Sim);
	/** The calls of the pitch about to be shown, in any order. */
	void SetSignals(const TArray<FBbSignal>& In);
	void Tick(const FBbCrewTick& In);
	int32 Num() const { return People.Num(); }
	/** Where an umpire stands as the pitch is thrown (sim frame, m). */
	FVector2D UmpireAt(int32 Umpire, const FBbCrewTick& In) const;
	/** The coach's place by first (+1) or third (-1). */
	static FVector2D CoachAt(double Side);

private:
	enum class EJob : uint8 { Umpire, Coach, Bench, Manager };
	struct FMan
	{
		EJob Job = EJob::Umpire;
		/** Umpire: which. Coach: 0 first, 1 third. Bench and manager: the side. */
		int32 Which = 0;
		BbPlayer Who{};
		/** Bench: his seat, and how he sits. */
		FVector Seat = FVector::ZeroVector;
		float SeatYaw = 0.f;
		int32 Variant = 0;
		/** Eased: how far down he is, which way he faces, what he watches. */
		float Set = 0.f, Yaw = 0.f;
		FVector Look = FVector::ZeroVector;
		bool bPlaced = false;
		/** An umpire's signal under way. */
		BbMotion::ESignal Signal = BbMotion::ESignal::None;
		float SignalT = 0.f;
		FVector2D SignalAt = FVector2D::ZeroVector;
		int32 Solved = 0;
	};
	void Place(int32 I, const FVector2D& At, double Z, const FVector2D& Face, float Dt);
	void Apply(int32 I, const FBbStick& S);

	UPROPERTY()
	TArray<TObjectPtr<ABbFigure>> People;
	UPROPERTY()
	TObjectPtr<ABbPark> Park;
	TArray<FMan> Men;
	TArray<FBbSignal> Signals;
	const FBbSkeleton* Skel = nullptr;
	double SeatCm = 45.0;
	float LastPlayTime = 0.f;
	bool bWasPlay = false;
	int32 Batting = -1;
	uint32 Ticks = 0;
};
