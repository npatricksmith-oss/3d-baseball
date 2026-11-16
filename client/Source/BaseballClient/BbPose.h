// Baseball motions written in code as poses (docs/player-visuals.md, step 3; owner's Q16).
//
// A pose is a stick figure: where the hips are and which way they face, which way the chest
// and head face, where each wrist and ankle is, and which way each elbow and knee points.
// FBbSkeleton turns one into the mannequin's bone transforms (two-bone IK for the limbs), and
// UBbPoseAnim lays that over whatever stock clip is playing, by a weight, so a figure can go
// from standing into a motion and back. The motions themselves (BbMotion) are tables of key
// poses over the sim's own clock: the hand is put on the sim's release point at the sim's
// release time, the mitt on the ball where the sim has it caught (invariant 4).
#pragma once

#include "CoreMinimal.h"
#include "Animation/AnimSingleNodeInstance.h"
#include "Animation/AnimSingleNodeInstanceProxy.h"
#include "BbPose.generated.h"

class USkeletalMesh;

/** A pose as a stick figure, in the mannequin's own frame: cm, +Y the way he faces, +X his
 *  left, +Z up, the origin on the ground under him. Every field is a vector so that poses
 *  can be blended field by field (FBbStick::V). */
struct FBbStick
{
	/** The pelvis joint, and which way the hips face and stand. */
	FVector Pelvis{ 0, 2, 96 }, PelvisFwd{ 0, 1, 0 }, PelvisUp{ 0, 0, 1 };
	FVector ChestFwd{ 0, 1, 0 }, ChestUp{ 0, 0, 1 };
	FVector HeadFwd{ 0, 1, 0 }, HeadUp{ 0, 0, 1 };
	/** Each wrist, from its own shoulder, in the chest's frame (so an arm goes round with the
	 *  trunk); which way the elbow points, in the same frame; which way the fingers point, in
	 *  the figure's frame (zero: on along the forearm). 0 left, 1 right. */
	FVector Wrist[2]{ { 6, 6, -50 }, { -6, 6, -50 } };
	FVector Elbow[2]{ { 1, -0.5, -0.3 }, { -1, -0.5, -0.3 } };
	FVector Fingers[2]{ FVector::ZeroVector, FVector::ZeroVector };
	/** Each ankle: across, along, and how far the foot is off the ground (BbMotion puts the
	 *  ground under it); which way the knee points; which way the toe points. */
	FVector Ankle[2]{ { 14, 0, 0 }, { -14, 0, 0 } };
	FVector Knee[2]{ { 0, 1, 0 }, { 0, 1, 0 } };
	FVector Toe[2]{ { 0, 1, 0 }, { 0, 1, 0 } };

	static constexpr int32 Num = 19;
	FVector* V() { return &Pelvis; }
	const FVector* V() const { return &Pelvis; }
};
static_assert(sizeof(FBbStick) == FBbStick::Num * sizeof(FVector), "FBbStick is vectors only");

/** A wrist that must be at a point whatever the pose says (the release point, the ball in the
 *  mitt): the point in the figure's frame and how much of it applies. The body is moved as
 *  far as it takes for the arm to reach. */
struct FBbReach
{
	FVector At[2]{ FVector::ZeroVector, FVector::ZeroVector };
	float Weight[2]{ 0.f, 0.f };
	/** The point is where something held in the hand is to be, this far past the wrist, cm. */
	float Past[2]{ 0.f, 0.f };
};

/** The mannequin's joints as the solver needs them, read once from its skeleton. */
struct FBbSkeleton
{
	bool Init(const USkeletalMesh* Mesh);
	bool Ok() const { return bOk; }
	/** The pose as bone transforms, each relative to its parent, for every bone of the mesh.
	 *  OutHold: where a ball held in each hand is, in the figure's frame; OutWrist: each wrist. */
	void Solve(const FBbStick& S, const FBbReach& Reach, TArray<FTransform>& OutLocal, FVector OutHold[2], FVector OutWrist[2]) const;
	/** The ankle joint's height when the foot is flat on the ground, cm. */
	double AnkleCm() const { return AnkleZ; }
	double ArmCm() const { return UpperArm + ForeArm; }

private:
	enum ERole : uint8 { Free, Pelvis, Spine, Neck, Head, UpperArm0, LowerArm0, Hand0, Thigh0, Calf0, Foot0 };
	struct FBone
	{
		int32 Parent = INDEX_NONE;
		FTransform RefLocal, Ref;
		uint8 Role = Free;
		/** Left 0, right 1 for a limb; the share of the way up for the spine and neck. */
		int32 Side = 0;
		float Share = 0.f;
		/** Where the next joint lies from this one as the mannequin stands, unit. */
		FVector Along = FVector::ZeroVector;
	};
	TArray<FBone> Bones;
	double UpperArm = 27.0, ForeArm = 27.0, Thigh = 43.0, Calf = 42.0, AnkleZ = 8.0;
	bool bOk = false;
};

/** What a motion needs to know about where the figure stands. */
struct FBbMotionGround
{
	/** Height of the drawn ground at a point of the figure's frame, against the ground at its
	 *  origin, cm in the figure's own scale. */
	TFunctionRef<double(double X, double Y)> Height;
};

namespace BbMotion
{
	/** The delivery from the stretch, by seconds from the sim's release (negative before):
	 *  set, leg lift, stride, plant, the arm laid back, release, follow-through, finish.
	 *  Right-handed; a left-hander is the same shown in a mirror. Positions are from the
	 *  pitcher's place on the rubber. */
	FBbStick Delivery(float T, const FBbSkeleton& Skel, const FBbMotionGround& Ground);
	/** How much of the sim's release point the throwing hand takes at T: all of it at 0. */
	float DeliveryReach(float T);
	/** Where the figure's own origin has got to at T, from the rubber (he finishes down the
	 *  mound, and stands there afterwards). */
	FVector2D DeliveryTravel(float T);
	/** The delivery's first and last moments, s from release. */
	constexpr float DeliveryStartS = -1.4f;
	constexpr float DeliveryEndS = 0.6f;

	/** The catcher's crouch. MittAt: where the ball is to be received, in the figure's frame.
	 *  The mitt's wrist target comes back in OutReach (the left arm; a left-handed catcher is
	 *  shown in a mirror). Positions are from the catcher's place as the sim has it; he sits
	 *  CrouchBack behind it, so that the mitt is out in front of him. */
	FBbStick Crouch(const FVector& MittAt, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach);
	/** Where the crouched figure's origin sits from the catcher's place. */
	const FVector2D CrouchBack(0.0, -66.0);
	/** How far past the wrist the ball sits in a hand, and in a mitt's pocket, cm. */
	constexpr double BallInHandCm = 9.0;
	constexpr double BallInMittCm = 14.0;

	/** What the batter's pose is made from (player-visuals §9). Everything is in his own
	 *  frame as he is shown at the plate: facing the pitcher (+Y), the plate on his right
	 *  (-X), standing on the sim's spot for him. That is a right-handed batter; a left-handed
	 *  one is the same shown in a mirror. */
	struct FSwingIn
	{
		/** The bat he is holding: the sim's while the sim has one, cm. */
		FVector Knob = FVector::ZeroVector, Tip = FVector(0, 0, 1);
		/** What he is watching. */
		FVector Look = FVector(0, 1700, 170);
		/** How far round his hands have come since the swing began, degrees (HandsDeg now less
		 *  HandsDeg at its least); and how much of a swing is on, 0 standing to 1 swinging. */
		float TurnDeg = 0.f, Swing = 0.f;
		/** The front foot's step toward the pitcher, 0 in the stance to 1 planted. */
		float Stride = 0.f;
		/** 0 in the stance, 1 at ease between pitches. */
		float Rest = 0.f;
	};
	/** The batter: stance, stride, the turn of hips and shoulders after the bat, both hands on
	 *  the handle. The body follows the bat, never the other way (invariant 4): OutReach puts
	 *  the lower hand (his left) and the upper hand on the handle wherever the bat is. */
	FBbStick Swing(const FSwingIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach);
	/** Where his hands are round him, seen from above, degrees: 0 on the plate's side of him,
	 *  +90 toward the pitcher, -90 toward the catcher. */
	double HandsDeg(const FVector& Knob, const FVector& Tip);
	/** Where on the handle each hand is, from the knob, and how far the handle sits past a
	 *  wrist, cm. */
	constexpr double GripLowCm = 6.0;
	constexpr double GripHighCm = 15.0;
	constexpr double GripPastCm = 7.0;
	/** The bat at ease between pitches, on the back shoulder: knob and direction, his frame. */
	const FVector RestKnob(-14.0, -8.0, 100.0);
	const FVector RestAxis(0.35, -0.45, 0.82);
}

/** The single-clip player with a pose laid over it. */
struct FBbPoseProxy : public FAnimSingleNodeInstanceProxy
{
	FBbPoseProxy(UAnimInstance* In) : FAnimSingleNodeInstanceProxy(In) {}
	virtual void PreUpdate(UAnimInstance* InAnimInstance, float DeltaSeconds) override;
	virtual bool Evaluate(FPoseContext& Output) override;

	TArray<FTransform> Local;
	float Weight = 0.f;
};

UCLASS(Transient)
class UBbPoseAnim : public UAnimSingleNodeInstance
{
	GENERATED_BODY()

public:
	/** Bone transforms (each relative to its parent, by the mesh's bone index) to lay over
	 *  the clip, and how much: 0 the clip alone, 1 the pose alone. */
	TArray<FTransform> PoseLocal;
	float PoseWeight = 0.f;

protected:
	virtual FAnimInstanceProxy* CreateAnimInstanceProxy() override;
};
