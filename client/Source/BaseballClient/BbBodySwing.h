// The force-driven body's practised swing, drawn as the checking model computed it (PLAN D43;
// docs/player-visuals.md §14). A viewing aid, not the game's swing: no game reads the file and
// nothing here runs in a game.
//
// The file (bb-body-swing 1) is written by analysis/research/body/swing_export.py: every rigid
// body's place and turn, every joint, the bat's knob, sweet spot and tip and each foot's contact
// points every millisecond, in the sim frame (metres), and the ball's line. The figure's pieces
// (UBbFigureShop) are placed segment by segment so that each runs from the file's joint to the
// file's joint: nothing is smoothed, solved or blended, and nothing is drawn where the model has
// no data (before its first sample and after its last, the figure holds that pose).
#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "BbBodySwing.generated.h"

class ABbPark;
class ACameraActor;
class UBbFigureShop;
class UMaterialInterface;
class UStaticMesh;
class UStaticMeshComponent;

/** One swing file, read whole. Positions in the sim frame, metres; times in seconds from contact. */
struct FBbSwingFile
{
	bool Load(const FString& InPath, FString& OutError);

	int32 Num() const { return Samples; }
	double T(int32 I) const { return Data[(int64)I * Columns]; }
	/** Column index of a named body, joint, point or contact point, or INDEX_NONE. */
	int32 Body(const TCHAR* What) const { return Bodies.IndexOfByKey(FString(What)); }
	int32 Joint(const TCHAR* What) const { return Joints.IndexOfByKey(FString(What)); }
	int32 Point(const TCHAR* What) const { return Points.IndexOfByKey(FString(What)); }
	int32 Contact(const TCHAR* What) const { return Contacts.IndexOfByKey(FString(What)); }
	FVector BodyPos(int32 I, int32 B) const { return Vec(I, 1 + 7 * B); }
	/** The body's turn as a quaternion acting on sim-frame vectors (world = q local). */
	FQuat BodyRot(int32 I, int32 B) const;
	FVector JointPos(int32 I, int32 J) const { return Vec(I, JointCol + 3 * J); }
	FVector PointPos(int32 I, int32 P) const { return Vec(I, PointCol + 3 * P); }
	double SweetMps(int32 I) const { return At(I, SweetCol); }
	FVector ContactPos(int32 I, int32 C) const { return Vec(I, ContactCol + 4 * C); }
	double ContactFz(int32 I, int32 C) const { return At(I, ContactCol + 4 * C + 3); }
	FVector BallPos(int32 I) const { return Vec(I, BallCol); }
	/** An event's time, or false if the file does not have it. */
	bool Event(const TCHAR* Name, double& Out) const;

	FString Path, Name, Plan, Commit, Tree;
	double HeightM = 0.0, ContactMps = 0.0, ContactMph = 0.0, MissCm = 0.0;
	double BatLengthM = 0.0, SweetFromKnobM = 0.0;
	TArray<FString> Bodies, Joints, Points, Contacts;
	TArray<TPair<FString, double>> Events;

private:
	double At(int32 I, int32 C) const { return Data[(int64)I * Columns + C]; }
	FVector Vec(int32 I, int32 C) const { return FVector(At(I, C), At(I, C + 1), At(I, C + 2)); }
	TArray<double> Data;
	int32 Columns = 0, Samples = 0;
	int32 JointCol = 0, PointCol = 0, SweetCol = 0, ContactCol = 0, BallCol = 0;
};

/** The file's state at one instant, interpolated between its samples (positions straight,
 *  turns by slerp). Sim frame, metres. */
struct FBbSwingPose
{
	TArray<FVector> BodyPos, JointPos, PointPos, ContactPos;
	TArray<FQuat> BodyRot;
	FVector Ball = FVector::ZeroVector;
	double Sweet = 0.0;
};

/** The segments a piece of the swing's figure can ride on. */
enum EBbSwingSeg : uint8 { SegPelvis, SegTrunk, SegThighL, SegShankL, SegFootL, SegThighR, SegShankR, SegFootR, SegUarmL, SegFarmL, SegHandL, SegUarmR, SegFarmR, SegHandR, SegNumSeg };
struct FBbSwingSeg
{
	/** The mannequin's side, in its mesh frame (cm): where the segment starts, its unit
	 *  axis to its far end, its unit forward and side, and the lengths the file's are set
	 *  against (along the axis, across for the pelvis and trunk, forward for the foot). */
	FVector O, A, F, L;
	double Len = 1.0, Wide = 0.0, Fwd = 0.0;
	/** The file's body that gives its forward; its near and far joints (or point for the
	 *  hand's far end), and for the pelvis and trunk the two joints that give the width. */
	int32 Body = INDEX_NONE, Near = INDEX_NONE, Far = INDEX_NONE, FarPoint = INDEX_NONE, WideA = INDEX_NONE, WideB = INDEX_NONE;
	int32 FarJointB = INDEX_NONE;
	/** The model's side as now posed, Unreal frame. */
	FVector OM, AM, FM, LM;
	double SA = 1.0, SF = 1.0, SL = 1.0;
};
struct FBbSwingPart
{
	UStaticMeshComponent* C = nullptr;
	int32 Seg = 0;
	/** Rides with the trunk's far end unscaled (neck, head). */
	bool bRigid = false;
	/** The piece as the mannequin wears it, in the mesh frame. */
	FTransform Ref;
	/** Joints this piece carries: the file's index and where the joint is in the piece's own frame. */
	TArray<TPair<int32, FVector>> Carries;
	/** A hand: the file's grip point and the knuckles in the piece's frame. */
	int32 Grip = INDEX_NONE;
	FVector GripLocal = FVector::ZeroVector;
	/** A shoe: the foot's two ball-of-foot contact points and the sole under the ball in its frame. */
	int32 BallIn = INDEX_NONE, BallOut = INDEX_NONE;
	FVector SoleLocal = FVector::ZeroVector;
};


UCLASS()
class ABbBodySwing : public AActor
{
	GENERATED_BODY()

public:
	ABbBodySwing();
	virtual void Tick(float DeltaSeconds) override;

	/** Reads the file and builds the figure. Returns -1 to go on playing (or taking pictures), or
	 *  an exit status: 0 the check passed, 1 it failed or the file would not open. */
	int32 Start(const FString& InFile, UBbFigureShop* InShop, ABbPark* InPark, ACameraActor* InCamera, UStaticMesh* Sphere, UMaterialInterface* BaseMaterial);

private:
	bool BuildFigure(FString& OutError);
	void Sample(double T, FBbSwingPose& Out) const;
	void Pose(const FBbSwingPose& P);
	FVector ToCm(const FVector& Sim) const;
	FVector AxisUE(const FBbSwingPose& P, int32 Body, const FVector& Local) const;
	FVector MapPoint(const FBbSwingSeg& S, const FVector& Mesh) const;
	FVector MapVector(const FBbSwingSeg& S, const FVector& Mesh) const;
	/** -BbBodySwingCheck: every sample drawn and measured against the file. 0 passed, 1 failed. */
	int32 RunCheck();
	void SetView(int32 InView);
	void UpdateText();
	void TickShots(float DeltaSeconds);

	FBbSwingFile File;
	/** Why the file cannot be shown, on screen. */
	FString Error;
	/** -BbBodySwingCheckFault=<name>: a fault planted in the drawing so that the check must fail
	 *  (player-visuals §14): "nostretch" (the limbs left at the mannequin's lengths), "mirror" (the
	 *  places' x turned over, the turns not), "twist" (each limb and hand turned 90° about its own axis). */
	FString Fault;
	TArray<FBbSwingSeg> Segs;
	TArray<FBbSwingPart> Parts;
	UPROPERTY()
	TObjectPtr<UBbFigureShop> Shop;
	UPROPERTY()
	TObjectPtr<ABbPark> Park;
	UPROPERTY()
	TObjectPtr<ACameraActor> Camera;
	UPROPERTY()
	TObjectPtr<UStaticMeshComponent> BatC;
	UPROPERTY()
	TObjectPtr<UStaticMeshComponent> BallC;
	/** The drawn ground under the plate, cm: the file's z = 0. */
	double GroundCm = 0.0;
	/** What the cameras look at (sim frame, m). */
	FVector Target = FVector::ZeroVector;

	/** The clock, s from contact; the speed (a share of real time); paused; held at an end, s of real time left. */
	double Clock = 0.0;
	int32 SpeedStep = 0;
	bool bPaused = false;
	double HoldStart = 0.0, HoldEnd = 0.0;
	int32 View = 0;
	float OrbitYaw = 40.f, OrbitPitch = 12.f, OrbitDist = 5.5f;

	/** -BbBodySwingShots: which picture is next, and the wall clock. */
	int32 Shot = -1;
	double Wall = 0.0, ShotSince = 0.0;
	bool bShotAsked = false;
	TArray<TPair<FString, double>> ShotTimes;
};
