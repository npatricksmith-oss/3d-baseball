// What a player is made of (docs/player-visuals.md, step 1): shaped pieces riding on the
// bones of the engine's mannequin, which stays as an unseen driver so that every motion
// that moved the mannequin moves the figure. The pieces are generated here in code, as the
// park is; the uniform's colours and the sizes of bodies are in BbLook.h.
#pragma once

#include "CoreMinimal.h"
#include "Animation/SkeletalMeshActor.h"
#include "bb_ffi.h"
#include "BbFigure.generated.h"

class ABbPark;
class UStaticMesh;
class UStaticMeshComponent;
class UMaterialInstanceDynamic;
class UTexture2D;
class USkeletalMesh;

/** What a piece is dressed in. */
enum class EBbCloth : uint8
{
	Jersey,
	/** The chest: the jersey with the club's name on the front and the number on the back. */
	Chest,
	Pants,
	Socks,
	Belt,
	Cap,
	Bill,
	Shoe,
	Skin,
	Hair,
	/** The kit (step 2): a hard hat in the cap's colour, the catcher's gear in the club's
	 *  trim, his mask, a glove's leather. */
	Helmet,
	Gear,
	Mask,
	Leather,
};

/** When a piece is worn. */
enum class EBbWear : uint8
{
	Always,
	/** The cap and its bill: off under a helmet. */
	Cap,
	/** The helmet's shell and ear flaps: a batter, a runner, the catcher. */
	Helmet,
	/** The helmet's bill: a batter or runner (the catcher's has none, under the mask). */
	HelmetBill,
	/** Mask, chest protector, shin guards. */
	Catcher,
	/** A glove on the bone of that name's hand. */
	GloveL,
	GloveR,
};

/** What a figure is on the field to do, which decides his kit. */
enum class EBbRole : uint8
{
	/** Cap, bare hands. */
	Plain,
	/** Cap and glove. */
	Fielder,
	/** Helmet, mask, chest protector, shin guards, a larger glove. */
	Catcher,
	/** A batter or runner: helmet, bare hands. */
	Batter,
	/** The umpire behind the plate: a cap and a mask. */
	PlateUmpire,
};

/** One ring of a piece turned about its own axis: where along the axis (cm), its half-widths
 *  across and front to back, and how far its centre sits off the axis toward the front. */
struct FBbRing
{
	double Z, Rx, Ry, Oy;
};

/** A piece as every figure has it: its mesh, the bone it rides on and where it sits there. */
struct FBbPiece
{
	FName Bone;
	FTransform OnBone;
	EBbCloth Cloth;
	/** Thicker on a heavier man. */
	bool bBuild;
	int32 Mesh;
	EBbWear Wear = EBbWear::Always;
};

/** The pieces, paints and jerseys shared by every figure: made once, from the mannequin's
 *  own skeleton, so a piece runs from joint to joint wherever the joints are. */
UCLASS()
class UBbFigureShop : public UObject
{
	GENERATED_BODY()

public:
	/** False (with a warning logged) if the skeleton lacks a bone the figure needs. */
	bool Init(USkeletalMesh* Skeleton, ABbPark* InPark);
	const TArray<FBbPiece>& Pieces() const { return Parts; }
	UStaticMesh* MeshOf(const FBbPiece& P) const { return Meshes[P.Mesh]; }
	/** The paint for a piece of a side's uniform (0 away, 1 home) on a body. */
	UMaterialInstanceDynamic* PaintFor(EBbCloth Cloth, int32 Side, int32 Skin, int32 Hair) const;
	/** The chest's picture: Number 0 leaves the back plain. bMirror: for a figure shown in a
	 *  mirror (a left-hander's motion), painted backwards so that it still reads. */
	UTexture2D* Jersey(int32 Side, int32 Number, bool bMirror);
	UMaterialInstanceDynamic* NewChest(UObject* Outer) const;
	/** The mannequin's standing height, cm: what an actor scale of 1 gives. */
	double StandingCm() const { return Standing; }
	/** The bat, turned on the same lathe: its knob at the origin, BatCm long along Z. */
	UStaticMesh* BatMesh() const { return Bat; }
	UMaterialInstanceDynamic* BatPaint() const { return WoodPaint; }
	static constexpr double BatCm = 86.0;
	/** A bone of the mannequin as it stands when made (its frame in the mesh's own frame, cm):
	 *  what the pieces were placed from. False if the skeleton has no such bone. */
	bool RefBone(FName Bone, FTransform& Out) const
	{
		const FTransform* T = RefCs.Find(Bone);
		if (T)
		{
			Out = *T;
		}
		return T != nullptr;
	}

private:
	UStaticMesh* Lathe(const TArray<FBbRing>& Rings, int32 Sides);

	UPROPERTY()
	TObjectPtr<ABbPark> Park;
	UPROPERTY()
	TArray<TObjectPtr<UStaticMesh>> Meshes;
	/** Club paints: [side][cloth]. */
	UPROPERTY()
	TArray<TObjectPtr<UMaterialInstanceDynamic>> ClubPaints;
	UPROPERTY()
	TArray<TObjectPtr<UMaterialInstanceDynamic>> SkinPaints;
	UPROPERTY()
	TArray<TObjectPtr<UMaterialInstanceDynamic>> HairPaints;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> ShoePaint;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> BeltPaint;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> MaskPaint;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> LeatherPaint;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> WoodPaint;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Bat;
	UPROPERTY()
	TMap<int32, TObjectPtr<UTexture2D>> Jerseys;
	TArray<FBbPiece> Parts;
	TMap<FName, FTransform> RefCs;
	double Standing = 180.0;
};

UCLASS()
class ABbFigure : public ASkeletalMeshActor
{
	GENERATED_BODY()

public:
	/** Puts the pieces on the bones and hides the mannequin. */
	void Build(UBbFigureShop* InShop);
	/** Dresses the figure as a player: his club's uniform, his number, and a height, build
	 *  and skin tone that follow from his id (the same every time). Without a player, a
	 *  plain uniform of the side. Does nothing if he is dressed so already. */
	void Dress(const BbPlayer* Player, int32 Side);
	/** His kit for what he is doing (docs/player-visuals.md, step 2). The glove goes on the
	 *  hand he does not throw with, as the viewer sees it, mirror or no mirror. */
	void SetRole(EBbRole InRole);
	/** Shown in a mirror across his own facing (a left-hander's motion from a right-handed
	 *  one). The lettering is repainted so that it does not read backwards. */
	void SetMirror(bool bMirror);
	/** Whether his pieces throw shadows. A man under a dugout's roof needs none, and they are
	 *  a good part of what a figure costs to draw. */
	void SetShadows(bool bCast);

private:
	void Apply();
	void ApplyKit();

	UPROPERTY()
	TObjectPtr<UBbFigureShop> Shop;
	UPROPERTY()
	TArray<TObjectPtr<UStaticMeshComponent>> Parts;
	UPROPERTY()
	TObjectPtr<UMaterialInstanceDynamic> ChestPaint;
	uint32 Dressed = 0;
	int32 Side = 1, Number = 0;
	float Height = 1.f;
	bool bMirrored = false;
	bool bThrowsLeft = false;
	EBbRole Role = EBbRole::Plain;
};
