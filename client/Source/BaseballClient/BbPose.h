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
	/** Which bones are the trunk, head and arms (the rest: the hips, the legs and the root),
	 *  by the mesh's bone index; and which bone is the pelvis. A motion made on the run lays
	 *  only the upper ones over the clip, so the legs keep running (UBbPoseAnim::LowerWeight). */
	TArray<bool> UpperBones() const;
	int32 PelvisBone() const { return PelvisIndex; }
	/** Which way hips face and stand, given the pelvis bone's transform in the figure's frame. */
	void PelvisFacing(const FTransform& Bone, FVector& OutFwd, FVector& OutUp) const;

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
	int32 PelvisIndex = INDEX_NONE;
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

	/** The bat before the sim's own moves (player-visuals §13.2-§13.4, the base swing's build
	 *  step 1). Times are play time, s from the pitcher's release; the batter's frame as above,
	 *  cm. All named and configurable [E]; "by eye" and "from [4]" are marked in §13.3. */
	/** The hands start to drift back from the stance (the pitcher in his stride): R -0.34 s. */
	constexpr float BatSettleS = -0.34f;
	/** The hands reach the load pose L: R +0.198 s, or sooner on a fast pitch (below). */
	constexpr float BatLoadAtS = 0.198f;
	/** The weight commitment comes this long before the pitch reaches the plane (T_swing 0.150
	 *  s: 0.170 s; T_swing is not known before the swing, so this is what a take has too, D22),
	 *  and the hands are at L this long before it at the latest. */
	constexpr float BatCommitLeadS = 0.170f;
	constexpr float BatLoadBeforeCommitS = 0.052f;
	/** The handover from the client's bat to the sim's takes at least this long from the moment
	 *  the sim's bat first moves. */
	constexpr float BatHandoverS = 0.03f;
	/** A pitch taken: the bat eases back from L toward the stance this long after the catch,
	 *  over this long. */
	constexpr float BatRelaxAfterCatchS = 0.2f;
	constexpr float BatRelaxOverS = 0.6f;
	/** The load pose L: the mean of the sim's swing-start pose (swing_pose_probe, step 1). */
	extern const FVector LoadKnob;
	extern const FVector LoadAxis;
	/** The client's bat at play time T: the stance, then from BatSettleS drifting to L by a
	 *  smooth curve, reaching it at min(BatLoadAtS, Ta - BatCommitLeadS - BatLoadBeforeCommitS),
	 *  then still; on a pitch taken (bTake) easing back to the stance after the catch (CatchT).
	 *  The same on every pitch, a swing or a take, until the sim's bat moves (D22). */
	void LoadBat(float T, float Ta, bool bTake, float CatchT, const FVector& StanceKnob, const FVector& StanceAxis, FVector& OutKnob, FVector& OutAxis);
	/** How much of the drawn bat is the sim's, 0 to 1 and smooth: from TL, the moment the sim's
	 *  bat first moves, to TS (the swing's start) or TL + BatHandoverS if later. */
	float HandoverWeight(float T, float TL, float TS);

	/** What a fielder's pose is made from (player-visuals §10). Everything is in the frame of
	 *  the figure as it is drawn: +Y the way he faces. A right-handed thrower, the glove on
	 *  his left hand; a left-hander is the same shown in a mirror. */
	struct FFieldIn
	{
		/** The ready position as the pitch is thrown: 0 standing easy to 1 down and set. */
		float Ready = 0.f;
		/** The glove to the ball: where the ball is to be caught, and how much of the reach is
		 *  on (1 as the ball arrives). */
		FVector GloveAt = FVector(0, 40, 110);
		float Glove = 0.f;
		/** He has the ball: hands together in front of him. */
		float Carry = 0.f;
		/** The throw, toward +Y: seconds from the sim's release, and how much of it shows. */
		float ThrowT = 0.f, Throw = 0.f;
		/** How far he sinks to throw, cm: the sim's release point is at one height over its
		 *  level field, and a man on the mound stands above that. */
		float ThrowSink = 0.f;
		/** What he is watching. */
		FVector Look = FVector(0, 1000, 150);
		/** How much the legs and hips are the pose's (1, standing) or the clip's (0, running);
		 *  and the hips as the clip has them, which the trunk is then built on. */
		float Lower = 1.f;
		FVector ClipPelvis = FVector(0, 2, 96), ClipFwd = FVector(0, 1, 0), ClipUp = FVector(0, 0, 1);
	};
	/** The fielder: ready, the glove to the ball, the ball brought in to the throwing hand,
	 *  the throw. OutReach puts the glove's pocket on GloveAt; the caller adds the throwing
	 *  hand's reach to the sim's release point (ThrowReach). OutPocket: which way the glove's
	 *  fingers point, the ball sitting BallInMittCm that way from the wrist. */
	FBbStick Field(const FFieldIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach, FVector& OutPocket);
	/** How much of the sim's release point the throwing hand takes at T: all of it at 0. */
	float ThrowReach(float T);
	/** How far off the ground he goes for a ball this high over his feet, cm. */
	double JumpForCm(double BallZ);
	/** The throw's first and last moments, s from release. */
	constexpr float ThrowStartS = -0.45f;
	constexpr float ThrowEndS = 0.5f;

	/** A fielder's dive (player-visuals §11): he drives off, is laid out in the air, lands on
	 *  his front and lies there, then gets up through his hands and knees. Since: seconds
	 *  since he left his feet; UpIn: seconds until the sim has him up. The figure faces the
	 *  way he dove and its origin is the sim's place for him, which is his middle as he lies.
	 *  GloveAt and Glove as for Field: OutReach puts the glove's pocket on the ball. */
	FBbStick Dive(float Since, float UpIn, const FVector& GloveAt, float Glove, const FVector& Look, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach, FVector& OutPocket);
	/** How long the fall takes and how long the getting up, s. */
	constexpr float DiveFallS = 0.3f;
	constexpr float DiveRiseS = 0.9f;

	/** A runner's feet-first slide (player-visuals §11), by seconds from the moment the sim
	 *  has him at the bag (negative before): he drops onto his left hip with that leg folded
	 *  under, the right foot out in front for the bag, sits there a moment and stands up on
	 *  it. The figure faces the way he ran; its origin is the sim's place for him, which is
	 *  his leading foot once he is down. */
	FBbStick Slide(float T, const FBbSkeleton& Skel, const FBbMotionGround& Ground);
	/** The slide's first and last moments, s from his arrival. */
	constexpr float SlideStartS = -0.42f;
	constexpr float SlideEndS = 1.2f;

	/** A runner's dive back to a bag (player-visuals §11; the sim's dive back, agent-models
	 *  §6.4): from the moment he leaves his feet he is laid out headfirst, his right hand
	 *  reaching for the bag; it is on the bag when the sim has it touch, he lies there, and
	 *  gets up onto the bag through his hands and knees as the sim has him up. Put out in the
	 *  air (bTouched false), he lies where the glide ended, no hand on a bag, and gets up there.
	 *  Since: seconds since he left his feet (negative before: the pose's first key); EndIn:
	 *  seconds until his glide ends (negative after); UpIn: seconds until the sim has him up.
	 *  The figure faces the way he dove; its origin is where the sim has his touching hand
	 *  (his path point at take-off, moving onto the hand over the first half of the glide).
	 *  OutReach puts the right wrist on the origin while the hand is on the bag. */
	FBbStick DiveBack(float Since, float EndIn, float UpIn, bool bTouched, const FBbSkeleton& Skel, const FBbMotionGround& Ground, FBbReach& OutReach);
	/** How long the getting up onto the bag takes, s (the sim has him down DIVE_BACK_UP_S,
	 *  1.0 s, from the touch). */
	constexpr float DiveBackRiseS = 0.7f;

	/** The people who are not playing (player-visuals §12): umpires, base coaches, the men
	 *  on the bench. Everything is in the figure's own frame, +Y the way he faces. */
	enum class ESignal : uint8 { None, Strike, Out, Safe, Foul, Fair };
	struct FCrewIn
	{
		/** 0 standing easy to 1 down for the pitch, hands on his knees. */
		float Set = 0.f;
		/** The plate umpire's set is lower, behind the catcher. */
		bool bPlate = false;
		/** Standing easy with his hands on his hips (a coach, a manager) instead of at his sides. */
		float Hips = 0.f;
		/** An umpire's signal and the seconds since it began. */
		ESignal Signal = ESignal::None;
		float SignalT = 0.f;
		/** For a Fair signal: the way to fair ground in his own frame (x toward his left, y the
		 *  way he faces; unit). He points the arm on that side. */
		FVector2D Point = FVector2D(1.0, 0.0);
		/** What he is watching. */
		FVector Look = FVector(0, 1000, 150);
	};
	FBbStick Crew(const FCrewIn& In, const FBbSkeleton& Skel, const FBbMotionGround& Ground);
	/** How long a signal is held, s. */
	constexpr float SignalS = 1.5f;
	/** A man sitting on a bench SeatCm above the floor he has his feet on; the figure's origin
	 *  is on the floor under the bench's front edge. Variant: forearms on his thighs, sitting
	 *  up with his hands on his thighs, or leaning back with his arms folded. */
	FBbStick Seated(int32 Variant, double SeatCm, const FVector& Look, const FBbSkeleton& Skel);
}

/** The single-clip player with a pose laid over it. */
struct FBbPoseProxy : public FAnimSingleNodeInstanceProxy
{
	FBbPoseProxy(UAnimInstance* In) : FAnimSingleNodeInstanceProxy(In) {}
	virtual void PreUpdate(UAnimInstance* InAnimInstance, float DeltaSeconds) override;
	virtual bool Evaluate(FPoseContext& Output) override;

	TArray<FTransform> Local;
	TArray<bool> Upper;
	float Weight = 0.f;
	float LowerWeight = 1.f;
	int32 PelvisBone = INDEX_NONE;
	/** The pelvis as the clip alone has it, in the figure's frame, from the last evaluation. */
	FTransform ClipPelvis = FTransform::Identity;
	bool bClipPelvis = false;
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
	/** How much of the pose the hips and legs take, as a share of PoseWeight: 1 the whole
	 *  figure, 0 the trunk, head and arms only (UpperBones says which those are). */
	float LowerWeight = 1.f;
	TArray<bool> UpperBones;
	int32 PelvisBone = INDEX_NONE;
	/** The pelvis as the clip alone had it when the figure was last worked out, in the
	 *  figure's frame; false until it has been. */
	FTransform ClipPelvis = FTransform::Identity;
	bool bClipPelvis = false;

protected:
	virtual FAnimInstanceProxy* CreateAnimInstanceProxy() override;
};
